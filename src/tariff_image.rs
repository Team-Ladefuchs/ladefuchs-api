use std::path::PathBuf;

use crate::{
    db::{
        self,
        card_image::{CardImage, CardImageContext},
        tariff,
    },
    io,
    slack::{self, MessageEmoji, Slack, SlackClient},
    state::State,
};

use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use once_cell::sync::Lazy;
use sqlx::{pool::PoolConnection, Pool, Postgres};
use tokio::fs;

static REGEX_FILENAME: Lazy<regex::Regex> = Lazy::new(|| {
    regex::RegexBuilder::new(r#"^(?:card_)*([a-zA-Z0-9-_ß]+)\.(?:jpg|jpeg|png|svg|gif)$"#)
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub async fn import_folder(state: &State) -> Result<(), eyre::Error> {
    let mut connection = state.database_pool.acquire().await?;
    let folder = &state.config.image_folder;
    if !folder.exists() {
        tokio::fs::create_dir(folder).await?;
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
        if REGEX_FILENAME.is_match(&filename) {
            let path = &entry.path().canonicalize()?;
            // TODO Maybe do no import every image into a separate transaction?

            if let Err(error) = insert_or_update(&mut connection, path).await {
                let message = format!("Ignoring image filename {filename}, error: {error}");
                tracing::warn!("{message}");
                errors.push(message);
            };
        }
    }
    if !errors.is_empty() && cfg!(release_assertions) {
        let slack = &state.slack;

        slack
            .send(Some(MessageEmoji::Warning), &errors.join("\n"))
            .await;
    }

    tracing::info!("Image import has finished");
    Ok(())
}

pub fn watch_folder(state: State) -> Result<(), eyre::Error> {
    tokio::task::spawn_blocking(move || {
        let folder = state.config.image_folder.clone();
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
                    let ret = handle_fs_event(event, &slack, &state.database_pool).await;
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
async fn handle_fs_event(
    event: Event,
    slack: &Option<Slack>,
    database_pool: &Pool<Postgres>,
) -> Result<(), eyre::Error> {
    match event {
        Event::Write(path) | Event::Create(path) | Event::Chmod(path) => {
            if !io::is_file(&path).await? {
                return Ok(());
            }
            let mut connection = database_pool.acquire().await?;
            tracing::info!(event = "Event::Create|Write", new=?path);
            match detect_rename(&mut connection, &path).await {
                Some(old_path) => {
                    tracing::info!(msg = "File is already known. It will be renamed", old=?old_path, new=?path);
                    update_path(&mut connection, &old_path, &path, slack).await?;
                }
                None => {
                    insert_or_update(&mut connection, &path).await?;
                    slack
                        .send(
                            Some(MessageEmoji::ImageFrame),
                            &format!(
                                "New card image was added\n path: {:#?},\tfilename: {:#?}",
                                path,
                                path.file_name().unwrap_or_default()
                            ),
                        )
                        .await
                }
            }
        }
        Event::Rename(old_path, new_path) => {
            if !io::is_file(&new_path).await? {
                return Ok(());
            }
            tracing::info!(event = "Event::Rename", old=?old_path, new=?new_path);
            let mut connection = database_pool.acquire().await?;
            update_path(&mut connection, &old_path, &new_path, slack).await?;
            slack
                .send(
                    None,
                    &format!(
                        "Renamed card image\n old path: {:#?}, new path {:#?}",
                        old_path, new_path
                    ),
                )
                .await;
        }

        Event::Remove(path) => {
            tracing::info!(event = "Event::Remove", path=?path);
            // TODO check if file exists in db??
            let mut connection = database_pool.acquire().await?;
            delete(&mut connection, &path).await?;
        }
        Event::Error(error, path) => {
            slack
                .send(
                    Some(MessageEmoji::Error),
                    &format!("An Error has occurred: {:#?},\tnew path {:#?}", error, path),
                )
                .await;
            tracing::error!("Error::Event {}, path: {:#?}", error, path);
        }
        _ => {}
    }
    Ok(())
}

async fn delete(
    connection: &mut PoolConnection<Postgres>,
    path: &PathBuf,
) -> Result<(), sqlx::Error> {
    db::card_image::delete(connection, path).await?;
    Ok(())
}

async fn update_path(
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
        msg = "Updating only path",
        old=?old_path,
        new=?new_path,
        filename=?filename
    );

    db::card_image::update_name_path(connection, old_path, new_path, &filename).await?;

    slack
        .send(
            None,
            &format!(
                "Renamed card image\n old path: {:#?}, new path {:#?}",
                old_path, new_path
            ),
        )
        .await;
    Ok(())
}

async fn insert_or_update(
    connection: &mut PoolConnection<Postgres>,
    new_path: &PathBuf,
) -> Result<(), eyre::Error> {
    let raw_filename = new_path
        .file_name()
        .ok_or_else(|| eyre::Error::msg("Unsupported filename"))?
        .to_string_lossy();

    let mime = crate::io::guess_image_mime(new_path).await?;

    let filename = parse_filename(&raw_filename)?;

    let tariff_id = tariff::get_by_name(connection, &filename)
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
        tariff_id=tariff_id,
        checksum=?checksum,
        new=?new_path.file_name().unwrap_or_default(),
        filename=?filename
    );

    let checksum = hash_file(&new_path).await?;

    let card_image = CardImageContext {
        tariff_id,
        image: CardImage {
            file_path: new_path.clone(),
            checksum,
            mime,
        },
        filename,
        updated: meta.modified()?.into(),
    };

    db::card_image::insert_or_update(connection, &card_image).await?;

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
    let card_image = db::card_image::get_by_checksum(connection, &checksum)
        .await
        .ok();

    card_image.map(|card| card.file_path)
}
