#[macro_use]
extern crate serde_derive;

extern crate reqwest;
extern crate serde_json;

use reqwest::{Response, Url};
use std::collections::HashMap;
use std::env;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Board {
    name: String,
    desc: String,
    url: String,
    id: String,
    starred: bool,
    closed: bool,
    subscribed: bool,
    label_names: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
struct Label {
    id: String,
    name: String,
    color: String,
}

#[derive(Deserialize, Debug)]
struct Card {
    id: String,
    desc: String,
    name: String,
    url: String,
    labels: Vec<Label>,
}

fn get_resource(url: &str, token: &str, key: &str) -> Response {
    let url = Url::parse_with_params(url, &[("key", key), ("token", token)]).unwrap();

    return reqwest::get(url).unwrap();
}

fn get_boards(token: &str, key: &str) -> Vec<Board> {
    let mut resp = get_resource("https://api.trello.com/1/members/me/boards", token, key);
    return resp.json().unwrap();
}

fn get_cards(board_id: &str, token: &str, key: &str) -> Vec<Card> {
    let mut resp = get_resource(
        &format!("https://api.trello.com/1/boards/{}/cards", board_id),
        token,
        key,
    );
    return resp.json().unwrap();
}

fn main() {
    let token = env::var("TRELLO_API_TOKEN").unwrap();
    let key = env::var("TRELLO_API_DEVELOPER_KEY").unwrap();

    let boards = get_boards(&token, &key);

    for (index, b) in boards.iter().enumerate() {
        if b.name == "TODO" {
            println!("{}: {}", index, b.name);
            let cards = get_cards(&b.id, &token, &key);
            for c in cards {
                let labels: Vec<&String> = c.labels.iter().map(|l| &l.name).collect();

                println!("{} {:?}", c.name, labels);
            }
            break;
        }
    }
}
