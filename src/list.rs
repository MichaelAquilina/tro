use super::client::Client;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct List {
    pub id: String,
    pub name: String,
}

impl List {
    pub fn get_all(client: &Client, board_id: &str) -> Result<Vec<List>, Box<dyn Error>> {
        let url = client.get_trello_url(
            &format!("/1/boards/{}/lists", board_id),
            &[("cards", "open")],
        )?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;
    use serde_json::json;

    #[test]
    fn test_get_all() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "GET",
            "/1/boards/some-board-id/lists?key=some-key&token=some-token&cards=open",
        )
        .with_status(200)
        .with_body(
            json!([
                {"name": "Red", "id": "823-123"},
                {"name": "Green", "id": "222-222"},
            ])
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "some-token", "some-key");
        let result = List::get_all(&client, "some-board-id")?;
        let expected = vec![
            List {
                name: String::from("Red"),
                id: String::from("823-123"),
            },
            List {
                name: String::from("Green"),
                id: String::from("222-222"),
            },
        ];
        assert_eq!(result, expected);
        Ok(())
    }
}
