use crate::client::Client;
use crate::trello_error::TrelloError;
use crate::trello_object::{Renderable, TrelloObject};

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
}

impl Renderable for Label {
    fn render(&self) -> String {
        self.simple_render()
    }

    fn simple_render(&self) -> String {
        format!(" {} ", self.name)
            .color(Color::White)
            .on_color(map_color(&self.color))
            .to_string()
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

    pub fn get_all(client: &Client, board_id: &str) -> Result<Vec<Label>> {
        let fields = Label::get_fields().join(",");

        let url = client.get_trello_url(
            &format!("/1/boards/{}/labels", board_id),
            &[("fields", &fields)],
        )?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }

    pub fn remove(client: &Client, card_id: &str, label_id: &str) -> Result<()> {
        let url =
            client.get_trello_url(&format!("/1/cards/{}/idLabels/{}", card_id, label_id), &[])?;

        reqwest::Client::new()
            .delete(url)
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn apply(client: &Client, card_id: &str, label_id: &str) -> Result<()> {
        let url = client.get_trello_url(&format!("/1/cards/{}/idLabels", card_id), &[])?;

        let params = [("value", label_id)];

        reqwest::Client::new()
            .post(url)
            .form(&params)
            .send()?
            .error_for_status()?;

        Ok(())
    }
}

fn map_color(color: &str) -> Color {
    match color {
        // values retrieved by inspecting elements in a browser on trello.com
        // date obtained: 2020-07-14
        "sky" => Color::TrueColor {
            r: 0x00,
            g: 0xc2,
            b: 0x0e,
        },
        "lime" => Color::TrueColor {
            r: 0x51,
            g: 0xe8,
            b: 0x98,
        },
        "green" => Color::TrueColor {
            r: 0x61,
            g: 0xbd,
            b: 0x4f,
        },
        "purple" => Color::TrueColor {
            r: 0xc3,
            g: 0x77,
            b: 0xe0,
        },
        "orange" => Color::TrueColor {
            r: 0xff,
            g: 0x9f,
            b: 0x1a,
        },
        "yellow" => Color::TrueColor {
            r: 0xf2,
            g: 0xd6,
            b: 0x00,
        },
        "red" => Color::TrueColor {
            r: 0xeb,
            g: 0x5a,
            b: 0x46,
        },
        "blue" => Color::TrueColor {
            r: 0x00,
            g: 0x79,
            b: 0xbf,
        },
        "pink" => Color::TrueColor {
            r: 0xff,
            g: 0x78,
            b: 0xcb,
        },
        "black" => Color::TrueColor {
            r: 0x34,
            g: 0x45,
            b: 0x63,
        },
        value => {
            println!("Unknown color: {}", value);
            Color::from(color)
        }
    }
}
