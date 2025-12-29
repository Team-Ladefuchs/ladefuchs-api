use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tariff {
    pub id: i32,
    pub relationship_id: uuid::Uuid,
    pub pub_tariff_id: uuid::Uuid,
    pub slug_name: String,
    pub monthly_fee: f64,
    pub note: String,
    pub standard: bool,
    pub hide: bool,
    pub provider_name: String,
    pub provider_id: uuid::Uuid,
    pub url: Option<String>,
    pub image: Option<i32>,
    pub updated: DateTime<Utc>,
    pub provider_customer_only: Option<bool>,
    pub ad_hoc: Option<bool>,
    pub brand_only: Option<bool>,
}

pub struct TariffBuilder {
    relationship_id: Option<uuid::Uuid>,
    pub_tariff_id: Option<uuid::Uuid>,
    slug_name: Option<String>,
    monthly_fee: f64,
    note: String,
    standard: bool,
    hide: bool,
    provider_name: Option<String>,
    provider_id: Option<uuid::Uuid>,
    url: Option<String>,
    image: Option<i32>,
    internal_name: Option<String>,
    updated: DateTime<Utc>,
    provider_customer_only: Option<bool>,
    ad_hoc: Option<bool>,
    brand_only: Option<bool>,
}

impl Default for TariffBuilder {
    fn default() -> Self {
        Self {
            relationship_id: None,
            pub_tariff_id: None,
            slug_name: None,
            monthly_fee: 0.0,
            note: "".to_owned(),
            standard: true,
            hide: false,
            provider_name: None,
            provider_id: None,
            url: None,
            image: None,
            internal_name: None,
            updated: Utc::now(),
            provider_customer_only: Some(false),
            ad_hoc: Some(false),
            brand_only: Some(false),
        }
    }
}

impl TariffBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn relationship_id(mut self, relationship_id: uuid::Uuid) -> Self {
        self.relationship_id = Some(relationship_id);
        self
    }

    pub fn pub_tariff_id(mut self, pub_tariff_id: uuid::Uuid) -> Self {
        self.pub_tariff_id = Some(pub_tariff_id);
        self
    }

    pub fn slug_name(mut self, slug_name: impl Into<String>) -> Self {
        self.slug_name = Some(slug_name.into());
        self
    }

    pub fn monthly_fee(mut self, monthly_fee: f64) -> Self {
        self.monthly_fee = monthly_fee;
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    pub fn standard(mut self, standard: bool) -> Self {
        self.standard = standard;
        self
    }

    pub fn hide(mut self, hide: bool) -> Self {
        self.hide = hide;
        self
    }

    pub fn provider_name(mut self, provider_name: impl Into<String>) -> Self {
        self.provider_name = Some(provider_name.into());
        self
    }

    pub fn provider_id(mut self, provider_id: uuid::Uuid) -> Self {
        self.provider_id = Some(provider_id);
        self
    }

    pub fn url(mut self, url: impl Into<Option<String>>) -> Self {
        self.url = url.into();
        self
    }

    pub fn image(mut self, image: Option<i32>) -> Self {
        self.image = image;
        self
    }

    pub fn internal_name(mut self, internal_name: impl Into<String>) -> Self {
        self.internal_name = Some(internal_name.into());
        self
    }

    pub fn updated(mut self, updated: DateTime<Utc>) -> Self {
        self.updated = updated;
        self
    }

    pub fn provider_customer_only(mut self, provider_customer_only: bool) -> Self {
        self.provider_customer_only = Some(provider_customer_only);
        self
    }

    pub fn ad_hoc(mut self, ad_hoc: bool) -> Self {
        self.ad_hoc = Some(ad_hoc);
        self
    }

    pub fn brand_only(mut self, brand_only: bool) -> Self {
        self.brand_only = Some(brand_only);
        self
    }

    pub async fn create(self, pool: &PgPool) -> Tariff {
        static NAME_SEQUENCE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        static SLUG_SEQUENCE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        static INTERNAL_NAME_SEQUENCE: std::sync::atomic::AtomicU32 =
            std::sync::atomic::AtomicU32::new(0);

        let relationship_id = self.relationship_id.unwrap_or_else(uuid::Uuid::new_v4);
        let pub_tariff_id = self.pub_tariff_id.unwrap_or_else(uuid::Uuid::new_v4);
        let provider_id = self.provider_id.unwrap_or_else(uuid::Uuid::new_v4);

        let provider_name = self.provider_name.unwrap_or_else(|| {
            let seq = NAME_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("Test Provider {}", seq)
        });

        let slug_name = self.slug_name.unwrap_or_else(|| {
            let seq = SLUG_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("Test Tariff {}", seq)
        });

        let internal_name = self.internal_name.unwrap_or_else(|| {
            let seq = INTERNAL_NAME_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!("internal-tariff-{}", seq)
        });

        sqlx::query_as(
            r#"
            INSERT INTO tariff (
            	relationship_id,
                pub_tariff_id,
                slug_name,
                monthly_fee,
                note,
                standard,
                hide,
                provider_name,
                provider_id,
                url,
                image,
                internal_name,
                updated,
                provider_customer_only,
                ad_hoc,
                brand_only
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, COALESCE($12, ''), $13,
                COALESCE($14, false),
                COALESCE($15, false),
                COALESCE($16, false)
            )
            RETURNING
                id,
                relationship_id,
                pub_tariff_id,
                slug_name,
                monthly_fee,
                note,
                standard,
                hide,
                provider_name,
                provider_id,
                url,
                image,
                updated,
                provider_customer_only,
                ad_hoc,
                brand_only
            "#,
        )
        .bind(relationship_id)
        .bind(pub_tariff_id)
        .bind(slug_name)
        .bind(self.monthly_fee)
        .bind(self.note)
        .bind(self.standard)
        .bind(self.hide)
        .bind(provider_name)
        .bind(provider_id)
        .bind(self.url)
        .bind(self.image)
        .bind(internal_name)
        .bind(self.updated)
        .bind(self.provider_customer_only)
        .bind(self.ad_hoc)
        .bind(self.brand_only)
        .fetch_one(pool)
        .await
        .expect("could not insert tariff fixture")
    }
}
