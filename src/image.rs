use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use eyre::Context;
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use once_cell::sync::Lazy;
use sqlx::{pool::PoolConnection, Acquire, Postgres};
use tree_magic_mini::match_filepath;

use crate::{
    db::{self, card_image::CardImage, tarif},
    state::{self, State},
};

static REGEX_FILENAME: Lazy<regex::Regex> = Lazy::new(|| {
    regex::RegexBuilder::new(r#"^(?:card_)*([a-zA-Z0-9-ß]+)\.(?:jpg|jpeg|png|svg|gif)$"#)
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub fn watch_folder(folder: PathBuf, state: State) -> Result<(), eyre::Error> {
    tokio::task::spawn_blocking(move || {
        let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize");
        hotwatch
            .watch(folder.clone(), move |event: Event| {
                let state = state.clone();
                tokio::task::spawn(async move {
                    let ret = handle_fs_event(event, state).await;
                    if let Err(err) = ret {
                        // TODO error pretty print
                        tracing::warn!(msg = "While watching the folder", err = ?err)
                    }
                });
                Flow::Continue
            })
            .expect(&format!("failed to watch path {:#?}", folder));
        hotwatch.run();
    });
    Ok(())
}
async fn handle_fs_event(event: Event, state: State) -> Result<(), eyre::Error> {
    match event {
        Event::Write(path) | Event::Create(path) => {
            tracing::info!(event = "Event::Write|Write", new=?path);
            let mut connection = state.database_pool.acquire().await?;
            insert_or_update(&mut connection, &path).await?;
        }
        Event::Rename(old_path, new_path) => {
            tracing::info!(event = "Event::Rename", old=?old_path, new=?new_path);
            let mut connection = state.database_pool.acquire().await?;
            update_path(&mut connection, &old_path, &new_path).await?;
        }
        // TODO deal with remove images
        Event::Remove(path) => {
            tracing::info!(event = "Event::Remove", path=?path);
            let mut connection = state.database_pool.acquire().await?;
            delete(&mut connection, &path).await?;
        }
        Event::Error(error, path) => {
            tracing::error!(
                "Error::Event {}, path: {}",
                error,
                path.unwrap_or_default().to_string_lossy()
            );
        }
        _ => {
            // tracing::info!("Unknown event");
        }
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
) -> Result<(), eyre::Error> {
    let raw_filename = new_path
        .file_name()
        .ok_or_else(|| eyre::Error::msg("Unsupported filename"))?
        .to_string_lossy();

    validate_from_guessed_mime(new_path)?;

    let filename = parse_filename(&raw_filename)?;
    tracing::info!(
        msg = "Updating path only",
        old=?old_path,
        new=?new_path,
        filename=?filename
    );

    db::card_image::update_path(connection, old_path, new_path, &filename).await?;
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

    validate_from_guessed_mime(new_path)?;

    let filename = parse_filename(&raw_filename)?;
    // transaction.

    let tarif_id = tarif::get_by_name(connection, &filename)
        .await
        .map_err(|_e| eyre::Error::msg(format!("tarif for filename {} was not found", filename)))?;
    tracing::info!(tarif_id = tarif_id);

    let checksum = hash_file(new_path).await?;

    tracing::info!(
        msg = "Inserting new image",
        checksum=?checksum,
        new=?new_path,
        filename=?filename
    );

    let checksum = hash_file(&new_path).await?;

    let card_image = CardImage {
        tarif_id,
        path: new_path.as_path(),
        checksum,
        filename,
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

fn validate_from_guessed_mime<P: AsRef<Path>>(path: P) -> Result<(), eyre::Error> {
    let path = path.as_ref();
    let mime_types = [
        mime::IMAGE_JPEG,
        mime::IMAGE_PNG,
        mime::IMAGE_SVG,
        mime::IMAGE_GIF,
    ];
    let guess_mime = tree_magic_mini::from_filepath(path);
    if let Some(mime) = guess_mime {
        for valid_mime in mime_types {
            if mime == valid_mime {
                return Ok(());
            }
        }
    }

    Err(eyre::Error::msg(format!(
        "Unsupported file type path: {}, type: {:#?}",
        path.to_string_lossy(),
        guess_mime
    )))
}
