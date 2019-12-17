use super::client::Client;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub id: String,
    pub name: String,
    pub url: String,
}

impl Board {
    pub fn get_all(client: &Client) -> Result<Vec<Board>, Box<dyn Error>> {
        let url = client.get_trello_url("/1/members/me/boards", &vec![("filter", "open")])?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }

    pub fn get(client: &Client, board_id: &str) -> Result<Board, Box<dyn Error>> {
        let url = client.get_trello_url(&format!("/1/boards/{}", board_id), &vec![])?;

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
            "/1/members/me/boards?key=some-key&token=some-secret-token&filter=open",
        )
        .with_status(200)
        .with_body(
            json!([
                {"name": "TODO", "id": "abc-def", "url": "http://bit.ly/12"},
                {"name": "foo", "id": "123-456", "url": ""},
            ])
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "some-secret-token", "some-key");
        let result = Board::get_all(&client)?;
        let expected = vec![
            Board {
                name: String::from("TODO"),
                id: String::from("abc-def"),
                url: String::from("http://bit.ly/12"),
            },
            Board {
                name: String::from("foo"),
                id: String::from("123-456"),
                url: String::from(""),
            },
        ];

        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_get() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock("GET", "/1/boards/some-board-id?key=KEY&token=TOKEN")
            .with_status(200)
            .with_body(
                json!({
                    "name": "My Favourite Board",
                    "id": "some-board-id",
                    "url": "https://trello.com/boards/some-board-id",
                })
                .to_string(),
            )
            .create();

        let client = Client::new(&mockito::server_url(), "TOKEN", "KEY");
        let result = Board::get(&client, "some-board-id")?;
        let expected = Board {
            name: String::from("My Favourite Board"),
            id: String::from("some-board-id"),
            url: String::from("https://trello.com/boards/some-board-id"),
        };
        assert_eq!(result, expected);
        Ok(())
    }
}
