use thiserror::Error;

#[derive(Debug, Error)]
pub enum TrelloError {
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("url Parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Card Parse Error: {0}")]
    CardParse(String),
}
