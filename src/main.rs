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


fn get_boards(token: &str, key: &str) -> Vec<Board> {
    let url = Url::parse_with_params(
        "https://api.trello.com/1/members/me/boards",
        &[("key", &key), ("token", &token)],
    ).unwrap();

    let mut resp = reqwest::get(url).unwrap();
    let text = resp.text().unwrap();

    return serde_json::from_str(&text).unwrap();
}


fn main() {
    let token = env::var("TRELLO_API_TOKEN").unwrap();
    let key = env::var("TRELLO_API_DEVELOPER_KEY").unwrap();

    let boards = get_boards(&token, &key);

    for board in boards {
        println!("{}", board.name);
    }
}
