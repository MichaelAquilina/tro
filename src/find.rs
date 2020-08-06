use clap::ArgMatches;
use regex::RegexBuilder;
use std::cmp::Ordering;
use thiserror::Error;
use trello::{Board, Card, List, TrelloClient, TrelloObject};

#[derive(Debug, PartialEq, Error)]
pub enum FindError {
    #[error("Regex Error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Multiple found: {0}")]
    Multiple(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Wildcard error: {0}")]
    WildCard(String),
}

/// Searches through a collection of Trello objects and tries
/// to match one and only one object to the name pattern provided.
/// * If no matches are found, an Error is returned
/// * If more than match is found, an Error is returned
/// * If only one item is matched, then it is returned
pub fn get_object_by_name<'a, T: TrelloObject>(
    objects: &'a [T],
    name: &str,
    ignore_case: bool,
) -> Result<&'a T, FindError> {
    let re = RegexBuilder::new(name)
        .case_insensitive(ignore_case)
        .build()?;

    let mut objects = objects
        .iter()
        .filter(|o| re.is_match(&o.get_name()))
        .collect::<Vec<&T>>();

    match objects.len().cmp(&1) {
        Ordering::Equal => Ok(objects.pop().unwrap()),
        Ordering::Greater => {
            return Err(FindError::Multiple(format!(
                "More than one {} found. Specify a more precise filter than '{}' (Found {})",
                T::get_type(),
                name,
                objects
                    .iter()
                    .map(|t| format!("'{}'", t.get_name()))
                    .collect::<Vec<String>>()
                    .join(", ")
            )))
        }
        Ordering::Less => {
            return Err(FindError::NotFound(format!(
                "{} not found. Specify a more precise filter than '{}'",
                T::get_type(),
                name
            )))
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TrelloResult {
    pub board: Option<Board>,
    pub list: Option<List>,
    pub card: Option<Card>,
}

#[derive(Debug, PartialEq)]
pub struct TrelloParams<'a> {
    pub board_name: Option<&'a str>,
    pub list_name: Option<&'a str>,
    pub card_name: Option<&'a str>,
    pub ignore_case: bool,
}

pub fn get_trello_params<'a>(matches: &'a ArgMatches) -> TrelloParams<'a> {
    TrelloParams {
        board_name: matches.value_of("board_name"),
        list_name: matches.value_of("list_name"),
        card_name: matches.value_of("card_name"),
        ignore_case: !matches.is_present("case_sensitive"),
    }
}

pub fn get_trello_object(
    client: &TrelloClient,
    params: &TrelloParams,
) -> Result<TrelloResult, Box<dyn std::error::Error>> {
    let board_name = match params.board_name {
        Some(bn) => bn,
        None => {
            return Ok(TrelloResult {
                board: None,
                list: None,
                card: None,
            })
        }
    };
    let boards = Board::get_all(&client)?;
    let mut board = get_object_by_name(&boards, &board_name, params.ignore_case)?.clone();

    // This should retrieve everything at once
    // This means better performance as it's less HTTP requests. But it does
    // mean we might retrieve more than we actually need in memory.
    board.retrieve_nested(client)?;

    if let Some("-") = params.list_name {
        if let Some(card_name) = params.card_name {
            let board_out = board.clone();
            let lists = board.lists.unwrap();

            let cards = lists
                .into_iter()
                .map(|l| l.cards.unwrap())
                .flatten()
                .collect::<Vec<Card>>();
            let card = get_object_by_name(&cards, &card_name, params.ignore_case)?;

            return Ok(TrelloResult {
                board: Some(board_out),
                list: None,
                card: Some(card.clone()),
            });
        } else {
            Err(Box::new(FindError::WildCard(
                "Card name must be specified with list '-' wildcard".to_string(),
            )))
        }
    } else if let Some(list_name) = params.list_name {
        let lists = &board.lists.as_ref().unwrap();
        let list = get_object_by_name(lists, &list_name, params.ignore_case)?.clone();

        if let Some(card_name) = params.card_name {
            let cards = &list.cards.as_ref().unwrap();

            let card = get_object_by_name(&cards, &card_name, params.ignore_case)?.clone();
            return Ok(TrelloResult {
                board: Some(board),
                list: Some(list),
                card: Some(card),
            });
        } else {
            return Ok(TrelloResult {
                board: Some(board),
                list: Some(list),
                card: None,
            });
        }
    } else {
        return Ok(TrelloResult {
            board: Some(board),
            list: None,
            card: None,
        });
    }
}
