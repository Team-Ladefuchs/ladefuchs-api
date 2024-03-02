use std::path::{Path, PathBuf};

use eyre::Context;
use hotwatch::{
    blocking::{Flow, Hotwatch},
    notify::event::{CreateKind, DataChange, ModifyKind, RemoveKind, RenameMode},
    Event,
};
use once_cell::sync::Lazy;
use sqlx::{Connection, PgConnection, Pool, Postgres};

use crate::{
    image_import::{insert_or_update, ImageFolder},
    io::hash_file,
};

pub static REGEX_IMAGE_FILENAME: Lazy<regex::Regex> = Lazy::new(|| {
    regex::RegexBuilder::new(
        r#"^(?:card_|cpo_|banner_){0,1}([a-zA-Z0-9-._ß+]+)\.(?:jpg|jpeg|png|svg|gif)$"#,
    )
    .case_insensitive(true)
    .build()
    .unwrap()
});

static REGEX_RELATIVE_IMAGE_PATH: Lazy<regex::Regex> = Lazy::new(|| {
    regex::RegexBuilder::new(r#"(images/(?:cards|banners|cpos))/.*$"#)
        .case_insensitive(true)
        .build()
        .unwrap()
});

use crate::{
    db::image::{self, delete_marked},
    importer, io,
    slack::{self, Emoji, Slack, SlackClient},
    state::State,
};

pub fn cleanup_task(state: State) {
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(importer::hours(1));
        loop {
            interval.tick().await;
            if let Ok(mut cxn) = state.as_ref().database_pool.acquire().await {
                if let Err(err) = delete_marked(&mut cxn).await {
                    tracing::error!(task="Delete marked card images", err=?err);
                };
            };
        }
    });
}

pub fn watch_image_folder<T>(state: State, image_folder: T) -> Result<(), eyre::Error>
where
    T: ImageFolder,
{
    cleanup_task(state.clone());
    tokio::task::spawn_blocking(move || {
        let folder = image_folder.folder_parent().to_owned();
        let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize");
        tracing::info!("Start watching {} folder for watching", folder.display());
        let image_folder = image_folder.clone();
        hotwatch
            .watch(&folder, move |event: Event| {
                let state = state.clone();
                let image_folder = image_folder.clone();
                tokio::task::spawn(async move {
                    let slack = &state.slack;
                    let context = HandleContext {
                        slack,
                        database_pool: &state.database_pool,
                        image_folder: &image_folder,
                    };
                    let ret = handle_fs_event(event, context).await;
                    if let Err(err) = ret {
                        tracing::warn!(msg = "While watching the folder", err = ?err);
                        let text = format!("{} Something went wrong:\n{}", slack::MALIK, err);
                        slack.send(Some(Emoji::Warning), &text).await;
                    }
                });
                Flow::Continue
            })
            .with_context(|| format!("failed to watch path {}", folder.display()))?;
        hotwatch.run();
        Ok::<(), eyre::Error>(())
    });

    Ok(())
}

pub fn to_relative_image_path<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    REGEX_RELATIVE_IMAGE_PATH
        .captures(&path.as_ref().to_string_lossy().to_string())
        .and_then(|captures| captures.get(0))
        .map(|s| PathBuf::from(s.as_str()))
}

struct HandleContext<'a, T>
where
    T: ImageFolder,
{
    slack: &'a Option<Slack>,
    database_pool: &'a Pool<Postgres>,
    image_folder: &'a T,
}

