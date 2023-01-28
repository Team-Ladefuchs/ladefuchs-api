use std::path::{Path, PathBuf};

use crate::{
    db::{
        cpo,
        image::{self, delete_marked, Image, ImageContext},
        tariff,
    },
    importer, io,
    slack::{self, MessageEmoji, Slack, SlackClient},
    state::State,
};

use axum::async_trait;
use eyre::Context;
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use once_cell::sync::Lazy;
use sqlx::{pool::PoolConnection, Acquire, Pool, Postgres, Transaction};
use tokio::fs;

static REGEX_FILENAME: Lazy<regex::Regex> = Lazy::new(|| {
    regex::RegexBuilder::new(
        r#"^(?:card_|cpo_){0,1}([a-zA-Z0-9-._ß+]+)\.(?:jpg|jpeg|png|svg|gif)$"#,
    )
    .case_insensitive(true)
    .build()
    .unwrap()
});

static CARDS_FOLDER: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("./images/cards"));

static CPOS_FOLDER: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("./images/cpos"));

pub struct CardFolder<'a> {
    folder_parent: &'a Path,
}

#[async_trait]
impl ImageImport for CardFolder<'_> {
    fn new() -> Self {
        Self {
            folder_parent: CARDS_FOLDER.as_path(),
        }
    }

    async fn get_id_by_name(
        &self,
        connection: &mut PoolConnection<Postgres>,
        filename: &str,
    ) -> Result<i32, eyre::Error> {
        tariff::get_by_name(connection, &filename)
            .await
            .map_err(|_e| {
                eyre::Error::msg(format!(
                    r#"Tariff for filename "{}" was not recognized"#,
                    filename
                ))
            })
    }
    async fn set_image_id(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        image_id: Option<i32>,
        id: i32,
    ) -> Result<(), sqlx::Error> {
        tariff::set_image(transaction, id, image_id).await
    }

    fn folder_parent(&self) -> &Path {
        self.folder_parent
    }
}

pub struct CpoFolder<'a> {
    folder_parent: &'a Path,
}

#[async_trait]
impl ImageImport for CpoFolder<'_> {
    fn new() -> Self {
        Self {
            folder_parent: CPOS_FOLDER.as_path(),
        }
    }

    async fn get_id_by_name(
        &self,
        connection: &mut PoolConnection<Postgres>,
        filename: &str,
    ) -> Result<i32, eyre::Error> {
        cpo::get_by_pub_id_or_name(connection, &filename)
            .await
            .ok_or_else(|| {
                eyre::Error::msg(format!(
                    r#"CPO for filename "{}" was not recognized"#,
                    filename
                ))
            })
    }
    async fn set_image_id(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        image_id: Option<i32>,
        id: i32,
    ) -> Result<(), sqlx::Error> {
        cpo::set_image(transaction, id, image_id).await
    }

    fn folder_parent(&self) -> &Path {
        self.folder_parent
    }
}
#[async_trait]
pub trait ImageImport {
    fn new() -> Self;
    async fn get_id_by_name(
        &self,
        connection: &mut PoolConnection<Postgres>,
        name: &str,
    ) -> Result<i32, eyre::Error>;
    async fn set_image_id(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        image_id: Option<i32>,
        id: i32,
    ) -> Result<(), sqlx::Error>;

    fn folder_parent(&self) -> &Path;
}

pub async fn import_folder<T>(state: &State, image_importer: T) -> Result<(), eyre::Error>
where
    T: ImageImport,
{
    let mut connection = state.database_pool.acquire().await?;
    let folder = image_importer.folder_parent();
    if !folder.exists() {
        tokio::fs::create_dir(folder)
            .await
            .with_context(|| format!("could not create folder: {}", folder.display()))?;
        tracing::info!("Creating folder {}", folder.to_string_lossy());
    }

    if !folder.is_dir() {
        return Err(eyre::Error::msg(format!(
            "{} is not a folder",
            folder.to_string_lossy()
        )));
    }

    let mut dir = tokio::fs::read_dir(folder).await?;
    let mut errors = vec![];
    while let Some(entry) = dir.next_entry().await? {
        let file = entry.file_name();
        let filename = file.to_str().unwrap_or_default();
        if !REGEX_FILENAME.is_match(&filename) {
            continue;
        }
        let path = &entry.path().canonicalize()?;

        if let Err(error) = insert_or_update(&mut connection, path, &image_importer).await {
            let message = format!("Ignoring image filename {filename}, error: {error}");
            tracing::warn!(message);
            errors.push(message);
        };
    }
    if !errors.is_empty() && cfg!(release_assertions) {
        let slack = &state.slack;

        slack
            .send(Some(MessageEmoji::Warning), &errors.join("\n"))
            .await;
    }

    tracing::info!("Image import done for folder: {} ", folder.display());
    Ok(())
}

