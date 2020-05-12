use super::board::Board;
use super::card::Card;
use super::client::Client;
use super::trello_error::TrelloError;
use super::trello_object::TrelloObject;

use serde::Deserialize;

type Result<T> = std::result::Result<T, TrelloError>;

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub cards: Vec<Card>,
    pub boards: Vec<Board>,
}

/// Implements the Trello Search API
/// https://developers.trello.com/reference/#search
pub async fn search(client: &Client, search_term: &str, partial: bool) -> Result<SearchResult> {
    let url = client.get_trello_url(
        "/1/search/",
        &[
            ("query", search_term),
            ("partial", &partial.to_string()),
            ("card_fields", &Card::get_fields().join(",")),
            ("board_fields", &Board::get_fields().join(",")),
        ],
    )?;
    Ok(reqwest::get(url).await?.error_for_status()?.json().await?)
}