async fn handle_fs_event<T>(event: Event, context: HandleContext<'_, T>) -> Result<(), eyre::Error>
where
    T: ImageFolder,
{
    match event.kind {
        hotwatch::EventKind::Create(CreateKind::File)
        | hotwatch::EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
            let Some(path) = event.paths.first().and_then(|p| to_relative_image_path(p)) else {
                return Ok(());
            };

            let mut connection = context.database_pool.acquire().await?;
            tracing::info!(event = "Event::Create|Write", ?path);

            match detect_rename(&mut connection, &path).await {
                Some(old_path) => {
                    tracing::info!(msg = "File is already known. It will be renamed", old=?old_path, new=?path);
                    rename_path(
                        &mut connection,
                        &RenameContext {
                            old_path: &old_path,
                            new_path: &path,
                            image_folder: context.image_folder,
                        },
                    )
                    .await?;
                }
                None => {
                    insert_or_update(&mut connection, &path, context.image_folder).await?;
                    tracing::info!(event = "Event::Create|Write", file=%path.display());
                }
            }
            let filename = path.file_name().unwrap_or_default();
            context
                .slack
                .send_new_image_slack(context.image_folder.id(), filename)
                .await;
        }

        hotwatch::EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
            let [from_path, to_path] = &event
                .paths
                .iter()
                .filter_map(|pp| to_relative_image_path(pp))
                .collect::<Vec<_>>()[..]
            else {
                return Ok(());
            };

            tracing::info!(event = "Event::Rename", old=?from_path, new=?to_path);
            let mut connection = context.database_pool.acquire().await?;
            rename_path(
                &mut connection,
                &RenameContext {
                    old_path: &from_path,
                    new_path: &to_path,
                    image_folder: context.image_folder,
                },
            )
            .await?;
            let old_file = from_path.file_name().unwrap_or_default();
            let new_file = to_path.file_name().unwrap_or_default();
            context
                .slack
                .send_rename_image(context.image_folder.id().0, old_file, new_file)
                .await;
        }
        hotwatch::EventKind::Remove(RemoveKind::File) => {
            let Some(path) = event.paths.first().and_then(|p| to_relative_image_path(p)) else {
                return Ok(());
            };

            tracing::info!(event = "Event::Remove", ?path);
            let mut connection = context.database_pool.acquire().await?;
            image::soft_delete(&mut connection, &path).await?
        }
        _ => {}
    }
    Ok(())
}

struct RenameContext<'a, T>
where
    T: ImageFolder,
{
    old_path: &'a PathBuf,
    new_path: &'a PathBuf,
    image_folder: &'a T,
}

async fn rename_path<T>(
    connection: &mut PgConnection,
    context: &RenameContext<'_, T>,
) -> Result<(), eyre::Error>
where
    T: ImageFolder,
{
    // todo check if path is  an image
    io::guess_image_mime(context.new_path).await?;

    let filename = parse_filename(&context.new_path)?;
    tracing::info!(
        msg = "Updating path",
        old=?context.old_path,
        new=?context.new_path,
        filename=?filename
    );

    let mut transaction = connection.begin().await?;

    match image::update_name_path(&mut transaction, context.old_path, context.new_path).await? {
        Some(id) => {
            context
                .image_folder
                .set_internal_name(&mut transaction, id, &filename)
                .await?;
            transaction.commit().await?;
        }
        None => tracing::info!(msg = "Could not rename file", path = ?context.old_path, filename),
    }

    Ok(())
}

pub fn parse_filename(path: &Path) -> Result<String, eyre::Error> {
    let raw_filename = path
        .file_name()
        .ok_or_else(|| eyre::Error::msg("Unsupported filename"))?
        .to_string_lossy();

    let captures = REGEX_IMAGE_FILENAME
        .captures(&raw_filename)
        .and_then(|c| c.get(1));

    match captures {
        Some(group) => Ok(group.as_str().to_owned()),
        None => Err(eyre::Error::msg(format!(
            "Wrong formatted filename: {}",
            raw_filename
        ))),
    }
}

async fn detect_rename(connection: &mut PgConnection, path: &PathBuf) -> Option<PathBuf> {
    let checksum = hash_file(path).await.ok()?;

    let card_image = image::get_by_checksum(connection, &checksum).await.ok();
    card_image.map(|card| card.file_path)
}
