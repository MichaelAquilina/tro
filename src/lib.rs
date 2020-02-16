#[macro_use]
extern crate simple_error;
#[macro_use]
extern crate log;

mod client;

#[cfg(test)]
mod test_lib;

pub use client::Client;

use colored::*;
use regex::RegexBuilder;
use serde::Deserialize;
use std::error::Error;
use std::fmt::Debug;

use simple_error::SimpleError;

fn title(text: &str) -> String {
    let border = "═".repeat(text.chars().count());

    [
        format!("╔═{}═╗", border),
        format!("║ {} ║", text),
        format!("╚═{}═╝", border),
    ]
    .join("\n")
}

fn header(text: &str, header_char: &str) -> String {
    [text, &header_char.repeat(text.chars().count())].join("\n")
}

pub trait TrelloObject: Debug {
    fn get_type() -> String;

    fn get_name(&self) -> &str;

    fn get_fields() -> &'static [&'static str];

    fn render(&self) -> String;
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: String,
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
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub closed: bool,
    pub url: String,
    pub labels: Option<Vec<Label>>,
}

impl TrelloObject for Card {
    fn get_type() -> String {
        String::from("Card")
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_fields() -> &'static [&'static str] {
        &["id", "name", "desc", "labels", "closed", "url"]
    }

    fn render(&self) -> String {
        [header(&self.name, "=").as_str(), &self.desc].join("\n")
    }
}

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

    fn render(&self) -> String {
        let title = header(&self.name, "-").bold().to_string();
        let mut result: Vec<String> = vec![title];
        if let Some(cards) = &self.cards {
            for c in cards {
                trace!("{:?}", c);
                let mut lformat: Vec<String> = vec![];

                if c.desc != "" {
                    lformat.push("[...]".dimmed().to_string());
                }

                if let Some(labels) = &c.labels {
                    for l in labels {
                        lformat.push(l.render());
                    }
                }

                let s = format!("* {} {}", &c.name, lformat.join(" "));

                // trim end in case there is no data presented by lformat
                result.push(s.trim_end().to_string());
            }
        }
        result.join("\n")
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub id: String,
    pub name: String,
    pub closed: bool,
    pub url: String,
    pub lists: Option<Vec<List>>,
}

impl TrelloObject for Board {
    fn get_type() -> String {
        String::from("Board")
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_fields() -> &'static [&'static str] {
        &["id", "name", "closed", "url"]
    }

