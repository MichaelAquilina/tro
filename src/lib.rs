mod client;

#[cfg(test)]
mod test_lib;

pub use client::Client;

use serde::Deserialize;
use std::error::Error;

pub trait TrelloObject {
    fn get_name(&self) -> &str;
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub closed: bool,
}

impl TrelloObject for Card {
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct List {
    pub id: String,
    pub name: String,
    pub closed: bool,
    pub cards: Option<Vec<Card>>,
}

impl TrelloObject for List {
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub id: String,
    pub name: String,
    pub closed: bool,
}

impl TrelloObject for Board {
    fn get_name(&self) -> &str {
        &self.name
    }
}

impl Card {
    pub fn create(client: &Client, list_id: &str, name: &str) -> Result<Card, Box<dyn Error>> {
        let url = client.get_trello_url("/1/cards/", &[("name", name), ("idList", list_id)])?;

        Ok(reqwest::Client::new()
            .post(url)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn update(client: &Client, card: &Card) -> Result<Card, Box<dyn Error>> {
        let url = client.get_trello_url(
            &format!("/1/cards/{}/", &card.id),
            &[
                ("name", &card.name),
                ("desc", &card.desc),
                ("closed", &card.closed.to_string()),
            ],
        )?;

        Ok(reqwest::Client::new()
            .put(url)
            .send()?
            .error_for_status()?
            .json()?)
    }
}

impl List {
    pub fn create(client: &Client, board_id: &str, name: &str) -> Result<List, Box<dyn Error>> {
        let url = client.get_trello_url("/1/lists/", &[("name", name), ("idBoard", board_id)])?;

        Ok(reqwest::Client::new()
            .post(url)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn update(client: &Client, list: &List) -> Result<List, Box<dyn Error>> {
        let url = client.get_trello_url(
            &format!("/1/lists/{}/", &list.id),
            &[("name", &list.name), ("closed", &list.closed.to_string())],
        )?;

        Ok(reqwest::Client::new()
            .put(url)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn get_all_cards(client: &Client, list_id: &str) -> Result<Vec<Card>, Box<dyn Error>> {
        let url = client.get_trello_url(&format!("/1/lists/{}/cards/", list_id), &[])?;
        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }
}

impl Board {
    pub fn create(client: &Client, name: &str) -> Result<Board, Box<dyn Error>> {
        let url = client.get_trello_url("/1/boards/", &[("name", name)])?;

        Ok(reqwest::Client::new()
            .post(url)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn update(client: &Client, board: &Board) -> Result<Board, Box<dyn Error>> {
        let url = client.get_trello_url(
            &format!("/1/boards/{}/", &board.id),
            &[("name", &board.name), ("closed", &board.closed.to_string())],
        )?;

        Ok(reqwest::Client::new()
            .put(url)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn get_all(client: &Client) -> Result<Vec<Board>, Box<dyn Error>> {
        let url = client.get_trello_url("/1/members/me/boards/", &[("filter", "open")])?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }

    pub fn get(client: &Client, board_id: &str) -> Result<Board, Box<dyn Error>> {
        let url = client.get_trello_url(&format!("/1/boards/{}", board_id), &[])?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }

    pub fn get_all_lists(
        client: &Client,
        board_id: &str,
        cards: bool,
    ) -> Result<Vec<List>, Box<dyn Error>> {
        let mut params = vec![];

        if cards {
            params.push(("cards", "open"));
        }

        let url = client.get_trello_url(&format!("/1/boards/{}/lists", board_id), &params)?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }
}
