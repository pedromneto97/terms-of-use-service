#[derive(Debug)]
pub struct CreateTermOfUseDTO {
    pub group: String,
    pub info: Option<String>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AcceptedTermOfUseDTO {
    pub term_id: i32,
    pub user_id: i32,
    pub group: String,
}
