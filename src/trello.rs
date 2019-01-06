extern crate console;
extern crate reqwest;

use std::collections::HashMap;

use console::{style, StyledObject};
use reqwest::{Response, Url};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub name: String,
    pub desc: String,
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

fn get_resource(url: &str, token: &str, key: &str) -> Response {
    let url = Url::parse_with_params(url, &[("key", key), ("token", token)]).unwrap();

    return reqwest::get(url).unwrap();
}

pub fn get_boards(token: &str, key: &str) -> Vec<Board> {
    let mut resp = get_resource("https://api.trello.com/1/members/me/boards", token, key);
    return resp.json().unwrap();
}

pub fn get_lists(board_id: &str, token: &str, key: &str) -> Vec<List> {
    let mut resp = get_resource(
        &format!("https://api.trello.com/1/boards/{}/lists", board_id),
        token,
        key,
    );
    return resp.json().unwrap();
}

pub fn get_cards(board_id: &str, token: &str, key: &str) -> Vec<Card> {
    let mut resp = get_resource(
        &format!("https://api.trello.com/1/boards/{}/cards", board_id),
        token,
        key,
    );
    return resp.json().unwrap();
}

pub fn create_card(list_id: &str, token: &str, key: &str) {
    let url = Url::parse_with_params(
        "https://api.trello.com/1/cards",
        &[("token", token), ("key", key), ("idList", list_id)],
    )
    .unwrap();
    let client = reqwest::Client::new();
    client.post(url).send().unwrap();
}
