use crate::client::TrelloClient;
use crate::formatting::header;
use crate::label::Label;
use crate::trello_error::TrelloError;
use crate::trello_object::{Renderable, TrelloObject};

use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::Deserialize;
use std::str::FromStr;

type Result<T> = std::result::Result<T, TrelloError>;

// https://developer.atlassian.com/cloud/trello/guides/rest-api/object-definitions/#card-object
#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub closed: bool,
    pub url: String,
    pub labels: Option<Vec<Label>>,
    pub due: Option<DateTime<Utc>>,
}

impl TrelloObject for Card {
    fn get_type() -> String {
        String::from("Card")
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_fields() -> &'static [&'static str] {
        &["id", "name", "desc", "labels", "closed", "due", "url"]
    }
}

impl Renderable for Card {
    fn render(&self, headers: bool) -> String {
        match headers {
            true => [header(&self.name, "=").as_str(), &self.desc].join("\n"),
            false => self.desc.clone(),
        }
    }

    fn simple_render(&self) -> String {
        let mut lformat: Vec<String> = vec![];

        if self.closed {
            lformat.push("[Closed]".red().to_string());
        }

        lformat.push(String::from(&self.name));

        if self.desc != "" {
            lformat.push("[...]".dimmed().to_string());
        }

        if let Some(labels) = &self.labels {
            for l in labels {
                lformat.push(l.simple_render());
            }
        }

        // trim end in case there is no data presented by lformat
        lformat.join(" ").trim_end().to_string()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CardContents {
    pub name: String,
    pub desc: String,
}

impl FromStr for CardContents {
    type Err = TrelloError;

    /// Takes a buffer of contents that represent a Card render and parses
    /// it into a CardContents structure. This is similar to a deserialization process
    /// except this is quite unstructured and is not very strict in order to allow
    /// the user to more easily edit card contents.
    /// ```
    /// # fn main() -> Result<(), trello::TrelloError> {
    /// let buffer = "Hello World\n===\nThis is my card";
    /// let card_contents: trello::CardContents = buffer.parse()?;
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
    fn from_str(value: &str) -> Result<CardContents> {
        // this is guaranteed to give at least one result
        let mut contents = value.split('\n').collect::<Vec<&str>>();
        trace!("{:?}", contents);

        // first line should *always* be the name of the card
        let mut name = vec![contents.remove(0)];

        // continue generating the name until we find a line entirely composed of '='
        // we cannot calculate header() here because we allow the user the benefit of not
        // having to add or remove characters in case the name grows or shrinks in size
        let mut found = false;
        while !contents.is_empty() {
            let line = contents.remove(0);

            if line.chars().take_while(|c| c == &'=').collect::<String>() != line {
                name.push(line);
            } else {
                found = true;
                break;
            }
        }

        if !found {
            return Err(TrelloError::CardParse(
                "Unable to find name delimiter '===='".to_owned(),
            ));
        }

        let name = name.join("\n");
        // The rest of the contents is assumed to be the description
        let desc = contents.join("\n");

        Ok(CardContents { name, desc })
    }
}

impl Card {
    pub fn new(
        id: &str,
        name: &str,
        desc: &str,
        labels: Option<Vec<Label>>,
        url: &str,
        due: Option<DateTime<Utc>>,
    ) -> Card {
        Card {
            id: String::from(id),
            name: String::from(name),
            desc: String::from(desc),
            url: String::from(url),
            labels,
            due,
            closed: false,
        }
    }

    pub fn get(client: &TrelloClient, card_id: &str) -> Result<Card> {
        let url = client
            .config
            .get_trello_url(&format!("/1/cards/{}", card_id), &[])?;

        Ok(client.client.get(url).send()?.error_for_status()?.json()?)
    }

    pub fn create(client: &TrelloClient, list_id: &str, card: &Card) -> Result<Card> {
        let url = client.config.get_trello_url("/1/cards/", &[])?;

        let params: [(&str, &str); 3] = [
            ("name", &card.name),
            ("desc", &card.desc),
            ("idList", list_id),
        ];

        Ok(client
            .client
            .post(url)
            .form(&params)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn open(client: &TrelloClient, card_id: &str) -> Result<Card> {
        let url = client
            .config
            .get_trello_url(&format!("/1/cards/{}", &card_id), &[])?;

        let params = [("closed", "false")];

        Ok(client
            .client
            .put(url)
            .form(&params)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn update(client: &TrelloClient, card: &Card) -> Result<Card> {
        let url = client
            .config
            .get_trello_url(&format!("/1/cards/{}/", &card.id), &[])?;

        let params = [
            ("name", &card.name),
            ("desc", &card.desc),
            ("closed", &card.closed.to_string()),
        ];

        Ok(client
            .client
            .put(url)
            .form(&params)
            .send()?
            .error_for_status()?
            .json()?)
    }

    // Moves a card to the list with the specified id
    pub fn change_list(client: &TrelloClient, card_id: &str, list_id: &str) -> Result<()> {
        let url = client
            .config
            .get_trello_url(&format!("/1/cards/{}/", card_id), &[])?;

        let params = [("idList", list_id)];

        client
            .client
            .put(url)
            .form(&params)
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn get_all(client: &TrelloClient, list_id: &str) -> Result<Vec<Card>> {
        let url = client.config.get_trello_url(
            &format!("/1/lists/{}/cards/", list_id),
            &[("fields", &Card::get_fields().join(","))],
        )?;
        Ok(client.client.get(url).send()?.error_for_status()?.json()?)
    }
}
