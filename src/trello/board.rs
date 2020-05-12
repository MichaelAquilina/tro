use crate::client::Client;
use crate::formatting::title;
use crate::list::List;
use crate::trello_error::TrelloError;
use crate::trello_object::{Renderable, TrelloObject};

use colored::*;
use serde::Deserialize;

type Result<T> = std::result::Result<T, TrelloError>;

// https://developer.atlassian.com/cloud/trello/guides/rest-api/object-definitions/#board-object
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
}

impl Renderable for Board {
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

impl Board {
    pub fn new(id: &str, name: &str, lists: Option<Vec<List>>, url: &str) -> Board {
        Board {
            id: String::from(id),
            name: String::from(name),
            url: String::from(url),
            lists,
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
    pub fn retrieve_nested(&mut self, client: &Client) -> Result<()> {
        self.lists = Some(List::get_all(client, &self.id, true)?);

        Ok(())
    }

    pub fn create(client: &Client, name: &str) -> Result<Board> {
        let url = client.get_trello_url("/1/boards/", &[])?;

        let params = [("name", name)];

        Ok(reqwest::Client::new()
            .post(url)
            .form(&params)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn open(client: &Client, board_id: &str) -> Result<Board> {
        let url = client.get_trello_url(&format!("/1/boards/{}", &board_id), &[])?;

        let params = [("closed", "false")];

        Ok(reqwest::Client::new()
            .put(url)
            .form(&params)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn update(client: &Client, board: &Board) -> Result<Board> {
        let url = client.get_trello_url(&format!("/1/boards/{}/", &board.id), &[])?;

        let params = [("name", &board.name), ("closed", &board.closed.to_string())];

        Ok(reqwest::Client::new()
            .put(url)
            .form(&params)
            .send()?
            .error_for_status()?
            .json()?)
    }

    pub fn get_all(client: &Client) -> Result<Vec<Board>> {
        let url = client.get_trello_url(
            "/1/members/me/boards/",
            &[
                ("filter", "open"),
                ("fields", &Board::get_fields().join(",")),
            ],
        )?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }

    pub fn get(client: &Client, board_id: &str) -> Result<Board> {
        let url = client.get_trello_url(
            &format!("/1/boards/{}", board_id),
            &[("fields", &Board::get_fields().join(","))],
        )?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }
}
