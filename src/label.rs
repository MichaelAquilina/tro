use super::client::Client;
use super::trello_error::TrelloError;
use super::trello_object::TrelloObject;

use colored::*;
use serde::Deserialize;

type Result<T> = std::result::Result<T, TrelloError>;

// https://developers.trello.com/reference/#label-object
#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: String,
}

impl TrelloObject for Label {
    fn get_type() -> String {
        String::from("Label")
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_fields() -> &'static [&'static str] {
        &["id", "name", "color"]
    }

    fn render(&self) -> String {
        format!("[{}]", self.colored_name())
    }
}

impl Label {
    pub fn new(id: &str, name: &str, color: &str) -> Label {
        Label {
            id: String::from(id),
            name: String::from(name),
            color: String::from(color),
        }
    }

    pub fn colored_name(&self) -> ColoredString {
        self.name.color(map_color(&self.color))
    }

    pub fn get_all(client: &Client, board_id: &str) -> Result<Vec<Label>> {
        let fields = Label::get_fields().join(",");

        let url = client.get_trello_url(
            &format!("/1/boards/{}/labels", board_id),
            &[("fields", &fields)],
        )?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }
}

fn map_color(color: &str) -> &str {
    match color {
        "sky" => "cyan",
        "lime" => "green",
        "orange" => "yellow",
        // black is not visible on a terminal
        "black" => "bright black",
        _ => color,
    }
}
