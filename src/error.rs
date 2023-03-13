use reqwest;
use tokio_postgres;

#[derive(Debug)]
pub enum Error {
    Error(String),
    RqError(reqwest::Error),
    PgError(tokio_postgres::Error),
}

impl Error {
    pub fn new(message: String) -> Self {
        Self::Error(message)
    }
    pub fn new_with_code(message: &str, code: i32) -> Self {
        Self::new(format!("{} [code: {}]", message, code))
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::RqError(error)
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(error: tokio_postgres::Error) -> Self {
        Error::PgError(error)
    }
}
