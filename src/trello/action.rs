use crate::client::Client;
use crate::trello_error::TrelloError;
use crate::trello_object::{Renderable, TrelloObject};

use serde::Deserialize;

type Result<T> = std::result::Result<T, TrelloError>;

// https://developer.atlassian.com/cloud/trello/guides/rest-api/object-definitions/#action-object
#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub id: String,
    #[serde(rename = "type")]
    pub action_type: String, // TODO: Could this be an enum?
    pub date: String,
}

impl Renderable for Action {
    fn render(&self) -> String {
        format!("{} on {}", &self.action_type, &self.date)
    }
}

impl TrelloObject for Action {
    fn get_type() -> String {
        String::from("Action")
    }

    fn get_name(&self) -> &str {
        &self.id
    }

    fn get_fields() -> &'static [&'static str] {
        &["id", "type", "date"]
    }
}

impl Action {
    pub fn get_all(client: &Client, board_id: &str) -> Result<Vec<Action>> {
        let url = client.get_trello_url(
            &format!("/1/boards/{}/actions", board_id),
            &[("fields", &Action::get_fields().join(","))],
        )?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }
}
