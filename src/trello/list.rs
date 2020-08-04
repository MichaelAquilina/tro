use crate::card::Card;
use crate::client::Client;
use crate::formatting::header;
use crate::trello_object::{Renderable, TrelloObject};

use anyhow::Result;
use colored::*;
use regex::RegexBuilder;
use serde::Deserialize;

// https://developers.trello.com/reference/#list-object
#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct List {
    pub id: String,
    pub name: String,
    pub closed: bool,
    pub cards: Option<Vec<Card>>,
}

impl TrelloObject for List {
    fn get_type() -> String {
        String::from("List")
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_fields() -> &'static [&'static str] {
        &["id", "name", "closed"]
    }
}

impl Renderable for List {
    fn render(&self) -> String {
        let title = header(&self.name, "-").bold().to_string();
        let mut result: Vec<String> = vec![title];
        if let Some(cards) = &self.cards {
            for c in cards {
                result.push(format!("* {}", c.simple_render()));
            }
        }
        result.join("\n")
    }

    fn simple_render(&self) -> String {
        self.name.clone()
    }
}

impl List {
    pub fn new(id: &str, name: &str, cards: Option<Vec<Card>>) -> List {
        List {
            id: String::from(id),
            name: String::from(name),
            cards,
            closed: false,
        }
    }

    /// Filters cards that match the given label_filter (As a regular expression).
    /// Returns a copy of the original List, with the correct filtering applied.
    ///
    /// ```
    /// use trello::{Card, Label, List};
    ///
    /// let list = List::new(
    ///     "123",
    ///     "TODO",
    ///     Some(vec![
    ///         Card::new("1", "Orange", "", Some(vec![Label::new("", "fruit", "")]), "", None),
    ///         Card::new("2", "Green", "", None, "", None),
    ///     ])
    /// );
    ///
    /// assert_eq!(
    ///     list.filter("idontexist"),
    ///     List::new(
    ///         "123",
    ///         "TODO",
    ///         Some(vec![]),
    ///     )
    /// );
    ///
    /// assert_eq!(
    ///     list.filter("fruit"),
    ///     List::new(
    ///         "123",
    ///         "TODO",
    ///         Some(vec![
    ///             Card::new("1", "Orange", "", Some(vec![Label::new("", "fruit", "")]), "", None)
    ///         ])
    ///     )
    /// );
    /// ```
    pub fn filter(&self, label_filter: &str) -> List {
        let re = RegexBuilder::new(label_filter)
            .case_insensitive(true)
            .build()
            .expect("Invalid regex for label filter");

        let closure = |c: &Card| -> bool {
            if let Some(labels) = &c.labels {
                for label in labels {
                    if re.is_match(&label.name) {
                        return true;
                    }
                }
            }
            false
        };

        let mut result = self.clone();
        result.cards = if let Some(cards) = result.cards {
            Some(cards.into_iter().filter(closure).collect())
        } else {
            None
        };
        result
    }

    pub fn create(client: &Client, board_id: &str, name: &str) -> Result<List> {
        let url = client.get_trello_url("/1/lists/", &[])?;

        let params = [("name", name), ("idBoard", board_id)];

        Ok(reqwest::Client::new()
            .post(url)
            .form(&params)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn open(client: &Client, list_id: &str) -> Result<List> {
        let url = client.get_trello_url(&format!("/1/lists/{}", &list_id), &[])?;

        let params = [("closed", "false")];

        Ok(reqwest::Client::new()
            .put(url)
            .form(&params)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn update(client: &Client, list: &List) -> Result<List> {
        let url = client.get_trello_url(&format!("/1/lists/{}/", &list.id), &[])?;

        let params = [("name", &list.name), ("closed", &list.closed.to_string())];

        Ok(reqwest::Client::new()
            .put(url)
            .form(&params)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn get_all(client: &Client, board_id: &str, cards: bool) -> Result<Vec<List>> {
        let fields = List::get_fields().join(",");
        let mut params = vec![("fields", fields.as_str())];

        if cards {
            params.push(("cards", "open"));
        }

        let url = client.get_trello_url(&format!("/1/boards/{}/lists", board_id), &params)?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }
}
