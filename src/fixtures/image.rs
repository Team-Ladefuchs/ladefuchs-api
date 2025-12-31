use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Image {
    pub id: i32,
    pub file_path: String,
    pub checksum: String,
    pub mime_type: String,
    pub updated: DateTime<Utc>,
    pub soft_delete: bool,
    pub is_ad_hoc: bool,
}

#[derive(Clone, Debug)]
pub struct ImageBuilder {
    file_path: Option<String>,
    checksum: Option<String>,
    mime_type: Option<String>,
    last_updated_date: DateTime<Utc>,
    soft_delete: bool,
    is_ad_hoc: bool,
}

impl Default for ImageBuilder {
    fn default() -> Self {
        Self {
            file_path: None,
            checksum: None,
            mime_type: None,
            last_updated_date: Utc::now(),
            soft_delete: false,
            is_ad_hoc: false,
        }
    }
}

impl ImageBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn file_path(mut self, file_path: impl Into<String>) -> Self {
        self.file_path = Some(file_path.into());
        self
    }

    pub fn checksum(mut self, checksum: impl Into<String>) -> Self {
        self.checksum = Some(checksum.into());
        self
    }

    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    pub fn last_updated_date(mut self, last_updated_date: DateTime<Utc>) -> Self {
        self.last_updated_date = last_updated_date;
        self
    }

    pub fn soft_delete(mut self, soft_delete: bool) -> Self {
        self.soft_delete = soft_delete;
        self
    }

    pub fn is_ad_hoc(mut self, is_ad_hoc: bool) -> Self {
        self.is_ad_hoc = is_ad_hoc;
        self
    }

    pub async fn create(self, pool: &sqlx::PgPool) -> Image {
        static PATH_SEQUENCE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        static HEX_SEQUENCE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

        let file_path = self.file_path.unwrap_or_else(|| {
            let seq = PATH_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("/tmp/fixture-image-{}.jpg", seq)
        });

        let checksum = self.checksum.unwrap_or_else(|| {
            let seq = HEX_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            // 64 hex chars
            format!("{:064x}", seq)
        });

        let mime_type = self.mime_type.unwrap_or_else(|| "image/jpeg".to_owned());

        sqlx::query_as(
            r#"
            INSERT INTO image (file_path, checksum, mime_type, updated, soft_delete, is_ad_hoc)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(file_path)
        .bind(checksum)
        .bind(mime_type)
        .bind(self.last_updated_date)
        .bind(self.soft_delete)
        .bind(self.is_ad_hoc)
        .fetch_one(pool)
        .await
        .unwrap()
    }
}
