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

    /// Gets the resultant URL of the Trello Client given some path and additoinal
    /// parameters. The authentication credentials provided will be included as part
    /// of the generated URL
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = trello::Client {
    ///     host: String::from("https://api.trello.com"),
    ///     token: String::from("some-token"),
    ///     key: String::from("some-key"),
    /// };
    /// let url = client.get_trello_url("/1/me/boards/", &[])?;
    /// assert_eq!(
    ///     url.to_string(),
    ///     "https://api.trello.com/1/me/boards/?key=some-key&token=some-token"
    /// );
    /// # Ok(())
    /// # }
    /// ```
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
