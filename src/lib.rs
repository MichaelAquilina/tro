mod client;

#[cfg(test)]
mod tests;

pub use client::Client;

use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: String,
    pub name: String,
    pub desc: String,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct List {
    pub id: String,
    pub name: String,
    pub cards: Option<Vec<Card>>,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub id: String,
    pub name: String,
    pub url: String,
    pub lists: Option<Vec<List>>,
}

impl List {
    pub fn get_all_cards(client: &Client, list_id: &str) -> Result<Vec<Card>, Box<dyn Error>> {
        let url = client.get_trello_url(&format!("/1/lists/{}/cards/", list_id), &[])?;
        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }
}

impl Board {
    pub fn get_all(client: &Client) -> Result<Vec<Board>, Box<dyn Error>> {
        let url = client.get_trello_url("/1/members/me/boards", &[("filter", "open")])?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }

    pub fn get(client: &Client, board_id: &str) -> Result<Board, Box<dyn Error>> {
        let url = client.get_trello_url(&format!("/1/boards/{}", board_id), &[])?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }

    pub fn get_all_lists(client: &Client, board_id: &str) -> Result<Vec<List>, Box<dyn Error>> {
        let url = client.get_trello_url(
            &format!("/1/boards/{}/lists", board_id),
            // TODO: Consider whether this makes idiomatic sense
            &[("cards", "open")],
        )?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }
}
