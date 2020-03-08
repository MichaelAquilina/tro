use std::error;
use std::fmt;

#[derive(Debug)]
pub enum TrelloError {
    Reqwest(reqwest::Error),
    UrlParse(reqwest::UrlError),
    Io(std::io::Error),
    CardParse(String),
}

impl fmt::Display for TrelloError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TrelloError::Reqwest(err) => write!(f, "Reqwest Error: {}", err),
            TrelloError::UrlParse(err) => write!(f, "UrlParse Error: {}", err),
            TrelloError::Io(err) => write!(f, "IO Error: {}", err),
            TrelloError::CardParse(msg) => write!(f, "Card Parse Error: {}", msg),
        }
    }
}

impl error::Error for TrelloError {
    fn description(&self) -> &str {
        match self {
            TrelloError::Reqwest(err) => err.description(),
            TrelloError::UrlParse(err) => err.description(),
            TrelloError::Io(err) => err.description(),
            TrelloError::CardParse(msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            TrelloError::Reqwest(ref err) => Some(err),
            TrelloError::UrlParse(ref err) => Some(err),
            TrelloError::Io(ref err) => Some(err),
            TrelloError::CardParse(_) => None,
        }
    }
}

impl From<std::io::Error> for TrelloError {
    fn from(err: std::io::Error) -> TrelloError {
        TrelloError::Io(err)
    }
}

impl From<reqwest::UrlError> for TrelloError {
    fn from(err: reqwest::UrlError) -> TrelloError {
        TrelloError::UrlParse(err)
    }
}

impl From<reqwest::Error> for TrelloError {
    fn from(err: reqwest::Error) -> TrelloError {
        TrelloError::Reqwest(err)
    }
}
