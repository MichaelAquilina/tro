// TODO: Create generic Trello client that authenticates on its own
use serde::Deserialize;
use reqwest::Url;
use std::error::Error;

#[cfg(test)]
use mockito;


fn get_trello_url(path: &str, token: &str, key: &str) -> Result<Url, Box<dyn Error>> {
    #[cfg(test)]
    let host = mockito::server_url();

    #[cfg(not(test))]
    let host = "https://api.trello.com/";

    return Ok(Url::parse_with_params(
        &format!("{}{}", host, path),
        &[("key", key), ("token", token)],
    )?);
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub name: String,
    pub id: String,
    pub url: String,
}

impl Board {
    pub fn get(board_id: &str, token: &str, key: &str) -> Result<Board, Box<dyn Error>> {
        let url = get_trello_url(&format!("/1/boards/{}", board_id), token, key)?;

        Ok(reqwest::get(url)?.error_for_status()?.json()?)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;
    use serde_json::json;


    #[test]
    fn test_get_trello_url() -> Result<(), Box<dyn Error>>{
        let result = get_trello_url("/foo/bar/", "some-secret-token", "some-key")?.to_string();

        // FIXME: this is not technically correct, should fix it
        // * parameter order should not make a difference
        // * host name for non-test mode should somehow be tested
        // * mockito server can probably change
        assert_eq!(result, "http://127.0.0.1:1234/foo/bar/?key=some-key&token=some-secret-token");
        Ok(())
    }

    #[test]
    fn test_get_board() -> Result<(), Box<dyn Error>> {
        let _m = mock("GET", "/1/boards/some-board-id?key=KEY&token=TOKEN")
            .with_status(200)
            .with_body(json!({
                "name": "My Favourite Board",
                "id": "some-board-id",
                "url": "https://trello.com/boards/some-board-id",
            }).to_string())
            .create();

        let result = Board::get("some-board-id", "TOKEN", "KEY")?;
        let expected = Board {
            name: String::from("My Favourite Board"),
            id: String::from("some-board-id"),
            url: String::from("https://trello.com/boards/some-board-id"),
        };
        assert_eq!(result, expected);
        Ok(())
    }
}
