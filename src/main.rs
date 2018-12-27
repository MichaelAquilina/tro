#[macro_use]
extern crate serde_derive;

extern crate reqwest;
extern crate serde_json;

use std::env;
use reqwest::Url;


#[derive(Deserialize, Debug)]
struct Board {
    name: String,
    desc: String,
    url: String,
}


fn main() {
    let token = env::var("TRELLO_API_TOKEN").unwrap();
    let key = env::var("TRELLO_API_DEVELOPER_KEY").unwrap();

    let url = Url::parse_with_params(
        "https://api.trello.com/1/members/me/boards",
        &[("key", &key), ("token", &token)],
    ).unwrap();

    println!("Requesting user board data");

    let mut resp = reqwest::get(url).unwrap();
    let text = resp.text().unwrap();

    let data: Vec<Board> = serde_json::from_str(&text).unwrap();
    for board in data {
        println!("{}", board.name);
    }
}
