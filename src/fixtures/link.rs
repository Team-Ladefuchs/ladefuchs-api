#[derive(sqlx::FromRow, Debug)]
pub struct Link {
    pub id: i32,
    pub is_affiliate: bool,
    pub source: String,
}

#[derive(Clone, Debug, Default)]
pub struct LinkBuilder {
    is_affiliate: bool,
    source: Option<String>,
}

impl LinkBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn is_affiliate(mut self, is_affiliate: bool) -> Self {
        self.is_affiliate = is_affiliate;
        self
    }

    pub async fn create(self, pool: &sqlx::PgPool) -> Link {
        let source = self
            .source
            .unwrap_or_else(|| "https://example.com".to_owned());

        sqlx::query_as(
            r#"
      	    INSERT INTO link (is_affiliate, source)
            VALUES ($1, $2)
            RETURNING *
            "#,
        )
        .bind(self.is_affiliate)
        .bind(source.to_string())
        .fetch_one(pool)
        .await
        .unwrap()
    }
}
