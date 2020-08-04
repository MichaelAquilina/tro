use serde::Deserialize;

use crate::client::Client;

use anyhow::Result;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub id: String,
    pub full_name: String,
    pub username: String,
}

impl Member {
    pub fn me(client: &Client) -> Result<Member> {
        let url = client.get_trello_url("/1/members/me/", &[])?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }
}
