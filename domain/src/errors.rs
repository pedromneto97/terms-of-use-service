#[derive(Debug)]
pub enum TermsOfUseError {
    NotFound,
    InternalServerError,
}

pub type Result<T> = std::result::Result<T, TermsOfUseError>;