async fn insert_or_update<T>(
    connection: &mut PoolConnection<Postgres>,
    new_path: &PathBuf,
    importer: &T,
) -> Result<(), eyre::Error>
where
    T: ImageImport,
{
    let raw_filename = new_path
        .file_name()
        .ok_or_else(|| eyre::Error::msg("Unsupported filename"))?
        .to_string_lossy();

    let mime = crate::io::guess_image_mime(new_path).await?;

    let filename = parse_filename(&raw_filename)?;

    let id = importer
        .get_id_by_name(connection, &filename)
        .await
        .map_err(|_e| {
            eyre::Error::msg(format!(
                r#"Tariff for filename "{}" was not recognized"#,
                filename
            ))
        })?;

    let checksum = hash_file(new_path).await?;
    let meta = fs::metadata(new_path).await?;

    tracing::debug!(
        msg = "Inserting new or update image",
        id,
        checksum=?checksum,
        new=?new_path.file_name().unwrap_or_default(),
        filename=?filename
    );

    let image_context = ImageContext {
        source_id: id,
        image: Image {
            file_path: new_path.clone(),
            checksum,
            mime,
        },
        filename,
        updated: meta.modified()?.into(),
    };

    let mut transaction = connection.begin().await?;

    let image_id = image::insert_or_update(&mut transaction, &image_context).await?;

    importer
        .set_image_id(&mut transaction, image_id, id)
        .await?;

    transaction.commit().await?;

    Ok(())
}

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

pub fn watch_cards_folder(state: State) -> Result<(), eyre::Error> {
    cleanup_task(state.clone());
    tokio::task::spawn_blocking(move || {
        let folder = CARDS_FOLDER.as_path();
        let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize");
        tracing::info!(
            "Start watching {} folder for watching",
            &folder.to_string_lossy()
        );
        hotwatch
            .watch(&folder, move |event: Event| {
                let state = state.clone();
                tokio::task::spawn(async move {
                    let slack = &state.slack;
                    let ret = handle_card_fs_event(event, &slack, &state.database_pool).await;
                    if let Err(err) = ret {
                        // TODO error pretty print
                        tracing::warn!(msg = "While watching the folder", err = ?err);
                        let text = format!("{} Something went wrong:\n{}", slack::MALIK, err);
                        slack.send(Some(MessageEmoji::Warning), &text).await;
                    }
                });
                Flow::Continue
            })
            .expect(&format!("failed to watch path {:#?}", folder));
        hotwatch.run();
    });

    Ok(())
}

async fn handle_card_fs_event(
    event: Event,
    slack: &Option<Slack>,
    database_pool: &Pool<Postgres>,
) -> Result<(), eyre::Error> {
    match event {
        Event::Write(path) | Event::Create(path) if io::is_file(&path).await? => {
            let mut connection = database_pool.acquire().await?;
            tracing::info!(event = "Event::Create|Write", ?path);

            match detect_rename(&mut connection, &path).await {
                Some(old_path) => {
                    tracing::info!(msg = "File is already known. It will be renamed", old=?old_path, new=?path);
                    rename_path(&mut connection, &old_path, &path, slack).await?;
                }
                None => {
                    insert_or_update(&mut connection, &path, &CardFolder::new()).await?;
                    let msg = &format!(
                        "New card image filename: {:#?}",
                        path.file_name().unwrap_or_default()
                    );
                    tracing::info!(event = "Event::Create|Write", %msg);
                    slack.send(Some(MessageEmoji::ImageFrame), msg).await
                }
            }
        }
        Event::Rename(old_path, new_path) if io::is_file(&new_path).await? => {
            tracing::info!(event = "Event::Rename", old=?old_path, new=?new_path);
            let mut connection = database_pool.acquire().await?;
            rename_path(&mut connection, &old_path, &new_path, slack).await?;
        }
        Event::Remove(path) => {
            tracing::info!(event = "Event::Remove", ?path);
            let mut connection = database_pool.acquire().await?;
            image::soft_delete(&mut connection, &path).await?
        }
        Event::Error(error, path) => {
            slack
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

async fn rename_path(
    connection: &mut PoolConnection<Postgres>,
    old_path: &PathBuf,
    new_path: &PathBuf,
    slack: &Option<Slack>,
) -> Result<(), eyre::Error> {
    // todo check if path is  an image
    let raw_filename = new_path
        .file_name()
        .ok_or_else(|| eyre::Error::msg("Unsupported filename"))?
        .to_string_lossy();

    io::guess_image_mime(new_path).await?;

    let filename = parse_filename(&raw_filename)?;
    tracing::info!(
        msg = "Updating path",
        old=?old_path,
        new=?new_path,
        filename=?filename
    );

    image::update_name_path(connection, old_path, new_path, &filename).await?;

    slack
        .send(
            Some(MessageEmoji::Rename),
            &format!(
                "Renamed card image\nold name: {:#?}, new name {:#?}",
                old_path.file_name().unwrap_or_default(),
                new_path.file_name().unwrap_or_default()
            ),
        )
        .await;
    Ok(())
}

fn parse_filename(name: &str) -> Result<String, eyre::Error> {
    let captures = REGEX_FILENAME.captures(name).and_then(|c| c.get(1));

    match captures {
        Some(group) => Ok(group.as_str().to_owned()),
        None => Err(eyre::Error::msg(format!(
            "Wrong formatted filename: {}",
            name
        ))),
    }
}

async fn hash_file(file: &PathBuf) -> Result<String, std::io::Error> {
    let bytes = tokio::fs::read(file).await?;
    let hash = blake3::hash(&bytes).to_hex().to_string();
    Ok(hash)
}

async fn detect_rename(
    connection: &mut PoolConnection<Postgres>,
    path: &PathBuf,
) -> Option<PathBuf> {
    let checksum = hash_file(path).await.ok()?;

    let card_image = image::get_by_checksum(connection, &checksum).await.ok();
    card_image.map(|card| card.file_path)
}
