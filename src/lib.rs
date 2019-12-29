mod client;

#[cfg(test)]
mod test_lib;

pub use client::Client;

use serde::Deserialize;
use std::error::Error;

fn header(text: &str, header_char: &str) -> String {
    [text, &header_char.repeat(text.chars().count())].join("\n")
}

pub trait TrelloObject {
    fn get_name(&self) -> &str;

    fn get_fields() -> &'static [&'static str];

    fn render(&self) -> String;
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

    fn get_fields() -> &'static [&'static str] {
        &["id", "name", "desc", "closed"]
    }

    fn render(&self) -> String {
        [header(&self.name, "-").as_str(), &self.desc].join("\n")
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

    fn get_fields() -> &'static [&'static str] {
        &["id", "name", "closed"]
    }

    fn render(&self) -> String {
        let title = header(&self.name, "-");
        let mut result: Vec<String> = vec![title];
        if let Some(cards) = &self.cards {
            for c in cards {
                let s = format!("* {}", &c.name);
                result.push(s);
            }
        }
        result.join("\n")
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub id: String,
    pub name: String,
    pub closed: bool,
    pub lists: Option<Vec<List>>,
}

impl TrelloObject for Board {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_fields() -> &'static [&'static str] {
        &["id", "name", "closed"]
    }

    fn render(&self) -> String {
        let mut result = vec![header(&self.name, "=")];
        if let Some(lists) = &self.lists {
            for list in lists {
                result.push(String::from(""));
                result.push(list.render());
            }
        }
        result.join("\n")
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
        let url = client.get_trello_url(
            &format!("/1/lists/{}/cards/", list_id),
            &[("fields", &Card::get_fields().join(","))],
        )?;
        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }
}

impl Board {
    pub fn retrieve_nested(&mut self, client: &Client) -> Result<(), Box<dyn Error>> {
        if let Some(lists) = &mut self.lists {
            for list in lists {
                list.cards = Some(List::get_all_cards(client, &list.id)?);
            }
        } else {
            self.lists = Some(Board::get_all_lists(client, &self.id, true)?);
        }

        Ok(())
    }

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
        let url = client.get_trello_url(
            "/1/members/me/boards/",
            &[
                ("filter", "open"),
                ("fields", &Board::get_fields().join(",")),
            ],
        )?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }

    pub fn get(client: &Client, board_id: &str) -> Result<Board, Box<dyn Error>> {
        let url = client.get_trello_url(
            &format!("/1/boards/{}", board_id),
            &[("fields", &Board::get_fields().join(","))],
        )?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }

    pub fn get_all_lists(
        client: &Client,
        board_id: &str,
        cards: bool,
    ) -> Result<Vec<List>, Box<dyn Error>> {
        let fields = List::get_fields().join(",");
        let mut params = vec![("fields", fields.as_str())];

        if cards {
            params.push(("cards", "open"));
        }

        let url = client.get_trello_url(&format!("/1/boards/{}/lists", board_id), &params)?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }
}
