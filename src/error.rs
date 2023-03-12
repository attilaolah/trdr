use reqwest;
use tokio_postgres;

#[derive(Debug)]
pub enum Error {
    RqError(reqwest::Error),
    PgError(tokio_postgres::Error),
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
