#[macro_use]
extern crate serde_derive;

extern crate reqwest;
extern crate serde_json;

use reqwest::Url;
use std::collections::HashMap;
use std::env;

#[derive(Deserialize, Serialize, Debug)]
struct Board {
    name: String,
    desc: String,
    url: String,
    id: String,
    starred: bool,
    closed: bool,
    subscribed: bool,
    // TODO: Figure out how to map to a snake_case name
    labelNames: HashMap<String, String>,
}

fn get_boards(token: &str, key: &str) -> Vec<Board> {
    let url = Url::parse_with_params(
        "https://api.trello.com/1/members/me/boards",
        &[("key", &key), ("token", &token)],
    )
    .unwrap();

    let mut resp = reqwest::get(url).unwrap();
    let text = resp.text().unwrap();

    return serde_json::from_str(&text).unwrap();
}

fn main() {
    let token = env::var("TRELLO_API_TOKEN").unwrap();
    let key = env::var("TRELLO_API_DEVELOPER_KEY").unwrap();

    let boards = get_boards(&token, &key);

    println!("{:?}", boards[0]);
}
