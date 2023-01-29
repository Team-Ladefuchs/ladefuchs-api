use std::path::PathBuf;

use crate::{image_import::ImageFolder, io::hash_file};

use eyre::Context;
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use once_cell::sync::Lazy;
use sqlx::{pool::PoolConnection, Acquire, Pool, Postgres};

pub static REGEX_FILENAME: Lazy<regex::Regex> = Lazy::new(|| {
    regex::RegexBuilder::new(
        r#"^(?:card_|cpo_){0,1}([a-zA-Z0-9-._ß+]+)\.(?:jpg|jpeg|png|svg|gif)$"#,
    )
    .case_insensitive(true)
    .build()
    .unwrap()
});

use crate::{
    db::image::{self, delete_marked},
    image_import::insert_or_update,
    importer, io,
    slack::{self, MessageEmoji, Slack, SlackClient},
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

pub fn watch_cards_folder<T>(state: State, image_folder: T) -> Result<(), eyre::Error>
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
                        slack.send(Some(MessageEmoji::Warning), &text).await;
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
    match event {
        Event::Write(path) | Event::Create(path) if io::is_file(&path).await? => {
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
        Event::Rename(old_path, new_path) if io::is_file(&new_path).await? => {
            tracing::info!(event = "Event::Rename", old=?old_path, new=?new_path);
            let mut connection = context.database_pool.acquire().await?;
            rename_path(
                &mut connection,
                &RenameContext {
                    old_path: &old_path,
                    new_path: &new_path,
                    image_folder: context.image_folder,
                },
            )
            .await?;
            let old_file = old_path.file_name().unwrap_or_default();
            let new_file = new_path.file_name().unwrap_or_default();
            context
                .slack
                .send_rename_image(context.image_folder.id(), old_file, new_file)
                .await;
        }
        Event::Remove(path) => {
            tracing::info!(event = "Event::Remove", ?path);
            let mut connection = context.database_pool.acquire().await?;
            image::soft_delete(&mut connection, &path).await?
        }
        Event::Error(error, path) => {
            context
                .slack
                .send(
                    Some(MessageEmoji::Error),
                    &format!("An Error has occurred: {:#?},\tpath {:#?}", error, path),
                )
                .await;
            tracing::error!("Error::Event {}, path: {:#?}", error, path);
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
    connection: &mut PoolConnection<Postgres>,
    context: &RenameContext<'_, T>,
) -> Result<(), eyre::Error>
where
    T: ImageFolder,
{
    // todo check if path is  an image
    let raw_filename = context
        .new_path
        .file_name()
        .ok_or_else(|| eyre::Error::msg("Unsupported filename"))?
        .to_string_lossy();

    io::guess_image_mime(context.new_path).await?;

    let filename = parse_filename(&raw_filename)?;
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
        None => tracing::info!(msg = "Could not rename file", path = ?context.old_path),
    }

    Ok(())
}

pub fn parse_filename(name: &str) -> Result<String, eyre::Error> {
    let captures = REGEX_FILENAME.captures(name).and_then(|c| c.get(1));

    match captures {
        Some(group) => Ok(group.as_str().to_owned()),
        None => Err(eyre::Error::msg(format!(
            "Wrong formatted filename: {}",
            name
        ))),
    }
}

async fn detect_rename(
    connection: &mut PoolConnection<Postgres>,
    path: &PathBuf,
) -> Option<PathBuf> {
    let checksum = hash_file(path).await.ok()?;

    let card_image = image::get_by_checksum(connection, &checksum).await.ok();
    card_image.map(|card| card.file_path)
}
