use chrono::NaiveDateTime;

pub struct TermOfUse {
    pub id: i32,
    pub group: String,
    pub url: String,
    pub version: u32,
    pub info: Option<String>,
    pub created_at: NaiveDateTime,
}

pub struct TermOfUserAgreement {
    pub id: i32,
    pub term_of_use_id: i32,
    pub user_id: i32,
    pub agreed_at: NaiveDateTime,
}
