use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    db::{
        banner,
        image::{self, Image, ImageContext},
        operator, tariff,
    },
    file_watcher::{parse_filename, to_relative_image_path, REGEX_IMAGE_FILENAME},
    io::hash_file,
    slack::{Emoji, SlackClient},
    state::State,
};

use axum::async_trait;
use eyre::Context;

use sqlx::{Connection, PgConnection};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct CardFolder {
    folder_parent: Arc<PathBuf>,
}

#[async_trait]
impl ImageFolder for CardFolder {
    fn new() -> Self {
        Self {
            folder_parent: Arc::new(PathBuf::from("./images/cards")),
        }
    }

    fn id(&self) -> ImageMetaFolder {
        ImageMetaFolder {
            prefix: "card",
            emoji: Emoji::ImageFrame,
        }
    }

    async fn get_id_by_name(
        &self,
        connection: &mut PgConnection,
        filename: &str,
    ) -> Result<i32, sqlx::Error> {
        tariff::get_by_name(connection, &filename).await
    }
    async fn set_image_id(
        &self,
        transaction: &mut PgConnection,
        image_id: Option<i32>,
        id: i32,
    ) -> Result<(), sqlx::Error> {
        tariff::admin::set_image(transaction, id, image_id).await
    }

    fn folder_parent(&self) -> &Path {
        self.folder_parent.as_path()
    }

    async fn set_internal_name(
        &self,
        transaction: &mut PgConnection,
        tariff_id: i32,
        name: &str,
    ) -> Result<(), sqlx::Error> {
        tariff::admin::set_internal_name(transaction, tariff_id, name).await
    }
}
#[derive(Debug, Clone)]
pub struct OperatorFolder {
    folder_parent: Arc<PathBuf>,
}

#[async_trait]
impl ImageFolder for OperatorFolder {
    fn new() -> Self {
        Self {
            folder_parent: Arc::new(PathBuf::from("./images/cpos")),
        }
    }

    fn id(&self) -> ImageMetaFolder {
        ImageMetaFolder {
            prefix: "CPO",
            emoji: Emoji::ElectricPlug,
        }
    }

    async fn get_id_by_name(
        &self,
        connection: &mut PgConnection,
        filename: &str,
    ) -> Result<i32, sqlx::Error> {
        operator::get_by_pub_id_or_name(connection, &filename).await
    }

    async fn set_image_id(
        &self,
        transaction: &mut PgConnection,
        image_id: Option<i32>,
        id: i32,
    ) -> Result<(), sqlx::Error> {
        operator::set_image(transaction, id, image_id).await
    }

    async fn set_internal_name(
        &self,
        _transaction: &mut PgConnection,
        _id: i32,
        _name: &str,
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }

    fn folder_parent(&self) -> &Path {
        self.folder_parent.as_path()
    }
}

#[derive(Debug, Clone)]
pub struct BannerFolder {
    folder_parent: Arc<PathBuf>,
}

#[async_trait]
impl ImageFolder for BannerFolder {
    fn new() -> Self {
        Self {
            folder_parent: Arc::new(PathBuf::from("./images/banners")),
        }
    }

    async fn get_id_by_name(
        &self,
        connection: &mut PgConnection,
        filename: &str,
    ) -> Result<i32, sqlx::Error> {
        let id = banner::get_id_by_name(connection, filename).await?;
        Ok(id)
    }

    async fn set_image_id(
        &self,
        transaction: &mut PgConnection,
        image_id: Option<i32>,
        banner_id: i32,
    ) -> Result<(), sqlx::Error> {
        banner::set_image(transaction, banner_id, image_id).await
    }

    async fn set_internal_name(
        &self,
        _transaction: &mut PgConnection,
        _id: i32,
        _name: &str,
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }

    fn folder_parent(&self) -> &Path {
        self.folder_parent.as_path()
    }

    fn id(&self) -> ImageMetaFolder {
        ImageMetaFolder {
            prefix: "Banner",
            emoji: Emoji::Art,
        }
    }
}

#[derive(Debug)]
pub struct ImageMetaFolder {
    pub prefix: &'static str,
    pub emoji: Emoji,
}

#[async_trait]
pub trait ImageFolder: Send + Sync + 'static + Clone {
    fn new() -> Self;
    async fn get_id_by_name(
        &self,
        connection: &mut PgConnection,
        name: &str,
    ) -> Result<i32, sqlx::Error>;
    async fn set_image_id(
        &self,
        transaction: &mut PgConnection,
        image_id: Option<i32>,
        id: i32,
    ) -> Result<(), sqlx::Error>;

    async fn set_internal_name(
        &self,
        transaction: &mut PgConnection,
        id: i32,
        name: &str,
    ) -> Result<(), sqlx::Error>;

    fn not_recognized_error(&self, filename: &str, path: &Path) -> eyre::Report {
        eyre::Error::msg(format!(
            r#"[type: {}, path: {}, filename: {}] The provided file was not recognized. Maybe check the internal name or ask dominic."#,
            self.id().prefix,
            path.display(),
            filename
        ))
    }

    fn folder_parent(&self) -> &Path;

    fn id(&self) -> ImageMetaFolder;
}

pub async fn import_folder<T>(state: &State, image_importer: &T) -> Result<(), eyre::Error>
where
    T: ImageFolder,
{
    let mut connection = state.database_pool.acquire().await?;
    let folder = image_importer.folder_parent();
    if !folder.exists() {
        tokio::fs::create_dir(folder).await.with_context(|| {
            format!(
                "while importing: could not create folder: {}",
                folder.display()
            )
        })?;
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
    while let Some(path) = dir
        .next_entry()
        .await?
        .and_then(|entry| to_relative_image_path(entry.path()))
    {
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if !REGEX_IMAGE_FILENAME.is_match(&filename) {
            continue;
        }

        if let Some(path) = to_relative_image_path(path) {
            if let Err(error) = insert_or_update(&mut connection, &path, image_importer).await {
                let message = format!("Ignoring image filename {filename}, error: {error}");
                tracing::warn!(message);
                errors.push(message);
            };
        }
    }
    if !errors.is_empty() && cfg!(release_assertions) {
        let slack = &state.slack;

        slack
            .send_message(crate::slack::TextMessage {
                emoji: Some(Emoji::Warning),
                text: errors.join("\n"),
            })
            .await;
    }
    tracing::info!("Image import done for folder: {} ", folder.display());
    Ok(())
}

pub async fn insert_or_update<T>(
    connection: &mut PgConnection,
    new_path: &PathBuf,
    importer: &T,
) -> Result<Option<i32>, eyre::Error>
where
    T: ImageFolder,
{
    let mime = crate::io::guess_image_mime(new_path).await?;

    let filename = parse_filename(&new_path)?;

    let id = importer
        .get_id_by_name(connection, &filename)
        .await
        .map_err(|_e| importer.not_recognized_error(&filename, new_path))?;

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

    Ok(image_id)
}
