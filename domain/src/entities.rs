use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TermOfUse {
    pub id: i32,
    pub group: String,
    pub url: String,
    pub version: u32,
    pub info: Option<String>,
    pub created_at: NaiveDateTime,
}
