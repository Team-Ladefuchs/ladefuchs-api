use chrono::{DateTime, Utc};

use crate::{
    api::banner::v3::Banner,
    fixtures::{
        image::ImageBuilder,
        link::{Link, LinkBuilder},
    },
};

#[derive(Clone, Debug)]
pub struct BannerBuilder {
    pub_id: Option<uuid::Uuid>,
    link_id: Option<i32>,
    updated: DateTime<Utc>,
    frequency: i16,
    expiration: DateTime<Utc>,
    starts: DateTime<Utc>,
    name: Option<String>,
    image: Option<i32>,
    impression: i32,
    status: String,
}

impl Default for BannerBuilder {
    fn default() -> Self {
        let now = Utc::now();

        Self {
            pub_id: None,
            link_id: None,
            updated: now,
            frequency: 1,
            expiration: "2030-12-31 00:00:00+00:00".parse().unwrap(),
            starts: now,
            name: None,
            image: None,
            impression: 0,
            status: "active".to_owned(),
        }
    }
}

impl BannerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pub_id(mut self, pub_id: uuid::Uuid) -> Self {
        self.pub_id = Some(pub_id);
        self
    }

    pub fn link_id(mut self, link_id: i32) -> Self {
        self.link_id = Some(link_id);
        self
    }

    pub fn updated(mut self, updated: DateTime<Utc>) -> Self {
        self.updated = updated;
        self
    }

    pub fn frequency(mut self, frequency: i16) -> Self {
        self.frequency = frequency;
        self
    }

    pub fn expiration(mut self, expiration: DateTime<Utc>) -> Self {
        self.expiration = expiration;
        self
    }

    pub fn starts(mut self, starts: DateTime<Utc>) -> Self {
        self.starts = starts;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn image(mut self, image: i32) -> Self {
        self.image = Some(image);
        self
    }

    pub fn impression(mut self, impression: i32) -> Self {
        self.impression = impression;
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub async fn create(self, pool: &sqlx::PgPool) -> Banner {
        static NAME_SEQUENCE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            pub_id: uuid::Uuid,
            link_id: i32,
            updated: DateTime<Utc>,
            frequency: i16,
        }

        let link_id = if let Some(link_id) = self.link_id {
            link_id
        } else {
            LinkBuilder::new().create(pool).await.id
        };

        let image_id = if let Some(image) = self.image {
            image
        } else {
            ImageBuilder::new().create(pool).await.id
        };

        let name = if let Some(name) = self.name {
            name
        } else {
            format!(
                "Test Banner {}",
                NAME_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            )
        };

        let banner: Row = sqlx::query_as(
            r#"
              INSERT INTO link_banner
                (pub_id, link_id, updated, frequency, expiration, starts, name, image, impression, status)
            VALUES
                 (COALESCE($1, gen_random_uuid()), $2, $3, $4, $5, $6, $7, $8, $9, $10::link_banner_status_type)
             RETURNING id, pub_id, link_id, updated, frequency
            "#,
        )
        .bind(self.pub_id)
        .bind(link_id)
        .bind(self.updated)
        .bind(self.frequency)
        .bind(self.expiration)
        .bind(self.starts)
        .bind(name)
        .bind(image_id)
        .bind(self.impression)
        .bind(self.status)
        .fetch_one(pool)
        .await
        .unwrap();

        let link: Link = sqlx::query_as("SELECT * FROM link WHERE id = $1")
            .bind(banner.link_id)
            .fetch_one(pool)
            .await
            .unwrap();

        // TODO: image url

        Banner {
            affiliate_link_url: link.source.parse().unwrap(),
            identifier: banner.pub_id,
            image_url: format!("http://example.org/image/{}.png", banner.id)
                .parse()
                .unwrap(),
            frequency: banner.frequency,
            is_affiliate: true,
            last_updated_date: banner.updated,
        }
    }
}
