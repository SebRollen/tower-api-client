use thiserror::Error;

#[derive(Debug, Error)]
/// Possible errors in the request-response lifecycle.
pub enum Error {
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    #[error("Http error: {0}")]
    Http(#[from] hyper::http::Error),

    #[error("Serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Serde error: {0}")]
    SerdeQs(#[from] serde_qs::Error),

    #[error("Serde error: {0}")]
    SerdeUrlEncoded(#[from] serde_urlencoded::ser::Error),

    #[error("Pagination error: {msg}")]
    Pagination { msg: String },

    #[error("Invalid request. Received status {0}. Message: {1}")]
    ClientError(hyper::StatusCode, String),

    #[error("Server error. Received status {0}. Message: {1}")]
    ServerError(hyper::StatusCode, String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;
