use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CustomError {
    HttpRequest(u16),
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::HttpRequest(errcode) => write!(f, "Http request failed with code {errcode}"),
        }
    }
}

impl Error for CustomError {}
