use actix_multipart::form::{MultipartForm, json::Json, tempfile::TempFile};
use serde::Deserialize;

use crate::domain::dto::CreateTermOfUseDTO;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgreementPayload {
    pub user_id: i32,
    pub term_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateTermPayload {
    pub group: String,
    #[serde(default)]
    pub info: Option<String>,
}

impl From<CreateTermPayload> for CreateTermOfUseDTO {
    fn from(payload: CreateTermPayload) -> Self {
        Self {
            group: payload.group,
            info: payload.info,
        }
    }
}

#[derive(Debug, MultipartForm)]
pub struct CreateTermForm {
    #[multipart(limit = "20MB")]
    pub file: TempFile,
    pub data: Json<CreateTermPayload>,
}

#[derive(Debug, Deserialize)]
pub struct GetLatestTermPayload {
    #[serde(default)]
    pub only_url: bool,
}
