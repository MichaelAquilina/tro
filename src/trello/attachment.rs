use crate::client::TrelloClient;
use crate::formatting::header;
use crate::trello_error::TrelloError;
use crate::trello_object::{Renderable, TrelloObject};

use serde::Deserialize;

type Result<T> = std::result::Result<T, TrelloError>;

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    pub name: String,
    pub url: String,
}

impl Attachment {
    pub fn get_all(client: &TrelloClient, card_id: &str) -> Result<Vec<Attachment>> {
        let url = client.config.get_trello_url(
            &format!("/1/cards/{}/attachments", card_id),
            &[("fields", &Attachment::get_fields().join(","))],
        )?;

        Ok(client.client.get(url).send()?.error_for_status()?.json()?)
    }

    pub fn apply(client: &TrelloClient, card_id: &str, file: &str) -> Result<Attachment> {
        let url = client
            .config
            .get_trello_url(&format!("/1/cards/{}/attachments", card_id), &[])?;

        let form = reqwest::multipart::Form::new().file("file", file)?;

        Ok(client
            .client
            .post(url)
            .multipart(form)
            .send()?
            .error_for_status()?
            .json()?)
    }
}

impl TrelloObject for Attachment {
    fn get_type() -> String {
        String::from("Attachment")
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_fields() -> &'static [&'static str] {
        &["id", "name", "url"]
    }
}

impl Renderable for Attachment {
    fn render(&self) -> String {
        [header(&self.name, "-").as_str(), &self.url].join("\n")
    }

    fn simple_render(&self) -> String {
        self.name.clone()
    }
}
