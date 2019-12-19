#[macro_use]
extern crate clap;

use clap::{AppSettings, ArgMatches};
use serde::Deserialize;
use std::error::Error;
use std::fs;
use trello::{Board, Card, Client, List};

#[derive(Deserialize, Debug)]
struct Config {
    host: String,
    token: String,
    key: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap_app!(myapp =>
        (version: "1.0")
        (author: "Michael Aquilina")
        (about: "Trello CLI interface")
        (@subcommand board =>
            (about: "Commands related to Trello boards")
            (@subcommand get =>
                (@arg name: -n --name +takes_value "specify board name")
            )
        )
        (@subcommand card =>
            (about: "Commands related to Trello cards")
            (@subcommand get =>
                (@arg name: -n --name +takes_value "specify card name")
            )
        )
    )
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .get_matches();

    let config = load_config()?;
    let client = Client::new(&config.host, &config.token, &config.key);

    if let Some(matches) = matches.subcommand_matches("board") {
        board_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("card") {
        card_subcommand(&client, &matches)?;
    }
    Ok(())
}

fn load_config() -> Result<Config, Box<dyn Error>> {
    let mut config_path = dirs::config_dir().expect("Unable to determine config directory");
    config_path.push("tro/config.toml");

    let contents = fs::read_to_string(config_path.to_str().unwrap())?;

    Ok(toml::from_str(&contents)?)
}

fn card_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    if let Some(matches) = matches.subcommand_matches("get") {
        if let Some(card_name) = matches.value_of("name") {
            if let Some(card) = get_card_by_name(&client, card_name)? {
                render_card(&card);
            }
        }
    }
    Ok(())
}

fn board_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    if let Some(matches) = matches.subcommand_matches("get") {
        if let Some(board_name) = matches.value_of("name") {
            if let Some(board) = get_board_by_name(&client, board_name)? {
                let lists = Board::get_all_lists(client, &board.id)?;

                render_board(&board);
                println!("");

                for list in lists {
                    render_list(&list);
                    println!("");
                }
            } else {
                println!("Could not find target board: '{}'", board_name);
            }
        } else {
            println!("You must specify a filter");
        }
    }
    Ok(())
}

// TODO Consider making this a trait for each Trello struct
fn render_board(board: &Board) {
    println!("{}", board.name);
    println!("===");
}

fn render_list(list: &List) {
    println!("{}", list.name);
    println!("---");

    if let Some(cards) = &list.cards {
        for card in cards {
            render_card(&card);
        }
    }
}

fn render_card(card: &Card) {
    println!("{}", card.name);
}


fn get_card_by_name(client: &Client, name: &str) -> Result<Option<Card>, Box<dyn Error>> {
    Ok(None)
}


fn get_board_by_name(client: &Client, name: &str) -> Result<Option<Board>, Box<dyn Error>> {
    let boards = Board::get_all(&client)?;

    for board in boards {
        if board.name == name {
            return Ok(Some(board));
        }
    }
    Ok(None)
}
