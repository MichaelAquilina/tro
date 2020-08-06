use serde::Deserialize;

use crate::client::TrelloClient;
use crate::trello_error::TrelloError;

type Result<T> = std::result::Result<T, TrelloError>;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub id: String,
    pub full_name: String,
    pub username: String,
}

impl Member {
    pub fn me(client: &TrelloClient) -> Result<Member> {
        let url = client.config.get_trello_url("/1/members/me/", &[])?;

        Ok(client.client.get(url).send()?.error_for_status()?.json()?)
    }
}
