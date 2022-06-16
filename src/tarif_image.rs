use std::path::{Path, PathBuf};

use axum::body::StreamBody;
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use once_cell::sync::Lazy;
use sqlx::{pool::PoolConnection, Pool, Postgres};
use tokio_util::io::ReaderStream;

use crate::{
    db::{
        self,
        card_image::{CardImage, CardImageContext},
        tarif,
    },
    slack::{MessageEmoji, Slack},
    state::State,
};

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
        tracing::info!("Creating folder {}", folder.to_string_lossy(),);
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
                let message = format!("Ignoring image filename {}, error: {}", filename, error);
                tracing::warn!("{}", message);
                errors.push(message);
            };
        }
    }
    if !errors.is_empty() && cfg!(release_assertions) {
        state
            .slack
            .send(Some(MessageEmoji::Warning), &errors.join("\n"))
            .await;
    }

    tracing::info!("Image import is done");
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
                        let text = format!("<@U028N463G1J> something went wrong:\t{}", err);
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
    slack: &Slack,
    database_pool: &Pool<Postgres>,
) -> Result<(), eyre::Error> {
    match event {
        Event::Write(path) | Event::Create(path) => {
            tracing::info!(event = "Event::Create|Write", new=?path);
            let mut connection = database_pool.acquire().await?;
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
        Event::Rename(old_path, new_path) => {
            tracing::info!(event = "Event::Rename", old=?old_path, new=?new_path);
            let mut connection = database_pool.acquire().await?;
            update_path(&mut connection, &old_path, &new_path).await?;
            // TODO maybe too much spam?!
            // slack
            //     .send(
            //         None,
            //         &format!(
            //             "Renamed card image\n old: {:#?},\tnew: {:#?}",
            //             old_path.file_name().unwrap_or_default(),
            //             new_path.file_name().unwrap_or_default()
            //         ),
            //     )
            //     .await;
        }
        Event::Remove(path) => {
            tracing::info!(event = "Event::Remove", path=?path);
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
    // todo cehck if path is  an image
    db::card_image::delete(connection, path).await?;
    Ok(())
}

async fn update_path(
    connection: &mut PoolConnection<Postgres>,
    old_path: &PathBuf,
    new_path: &PathBuf,
) -> Result<(), eyre::Error> {
    // todo check if path is  an image
    let raw_filename = new_path
        .file_name()
        .ok_or_else(|| eyre::Error::msg("Unsupported filename"))?
        .to_string_lossy();

    guess_mime(new_path).await?;

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

    let mime = guess_mime(new_path).await?;

    let filename = parse_filename(&raw_filename)?;

    let tarif_id = tarif::get_by_name(connection, &filename)
        .await
        .map_err(|_e| {
            eyre::Error::msg(format!(
                r#"Tariff for filename "{}" was not recognized"#,
                filename
            ))
        })?;

    let checksum = hash_file(new_path).await?;

    tracing::info!(
        msg = "Inserting new image",
        tarif_id=tarif_id,
        checksum=?checksum,
        new=?new_path.file_name().unwrap_or_default(),
        filename=?filename
    );

    let checksum = hash_file(&new_path).await?;

    let card_image = CardImageContext {
        tarif_id,
        image: CardImage {
            file_path: new_path.clone(),
            checksum,
            mime,
        },
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

async fn guess_mime<P: AsRef<Path>>(path: P) -> Result<mime::Mime, eyre::Error> {
    let path = path.as_ref();
    let mime_types = [
        mime::IMAGE_JPEG,
        mime::IMAGE_PNG,
        mime::IMAGE_SVG,
        mime::IMAGE_GIF,
    ];
    let bytes = read_bytes(path, 2048).await?;
    let guess_mime = tree_magic::from_u8(&bytes);
    for valid_mime in mime_types {
        if guess_mime.as_str() == valid_mime {
            return Ok(valid_mime);
        }
    }

    Err(eyre::Error::msg(format!(
        "Unsupported file type path: {}, type: {:#?}",
        path.to_string_lossy(),
        guess_mime
    )))
}

async fn read_bytes(filepath: &Path, byte_count: usize) -> Result<Vec<u8>, std::io::Error> {
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;

    let mut bytes = Vec::<u8>::with_capacity(byte_count);
    let file = File::open(filepath).await?;
    file.take(byte_count as u64).read_to_end(&mut bytes).await?;
    Ok(bytes)
}

pub type FileStream = StreamBody<ReaderStream<tokio::fs::File>>;

pub async fn read_file<P: AsRef<Path>>(path: P) -> Result<FileStream, tokio::io::Error> {
    let file = tokio::fs::File::open(path).await?;
    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);
    Ok(body)
}
