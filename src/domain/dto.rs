use std::fs::File;

pub struct CreateTermOfUseDTO {
    pub group: String,
    pub file: File,
    pub info: Option<String>,
}

pub struct UploadedFileDTO {
    pub path: String,
    pub url: String,
}
