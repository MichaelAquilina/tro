use reqwest::Url;
use std::error::Error;

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

    pub fn get_trello_url(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<Url, Box<dyn Error>> {
        let auth_params = &[("key", self.key.as_str()), ("token", self.token.as_str())];

        Ok(Url::parse_with_params(
            &format!("{}{}", self.host, path),
            &[auth_params, params].concat(),
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_trello_url() -> Result<(), Box<dyn Error>> {
        let client = Client::new("https://api.trello.com", "some-secret-token", "some-key");
        let result = client.get_trello_url("/foo/bar/", &vec![])?.to_string();

        // FIXME: this is not technically correct, should fix it
        // * parameter order should not make a difference
        assert_eq!(
            result,
            "https://api.trello.com/foo/bar/?key=some-key&token=some-secret-token"
        );
        Ok(())
    }
}
