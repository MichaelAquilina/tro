mod trello;

use serde::Deserialize;
use std::fs;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct Config {
    host: String,
    token: String,
    key: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut config_path = dirs::config_dir().expect(
        "Unable to determine config directory"
    );
    config_path.push("tro/config.toml");

    let contents = fs::read_to_string(config_path.to_str().unwrap())?;

    let config: Config = toml::from_str(&contents)?;

    let client = trello::Client::new(&config.host, &config.token, &config.key);

    let board_name = "TODO";

    if let Some(board) = get_board_by_name(&client, board_name)? {
        println!("{:?}", board);
    } else {
        println!("Could not find target board: '{}'", board_name);
    }

    return Ok(());
}

fn get_board_by_name(client: &trello::Client, name: &str) -> Result<Option<trello::Board>, Box<dyn Error>> {
    let boards = client.get_all_boards()?;

    for board in boards {
        if board.name == name {
            return Ok(Some(board));
        }
    }
    return Ok(None);
}
