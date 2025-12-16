use domain::entities::TermOfUse;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TermOfUseUrlResponse {
    pub url: String,
}

impl From<TermOfUse> for TermOfUseUrlResponse {
    fn from(term: TermOfUse) -> Self {
        TermOfUseUrlResponse { url: term.url }
    }
}

#[derive(Debug, Serialize)]
pub struct TermOfUseResponse {
    pub id: i32,
    pub url: String,
    pub group: String,
    pub info: Option<String>,
}

impl From<TermOfUse> for TermOfUseResponse {
    fn from(term: TermOfUse) -> Self {
        TermOfUseResponse {
            id: term.id,
            url: term.url,
            group: term.group,
            info: term.info,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HasConsentedResponse {
    pub has_consented: bool,
}
