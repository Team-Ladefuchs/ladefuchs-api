pub mod v1 {
    use serde::{Deserialize, Serialize};

    use crate::db::tariff::Provider;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]

    pub struct Tariff {
        pub identifier: uuid::Uuid,
        pub provider: Provider,
        pub name: String,
        pub monthly_fee: f64,
        pub note: String,
        pub image: Option<String>,
        pub standard: bool,
        pub url: Option<String>,
    }
}
