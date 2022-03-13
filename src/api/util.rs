use super::{ApiJson, ApiJsonList, CardVersion};
use const_format::concatcp;

pub fn json<T>(data: T) -> ApiJson<T> {
    Ok(axum::Json(data))
}

pub fn json_list<T>(data: Vec<T>) -> ApiJsonList<T> {
    json(data)
}

pub const fn fmt_card_path(version: CardVersion) -> &'static str {
    const PATH: &str = "/cards/de/:cpo_name/:charge_type";
    match version {
        CardVersion::V1 => PATH,
        CardVersion::V2 => concatcp!("/v2", PATH),
        CardVersion::V3 => concatcp!("/v3", PATH),
    }
}
