use std::{error::Error, fmt};

#[derive(Debug)]
pub(crate) enum EstatError {
    Http(reqwest::Error),
    Json(serde_json::Error),
    ApiStatus { endpoint: &'static str, status: i64 },
    MissingStatus { endpoint: &'static str },
}

impl fmt::Display for EstatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(error) => write!(formatter, "e-Stat HTTP error: {error}"),
            Self::Json(error) => write!(formatter, "e-Stat JSON error: {error}"),
            Self::ApiStatus { endpoint, status } => {
                write!(
                    formatter,
                    "e-Stat {endpoint} failed with API status {status}"
                )
            }
            Self::MissingStatus { endpoint } => {
                write!(
                    formatter,
                    "e-Stat {endpoint} response omitted a valid RESULT.STATUS"
                )
            }
        }
    }
}

impl Error for EstatError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Http(error) => Some(error),
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for EstatError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error.without_url())
    }
}

impl From<serde_json::Error> for EstatError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
