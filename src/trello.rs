use reqwest::Url;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub name: String,
    pub id: String,
    pub url: String,
}

pub struct Client {
    pub host: String,
    pub token: String,
    pub key: String,
}

impl Client {
    pub fn new(host: &str, token: &str, key: &str) -> Client {
        Client {
            host: String::from(host),
            token: String::from(token),
            key: String::from(key),
        }
    }

    pub fn get_all_boards(self) -> Result<Vec<Board>, Box<dyn Error>> {
        let url = self.get_trello_url("/1/members/me/boards")?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }

    pub fn get_board(self, board_id: &str) -> Result<Board, Box<dyn Error>> {
        let url = self.get_trello_url(&format!("/1/boards/{}", board_id))?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }

    fn get_trello_url(self, path: &str) -> Result<Url, Box<dyn Error>> {
        return Ok(Url::parse_with_params(
            &format!("{}{}", self.host, path),
            &[("key", &self.key), ("token", &self.token)],
        )?);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;
    use serde_json::json;

    #[test]
    fn test_get_trello_url() -> Result<(), Box<dyn Error>> {
        let client = Client::new("https://api.trello.com", "some-secret-token", "some-key");
        let result = client.get_trello_url("/foo/bar/")?.to_string();

        // FIXME: this is not technically correct, should fix it
        // * parameter order should not make a difference
        assert_eq!(
            result,
            "https://api.trello.com/foo/bar/?key=some-key&token=some-secret-token"
        );
        Ok(())
    }

    #[test]
    fn test_get_all_boards() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "GET",
            "/1/members/me/boards?key=some-key&token=some-secret-token",
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
        let result = client.get_all_boards()?;
        let expected: Vec<Board> = vec![
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
    fn test_get_board() -> Result<(), Box<dyn Error>> {
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

        let result =
            Client::new(&mockito::server_url(), "TOKEN", "KEY").get_board("some-board-id")?;
        let expected = Board {
            name: String::from("My Favourite Board"),
            id: String::from("some-board-id"),
            url: String::from("https://trello.com/boards/some-board-id"),
        };
        assert_eq!(result, expected);
        Ok(())
    }
}
