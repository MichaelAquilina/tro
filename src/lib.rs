#[macro_use]
extern crate simple_error;
#[macro_use]
extern crate log;

mod client;

#[cfg(test)]
mod test_lib;

pub use client::Client;

use colored::*;
use serde::Deserialize;
use std::error::Error;

use simple_error::SimpleError;

fn header(text: &str, header_char: &str) -> String {
    [text, &header_char.repeat(text.chars().count())].join("\n")
}

pub trait TrelloObject {
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
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_fields() -> &'static [&'static str] {
        &["id", "name", "color"]
    }

    fn render(&self) -> String {
        format!("[{}]", self.name.color(map_color(&self.color)))
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub closed: bool,
    pub labels: Option<Vec<Label>>,
}

impl TrelloObject for Card {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_fields() -> &'static [&'static str] {
        &["id", "name", "desc", "labels", "closed"]
    }

    fn render(&self) -> String {
        [header(&self.name, "-").as_str(), &self.desc].join("\n")
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
    pub lists: Option<Vec<List>>,
}

impl TrelloObject for Board {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_fields() -> &'static [&'static str] {
        &["id", "name", "closed"]
    }

    fn render(&self) -> String {
        let mut result = vec![header(&self.name, "=").bold().to_string()];
        if let Some(lists) = &self.lists {
            for list in lists {
                result.push(String::from(""));
                result.push(list.render());
            }
        }
        result.join("\n")
    }
}

impl Card {
    pub fn new(id: &str, name: &str, desc: &str) -> Card {
        Card {
            id: String::from(id),
            name: String::from(name),
            desc: String::from(desc),
            labels: None,
            closed: false,
        }
    }

    /// Takes a buffer of contents that represent a Card render and parses
    /// it into a Card structure. This is similar to a deserialization process
    /// except this is quite unstructured and is not very strict in order to allow
    /// the user to more easily edit card contents.
    ///
    /// Card id is left empty as there is no way to derive that from the contents.
    /// The resultant card is also assumed to be open.
    /// ```
    /// # use simple_error::SimpleError;
    /// # fn main() -> Result<(), SimpleError> {
    /// let buffer = "Hello World\n---\nThis is my card";
    /// let card = trello::Card::parse(buffer)?;
    ///
    /// assert_eq!(
    ///     card,
    ///     trello::Card {
    ///         id: String::new(),
    ///         name: String::from("Hello World"),
    ///         desc: String::from("This is my card"),
    ///         labels: None,
    ///         closed: false,
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
    ///     Err(SimpleError::new("Unable to parse - Unable to find name delimiter '----'"))
    /// );
    /// ```
    pub fn parse(buffer: &str) -> Result<Card, SimpleError> {
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

            if &line.chars().take_while(|c| c == &'-').collect::<String>() != line {
                name.push(line);
            } else {
                found = true;
                break;
            }
        }

        if !found {
            bail!("Unable to parse - Unable to find name delimiter '----'");
        }

        let name = name.join("\n");
        // The rest of the contents is assumed to be the description
        let desc = contents.join("\n");

        Ok(Card {
            id: String::new(),
            name: String::from(name),
            desc: String::from(desc),
            labels: None,
            closed: false,
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
    pub fn new(id: &str, name: &str, lists: Option<Vec<List>>) -> Board {
        Board {
            id: String::from(id),
            name: String::from(name),
            lists: lists,
            closed: false,
        }
    }

    /// Retrieves any missing nested content for the given board. This potentially
    /// means one or more network requests in order to retrieve the data. The Board
    /// will be mutated to include all its associated lists. The lists will also in turn
    /// contain the associated card resources.
    pub fn retrieve_nested(&mut self, client: &Client) -> Result<(), Box<dyn Error>> {
        // TODO: might be more efficient to just re-retrieve all lists with cards: true?
        if let Some(lists) = &mut self.lists {
            for list in lists {
                list.cards = Some(List::get_all_cards(client, &list.id)?);
            }
        } else {
            self.lists = Some(Board::get_all_lists(client, &self.id, true)?);
        }

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
