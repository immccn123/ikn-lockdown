use thiserror::Error;

#[derive(Error, Debug)]
pub enum LockdownError {
    #[error("error when calling windows api")]
    WinapiError(#[from] windows::core::Error),
    #[error("error when converting utf16 string to utf8 string")]
    FromUtf16Error(#[from] std::string::FromUtf16Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}
