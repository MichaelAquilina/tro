use std::error;
use std::fmt;

#[derive(Debug)]
pub enum TrelloError {
    Reqwest(reqwest::Error),
    UrlParse(url::ParseError),
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

impl From<url::ParseError> for TrelloError {
    fn from(err: url::ParseError) -> TrelloError {
        TrelloError::UrlParse(err)
    }
}

impl From<reqwest::Error> for TrelloError {
    fn from(err: reqwest::Error) -> TrelloError {
        TrelloError::Reqwest(err)
    }
}
