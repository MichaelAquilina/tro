extern crate console;
extern crate reqwest;

use std::collections::HashMap;

use console::{style, StyledObject};
use reqwest::{Response, Url};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub name: String,
    pub desc_data: Option<String>,
    pub url: String,
    pub id: String,
    pub starred: bool,
    pub closed: bool,
    pub subscribed: bool,
    pub label_names: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct List {
    pub id: String,
    pub name: String,
    pub closed: bool,
    pub id_board: String,
    pub subscribed: bool,
    pub cards: Option<Vec<Card>>,
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

        // TODO: Use match instead of if statements
        // TODO: Use wider palette range
        if self.color == "red" {
            return result.red();
        } else if self.color == "yellow" {
            return result.yellow();
        } else if self.color == "green" {
            return result.green();
        } else if self.color == "purple" {
            return result.magenta();
        } else if self.color == "orange" {
            return result.yellow();
        } else if self.color == "pink" {
            return result.red();
        } else if self.color == "lime" {
            return result.green();
        }

        return result;
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

fn get_resource(url: &str, params: &Vec<(&str, &str)>) -> Response {
    let url = Url::parse_with_params(url, params).unwrap();

    return reqwest::get(url).unwrap();
}

pub fn get_board(board_id: &str, token: &str, key: &str) -> Board {
    let mut resp = get_resource(
        &format!("https://api.trello.com/1/boards/{}", board_id),
        &vec![("key", key), ("token", token), ("fields", "all")],
    );
    return resp.json().unwrap();
}

pub fn get_board_by_name(board_name: &str, token: &str, key: &str) -> Option<Board> {
    let boards = get_boards(token, key);
    for b in boards {
        if b.name.to_lowercase() == board_name.to_lowercase() {
            return Some(b);
        }
    }
    return None
}

pub fn get_boards(token: &str, key: &str) -> Vec<Board> {
    let mut resp = get_resource(
        "https://api.trello.com/1/members/me/boards",
        &vec![("key", key), ("token", token), ("filter", "open")],
    );
    return resp.json().unwrap();
}

pub fn get_lists(board_id: &str, token: &str, key: &str) -> Vec<List> {
    let mut resp = get_resource(
        &format!("https://api.trello.com/1/boards/{}/lists", board_id),
        &vec![("key", key), ("token", token), ("cards", "open")],
    );
    return resp.json().unwrap();
}
