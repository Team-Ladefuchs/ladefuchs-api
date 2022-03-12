use super::{ApiJson, ApiJsonList};

pub fn json<T>(data: T) -> ApiJson<T> {
    Ok(axum::Json(data))
}

pub fn json_list<T>(data: Vec<T>) -> ApiJsonList<T> {
    json(data)
}
