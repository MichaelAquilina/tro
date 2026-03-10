use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientConfig {
    #[serde(default = "ClientConfig::default_host")]
    pub host: String,
    pub token: String,
    pub key: String,
}

#[derive(Debug)]
pub struct TrelloClient {
    pub config: ClientConfig,
    pub client: reqwest::blocking::Client,
}

impl TrelloClient {
    pub fn new(config: ClientConfig) -> Self {
        let auth_value = HeaderValue::from_str(&format!(
            "OAuth oauth_consumer_key=\"{}\", oauth_token=\"{}\"",
            config.key, config.token
        ))
        .expect("API key and token must contain only visible ASCII characters");

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, auth_value);

        TrelloClient {
            config,
            client: reqwest::blocking::Client::builder()
                .default_headers(headers)
                .build()
                .expect("Failed to build HTTP client"),
        }
    }
}

impl ClientConfig {
    pub fn new(host: &str, token: &str, key: &str) -> Self {
        ClientConfig {
            host: String::from(host),
            token: String::from(token),
            key: String::from(key),
        }
    }

    fn config_dir() -> Result<PathBuf, Box<dyn Error>> {
        let mut config_path = dirs::config_dir().ok_or("Unable to determine config directory")?;
        config_path.push("tro");

        Ok(config_path)
    }

    fn config_path() -> Result<PathBuf, Box<dyn Error>> {
        let mut config_path = Self::config_dir()?;
        config_path.push("config.toml");

        Ok(config_path)
    }

    pub fn save_config(&self) -> Result<(), Box<dyn Error>> {
        fs::create_dir_all(Self::config_dir()?)?;

        let config_path = Self::config_path()?;
        debug!("Saving configuration to {:?}", config_path);

        fs::write(config_path, toml::to_string(self)?)?;

        Ok(())
    }

    pub fn load_config() -> Result<Self, Box<dyn Error>> {
        let config_path = Self::config_path()?;
        debug!("Loading configuration from {:?}", config_path);
        let contents = fs::read_to_string(config_path)?;

        Ok(toml::from_str(&contents)?)
    }

    pub fn default_host() -> String {
        String::from("https://api.trello.com")
    }

    /// Gets the resultant URL of the Trello Config given some path and additional
    /// parameters. Authentication is handled via the `Authorization` header on the
    /// HTTP client, not as query parameters.
    /// ```
    /// # fn main() -> Result<(), url::ParseError> {
    /// let config = trello::ClientConfig {
    ///     host: String::from("https://api.trello.com"),
    ///     token: String::from("some-token"),
    ///     key: String::from("some-key"),
    /// };
    /// let url = config.get_trello_url("/1/me/boards/", &[])?;
    /// assert_eq!(
    ///     url.to_string(),
    ///     "https://api.trello.com/1/me/boards/"
    /// );
    /// let url = config.get_trello_url("/1/boards/some-id/", &[("lists", "open")])?;
    /// assert_eq!(
    ///     url.to_string(),
    ///     "https://api.trello.com/1/boards/some-id/?lists=open",
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_trello_url(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<url::Url, url::ParseError> {
        if params.is_empty() {
            url::Url::parse(&format!("{}{}", self.host, path))
        } else {
            url::Url::parse_with_params(&format!("{}{}", self.host, path), params)
        }
    }
}
