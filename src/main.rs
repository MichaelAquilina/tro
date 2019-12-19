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
                (@subcommand list =>
                    (about: "Interact with board lists")
                    (@arg name: -n --name +takes_value "specify list name")
                    (@subcommand card =>
                        (about: "Interact with list cards")
                        (@arg name: -n --name +takes_value "specify card name")
                    )
                )
            )
        )

    )
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .get_matches();

    let config = load_config()?;
    let client = Client::new(&config.host, &config.token, &config.key);

    if let Some(matches) = matches.subcommand_matches("board") {
        board_subcommand(&client, &matches)?;
    }
    Ok(())
}

fn load_config() -> Result<Config, Box<dyn Error>> {
    let mut config_path = dirs::config_dir().expect("Unable to determine config directory");
    config_path.push("tro/config.toml");

    let contents = fs::read_to_string(config_path.to_str().unwrap())?;

    Ok(toml::from_str(&contents)?)
}

fn card_subcommand(client: &Client, matches: &ArgMatches, list_id: &str) -> Result<(), Box<dyn Error>> {
    if let Some(card_name) = matches.value_of("name") {
        if let Some(card) = get_card_by_name(&client, list_id, card_name)? {
            render_card(&card, true);
        } else {
            println!("Could not find a card with the name: {}", card_name);
        }
    }
    Ok(())
}


fn list_subcommand(client: &Client, matches: &ArgMatches, board_id: &str) -> Result<(), Box<dyn Error>> {
    if let Some(list_name) = matches.value_of("name") {
        if let Some(list) = get_list_by_name(&client, board_id, list_name)? {

            if let Some(matches) = matches.subcommand_matches("card") {
                card_subcommand(client, matches, &list.id)?;
            } else {
                render_list(&list);
            }
        } else {
            println!("Could not find a list with the name: {}", list_name);
        }
    }
    Ok(())
}

fn board_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    if let Some(matches) = matches.subcommand_matches("get") {
        if let Some(board_name) = matches.value_of("name") {
            if let Some(board) = get_board_by_name(&client, board_name)? {

                if let Some(matches) = matches.subcommand_matches("list") {
                    list_subcommand(client, matches, &board.id)?;
                } else {
                    let lists = Board::get_all_lists(client, &board.id)?;

                    render_board(&board);
                    println!("");

                    for list in lists {
                        render_list(&list);
                        println!("");
                    }
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
            render_card(&card, false);
        }
    }
}

fn render_card(card: &Card, detail: bool) {
    println!("{}", card.name);

    if detail {
        println!("---");
        println!("{}", card.desc);
    }
}

fn get_card_by_name(client: &Client, list_id: &str, name: &str) -> Result<Option<Card>, Box<dyn Error>> {
    let cards = List::get_all_cards(client, list_id)?;

    for card in cards {
        if card.name == name {
            return Ok(Some(card));
        }
    }
    Ok(None)
}

fn get_list_by_name(client: &Client, board_id: &str, name: &str) -> Result<Option<List>, Box<dyn Error>> {
    let lists = Board::get_all_lists(client, board_id)?;

    for list in lists {
        if list.name == name {
            return Ok(Some(list));
        }
    }
    Ok(None)
}


fn get_board_by_name(client: &Client, name: &str) -> Result<Option<Board>, Box<dyn Error>> {
    let boards = Board::get_all(client)?;

    for board in boards {
        if board.name == name {
            return Ok(Some(board));
        }
    }
    Ok(None)
}
