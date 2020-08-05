use super::board::Board;
use super::card::Card;
use super::client::Client;
use super::trello_error::TrelloError;
use super::trello_object::TrelloObject;

use serde::Deserialize;

type Result<T> = std::result::Result<T, TrelloError>;

pub struct SearchOptions {
    pub partial: bool,
    pub cards_limit: Option<i32>,
    pub boards_limit: Option<i32>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions {
            partial: false,
            cards_limit: None,
            boards_limit: None,
        }
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    #[serde(default = "Vec::new")]
    pub cards: Vec<Card>,
    #[serde(default = "Vec::new")]
    pub boards: Vec<Board>,
}

/// Implements the Trello Search API
/// https://developer.atlassian.com/cloud/trello/rest/api-group-search/#api-search-get
pub fn search(client: &Client, search_term: &str, options: &SearchOptions) -> Result<SearchResult> {
    let partial = options.partial.to_string();
    let card_fields = Card::get_fields().join(",");
    let board_fields = Board::get_fields().join(",");

    let mut params = vec![
        ("query", search_term),
        ("partial", &partial),
        ("card_fields", &card_fields),
        ("board_fields", &board_fields),
    ];

    // declared in the outer scope so that references live long enough
    let cards_limit;
    let boards_limit;

    if let Some(value) = options.cards_limit {
        cards_limit = value.to_string();
        params.push(("cards_limit", &cards_limit));
    }
    if let Some(value) = options.boards_limit {
        boards_limit = value.to_string();
        params.push(("boards_limit", &boards_limit));
    }

    let url = client.get_trello_url("/1/search/", &params)?;

    Ok(reqwest::get(url)?.error_for_status()?.json()?)
}
