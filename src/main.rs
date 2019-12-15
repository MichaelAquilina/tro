mod trello;

use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
struct Config {
    host: String,
    token: String,
    key: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config_path = dirs::config_dir().unwrap();
    config_path.push("tro/config.toml");

    let contents = fs::read_to_string(config_path.to_str().unwrap())?;

    let config: Config = toml::from_str(&contents)?;

    let client = trello::Client::new(&config.host, &config.token, &config.key);
    let boards = client.get_all_boards();

    println!("{:?}", boards);
    Ok(())
}
