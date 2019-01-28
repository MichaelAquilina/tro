extern crate console;
extern crate reqwest;

use std::collections::HashMap;

use console::{style, StyledObject};
use reqwest::{Client, Error, Response, Url};

fn get_resource(url: &str, params: &Vec<(&str, &str)>) -> Result<Response, Error> {
    let url = Url::parse_with_params(url, params).unwrap();

    return reqwest::get(url)?.error_for_status();
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub name: String,
    pub desc_data: Option<String>,
    pub url: String,
    pub id: String,
    pub starred: Option<bool>,
    pub closed: Option<bool>,
    pub subscribed: Option<bool>,
    pub label_names: HashMap<String, String>,
}

impl Board {
    pub fn get(board_id: &str, token: &str, key: &str) -> Result<Board, Error> {
        let mut resp = get_resource(
            &format!("https://api.trello.com/1/boards/{}", board_id),
            &vec![("key", key), ("token", token), ("fields", "all")],
        )?;
        return resp.json();
    }

    pub fn get_by_name(board_name: &str, token: &str, key: &str) -> Result<Option<Board>, Error> {
        let boards = Board::get_all(token, key)?;
        for b in boards {
            if b.name.to_lowercase() == board_name.to_lowercase() {
                return Ok(Some(b));
            }
        }
        return Ok(None);
    }

    pub fn get_all(token: &str, key: &str) -> Result<Vec<Board>, Error> {
        let mut resp = get_resource(
            "https://api.trello.com/1/members/me/boards",
            &vec![("key", key), ("token", token), ("filter", "open")],
        )?;
        return resp.json();
    }

    pub fn close(board_id: &str, token: &str, key: &str) -> Result<Board, Error> {
        let url = Url::parse_with_params(
            &format!("https://api.trello.com/1/board/{}/closed", board_id),
            &[("token", token), ("key", key), ("value", "true")],
        )
        .unwrap();
        let client = Client::new();
        return client.put(url).send()?.json();
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct List {
    pub id: String,
    pub name: String,
    pub closed: bool,
    pub id_board: String,
    pub subscribed: Option<bool>,
    pub cards: Option<Vec<Card>>,
}

impl List {
    pub fn get_all(board_id: &str, token: &str, key: &str) -> Result<Vec<List>, Error> {
        let mut resp = get_resource(
            &format!("https://api.trello.com/1/boards/{}/lists", board_id),
            &vec![("key", key), ("token", token), ("cards", "open")],
        )?;
        return resp.json();
    }

    pub fn close(list_id: &str, token: &str, key: &str) -> Result<List, Error> {
        let url = Url::parse_with_params(
            &format!("https://api.trello.com/1/list/{}/closed", list_id),
            &[("token", token), ("key", key), ("value", "true")],
        )
        .unwrap();
        let client = Client::new();
        let mut resp = client.put(url).send()?;
        return resp.json();
    }
}

#[derive(Deserialize, Debug)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: String,
}

impl Label {
    pub fn get_colored_name(&self) -> StyledObject<&String> {
        let result = style(&self.name);

        // TODO: Use wider palette range
        return match self.color.as_str() {
            "red" => result.red(),
            "yellow" => result.yellow(),
            "green" => result.green(),
            "purple" => result.magenta(),
            "orange" => result.yellow(),
            "pink" => result.red(),
            "lime" => result.green(),
            _ => result,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Card {
    pub id: String,
    pub desc: String,
    pub name: String,
    pub url: String,
    pub labels: Vec<Label>,
}

impl Card {
    pub fn get(card_id: &str, token: &str, key: &str) -> Result<Card, Error> {
        let mut resp = get_resource(
            &format!("https://api.trello.com/1/cards/{}", card_id),
            &vec![("token", token), ("key", key)],
        )?;
        return resp.json();
    }

    pub fn create(card: &Card, list_id: &str, token: &str, key: &str) -> Result<Card, Error> {
        let url = Url::parse_with_params(
            &format!("https://api.trello.com/1/cards"),
            &[
                ("token", token),
                ("key", key),
                ("idList", list_id),
                ("name", &card.name),
                ("desc", &card.desc),
            ],
        )
        .unwrap();
        let client = Client::new();
        let mut resp = client.post(url).send()?;
        return resp.json();
    }

    pub fn close(card_id: &str, token: &str, key: &str) -> Result<Card, Error> {
        let url = Url::parse_with_params(
            &format!("https://api.trello.com/1/cards/{}/closed", card_id),
            &[("token", token), ("key", key), ("value", "true")],
        )
        .unwrap();
        let client = Client::new();
        let mut resp = client.put(url).send()?;
        return resp.json();
    }
}
