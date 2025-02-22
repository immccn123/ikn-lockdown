use thiserror::Error;

use crate::utils::error_box;

#[derive(Error, Debug)]
pub enum LockdownError {
    #[error("error when calling windows api: {0}")]
    WinapiError(#[from] windows::core::Error),
    #[error("error when converting utf16 string to utf8 string: {0}")]
    FromUtf16Error(#[from] std::string::FromUtf16Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

pub trait UnwrapOrReport<T> {
    fn unwrap_or_report(self) -> T;
}

impl<T> UnwrapOrReport<T> for Result<T, LockdownError>  {
    fn unwrap_or_report(self) -> T {
        match self {
            Ok(x) => x,
            Err(e) => {
                error_box(&e);
                panic!("Error has been reported in another box");
            }
        }
    }
}