    fn render(&self) -> String {
        let mut result = vec![title(&self.name).bold().to_string()];
        if let Some(lists) = &self.lists {
            for list in lists {
                result.push(String::from(""));
                result.push(list.render());
            }
        }
        result.join("\n")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CardContents {
    pub name: String,
    pub desc: String,
}

impl Card {
    pub fn new(id: &str, name: &str, desc: &str, labels: Option<Vec<Label>>, url: &str) -> Card {
        Card {
            id: String::from(id),
            name: String::from(name),
            desc: String::from(desc),
            url: String::from(url),
            labels: labels,
            closed: false,
        }
    }

    /// Takes a buffer of contents that represent a Card render and parses
    /// it into a CardContents structure. This is similar to a deserialization process
    /// except this is quite unstructured and is not very strict in order to allow
    /// the user to more easily edit card contents.
    /// ```
    /// # use simple_error::SimpleError;
    /// # fn main() -> Result<(), SimpleError> {
    /// let buffer = "Hello World\n===\nThis is my card";
    /// let card_contents = trello::Card::parse(buffer)?;
    ///
    /// assert_eq!(
    ///     card_contents,
    ///     trello::CardContents {
    ///         name: String::from("Hello World"),
    ///         desc: String::from("This is my card"),
    ///     },
    /// );
    /// # Ok(())
    /// # }
    /// ```
    /// Invalid data will result in an appropriate error being returned.
    ///
    /// ```
    /// use simple_error::SimpleError;
    /// let buffer = "";
    /// let result = trello::Card::parse(buffer);
    /// assert_eq!(
    ///     result,
    ///     Err(SimpleError::new("Unable to parse - Unable to find name delimiter '===='"))
    /// );
    /// ```
    pub fn parse(buffer: &str) -> Result<CardContents, SimpleError> {
        // this is guaranteed to give at least one result
        let mut contents = buffer.split("\n").collect::<Vec<&str>>();
        trace!("{:?}", contents);

        // first line should *always* be the name of the card
        let mut name = vec![contents.remove(0)];

        // continue generating the name until we find a line entirely composed of '-'
        // we cannot calculate header() here because we allow the user the benefit of not
        // having to add or remove characters in case the name grows or shrinks in size
        let mut found = false;
        while contents.len() > 0 {
            let line = contents.remove(0);

            if &line.chars().take_while(|c| c == &'=').collect::<String>() != line {
                name.push(line);
            } else {
                found = true;
                break;
            }
        }

        if !found {
            bail!("Unable to parse - Unable to find name delimiter '===='");
        }

        let name = name.join("\n");
        // The rest of the contents is assumed to be the description
        let desc = contents.join("\n");

        Ok(CardContents {
            name: String::from(name),
            desc: String::from(desc),
        })
    }

    pub fn create(client: &Client, list_id: &str, card: &Card) -> Result<Card, Box<dyn Error>> {
        let url = client.get_trello_url(
            "/1/cards/",
            &[
                ("name", &card.name),
                ("desc", &card.desc),
                ("idList", list_id),
            ],
        )?;

        Ok(reqwest::Client::new()
            .post(url)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn update(client: &Client, card: &Card) -> Result<Card, Box<dyn Error>> {
        let url = client.get_trello_url(
            &format!("/1/cards/{}/", &card.id),
            &[
                ("name", &card.name),
                ("desc", &card.desc),
                ("closed", &card.closed.to_string()),
            ],
        )?;

        Ok(reqwest::Client::new()
            .put(url)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn remove_label(
        client: &Client,
        card_id: &str,
        label_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        let url =
            client.get_trello_url(&format!("/1/cards/{}/idLabels/{}", card_id, label_id), &[])?;

        reqwest::Client::new()
            .delete(url)
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn apply_label(
        client: &Client,
        card_id: &str,
        label_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        let url = client.get_trello_url(
            &format!("/1/cards/{}/idLabels", card_id),
            &[("value", label_id)],
        )?;

        reqwest::Client::new()
            .post(url)
            .send()?
            .error_for_status()?;

        Ok(())
    }
}

impl List {
    pub fn new(id: &str, name: &str, cards: Option<Vec<Card>>) -> List {
        List {
            id: String::from(id),
            name: String::from(name),
            cards: cards,
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
    ///         Card::new("1", "Orange", "", Some(vec![Label::new("", "fruit", "")]), ""),
    ///         Card::new("2", "Green", "", None, ""),
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
    ///             Card::new("1", "Orange", "", Some(vec![Label::new("", "fruit", "")]), "")
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

    pub fn create(client: &Client, board_id: &str, name: &str) -> Result<List, Box<dyn Error>> {
        let url = client.get_trello_url("/1/lists/", &[("name", name), ("idBoard", board_id)])?;

        Ok(reqwest::Client::new()
            .post(url)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn update(client: &Client, list: &List) -> Result<List, Box<dyn Error>> {
        let url = client.get_trello_url(
            &format!("/1/lists/{}/", &list.id),
            &[("name", &list.name), ("closed", &list.closed.to_string())],
        )?;

        Ok(reqwest::Client::new()
            .put(url)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn get_all_cards(client: &Client, list_id: &str) -> Result<Vec<Card>, Box<dyn Error>> {
        let url = client.get_trello_url(
            &format!("/1/lists/{}/cards/", list_id),
            &[("fields", &Card::get_fields().join(","))],
        )?;
        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }
}

impl Board {
    pub fn new(id: &str, name: &str, lists: Option<Vec<List>>, url: &str) -> Board {
        Board {
            id: String::from(id),
            name: String::from(name),
            url: String::from(url),
            lists: lists,
            closed: false,
        }
    }

    pub fn filter(&self, filter_name: &str) -> Board {
        let mut result = self.clone();

        result.lists = if let Some(lists) = result.lists {
            Some(lists.into_iter().map(|l| l.filter(filter_name)).collect())
        } else {
            None
        };
        result
    }

    /// Retrieves any missing nested content for the given board. This potentially
    /// means one or more network requests in order to retrieve the data. The Board
    /// will be mutated to include all its associated lists. The lists will also in turn
    /// contain the associated card resources.
    pub fn retrieve_nested(&mut self, client: &Client) -> Result<(), Box<dyn Error>> {
        self.lists = Some(Board::get_all_lists(client, &self.id, true)?);

        Ok(())
    }

    pub fn create(client: &Client, name: &str) -> Result<Board, Box<dyn Error>> {
        let url = client.get_trello_url("/1/boards/", &[("name", name)])?;

        Ok(reqwest::Client::new()
            .post(url)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn update(client: &Client, board: &Board) -> Result<Board, Box<dyn Error>> {
        let url = client.get_trello_url(
            &format!("/1/boards/{}/", &board.id),
            &[("name", &board.name), ("closed", &board.closed.to_string())],
        )?;

        Ok(reqwest::Client::new()
            .put(url)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn get_all(client: &Client) -> Result<Vec<Board>, Box<dyn Error>> {
        let url = client.get_trello_url(
            "/1/members/me/boards/",
            &[
                ("filter", "open"),
                ("fields", &Board::get_fields().join(",")),
            ],
        )?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }

    pub fn get(client: &Client, board_id: &str) -> Result<Board, Box<dyn Error>> {
        let url = client.get_trello_url(
            &format!("/1/boards/{}", board_id),
            &[("fields", &Board::get_fields().join(","))],
        )?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }

    pub fn get_all_labels(client: &Client, board_id: &str) -> Result<Vec<Label>, Box<dyn Error>> {
        let fields = Label::get_fields().join(",");

        let url = client.get_trello_url(
            &format!("/1/boards/{}/labels", board_id),
            &[("fields", &fields)],
        )?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }

    pub fn get_all_lists(
        client: &Client,
        board_id: &str,
        cards: bool,
    ) -> Result<Vec<List>, Box<dyn Error>> {
        let fields = List::get_fields().join(",");
        let mut params = vec![("fields", fields.as_str())];

        if cards {
            params.push(("cards", "open"));
        }

        let url = client.get_trello_url(&format!("/1/boards/{}/lists", board_id), &params)?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }
}
