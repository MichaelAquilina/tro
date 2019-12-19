#[macro_use]
extern crate clap;
#[macro_use]
extern crate simple_error;

#[cfg(test)]
mod test_main;

use clap::ArgMatches;
use regex::RegexBuilder;
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
        (@subcommand boards =>
            (about: "Commands related to Trello boards")
            (@subcommand create =>
                (about: "Create a new board")
                (@arg NAME: +required "Name of the board")
            )
            (@subcommand ls =>
                (about: "List all available boards")
            )
            (@subcommand get =>
                (about: "Get details for a specific board")
                (@arg name: -n --name +takes_value "Specify board by name. Supports regex patterns.")
                (@arg ignore_case: -i --("ignore-case") "Ignore case when searching by board name.")
                (@subcommand close =>
                    (about: "Close the board")
                )
                (@subcommand lists =>
                    (about: "Interact with board lists")
                    (@subcommand create =>
                        (about: "Create a new list for this board")
                        (@arg NAME: +required "Name of the list")
                    )
                    (@subcommand ls =>
                        (about: "List all lists in this board")
                    )
                    (@subcommand get =>
                        (about: "Get details for a specific list")
                        (@arg name: -n --name +takes_value "Specify list by name. Supports regex patterns.")
                        (@arg ignore_case: -i --("ignore-case") "Ignore case when searching by list name.")
                        (@subcommand close =>
                            (about: "Close the list")
                        )
                        (@subcommand cards =>
                            (about: "Interact with list cards")
                            (@subcommand create =>
                                (about: "Create a new card for this list")
                                (@arg NAME: +required "Name of the card")
                            )
                            (@subcommand ls =>
                                (about: "List all cards in this list")
                            )
                            (@subcommand get =>
                                (about: "Get details for a specific card")
                                (@arg name: -n --name +takes_value "Specify card by name. Supports regex patterns.")
                                (@arg ignore_case: -i --("ignore-case") "Ignore case when searching by card name.")
                                (@subcommand close =>
                                    (about: "Close the card")
                                )
                            )
                        )
                    )
                )
            )
        )
    )
    .get_matches();

    let config = load_config()?;
    let client = Client::new(&config.host, &config.token, &config.key);

    if let Some(matches) = matches.subcommand_matches("boards") {
        board_subcommand(&client, &matches)?;
    } else {
        println!("{}", matches.usage());
    }
    Ok(())
}

fn load_config() -> Result<Config, Box<dyn Error>> {
    let mut config_path = dirs::config_dir().expect("Unable to determine config directory");
    config_path.push("tro/config.toml");

    let contents = fs::read_to_string(config_path.to_str().unwrap())?;

    Ok(toml::from_str(&contents)?)
}

fn card_subcommand(
    client: &Client,
    matches: &ArgMatches,
    list_id: &str,
) -> Result<(), Box<dyn Error>> {
    if let Some(matches) = matches.subcommand_matches("get") {
        if let Some(card_name) = matches.value_of("name") {
            let ignore_case = matches.is_present("ignore_case");
            if let Some(mut card) = get_card_by_name(&client, list_id, card_name, ignore_case)? {
                if matches.subcommand_matches("close").is_some() {
                    card.closed = true;
                    Card::update(client, &card)?;
                    println!("Closed card {} with id {}", card.name, card.id);
                } else {
                    render_card(&card, true);
                }
            } else {
                println!("Could not find a card with the name: {}", card_name);
            }
        } else {
            println!("Must specify a filter to target card");
        }
    } else if matches.subcommand_matches("ls").is_some() {
        let cards = List::get_all_cards(&client, list_id)?;
        for card in cards {
            println!("{}", card.name);
        }
    } else if let Some(matches) = matches.subcommand_matches("create") {
        let card_name = matches.value_of("NAME").unwrap();
        let card = Card::create(client, list_id, card_name)?;
        println!("{:?}", card);
    } else {
        println!("{}", matches.usage());
    }
    Ok(())
}

fn list_subcommand(
    client: &Client,
    matches: &ArgMatches,
    board_id: &str,
) -> Result<(), Box<dyn Error>> {
    if let Some(matches) = matches.subcommand_matches("get") {
        if let Some(list_name) = matches.value_of("name") {
            let ignore_case = matches.is_present("ignore_case");
            if let Some(mut list) = get_list_by_name(&client, board_id, list_name, ignore_case)? {
                if let Some(matches) = matches.subcommand_matches("cards") {
                    card_subcommand(client, matches, &list.id)?;
                } else if matches.subcommand_matches("close").is_some() {
                    list.closed = true;
                    List::update(client, &list)?;
                    println!("Closed list {} with id {}", list.name, list.id);
                } else {
                    render_list(&list);
                }
            } else {
                println!("Could not find a list with the name: {}", list_name);
            }
        } else {
            println!("Must specify a filter to target list");
        }
    } else if matches.subcommand_matches("ls").is_some() {
        let lists = Board::get_all_lists(client, board_id)?;
        for list in lists {
            println!("{}", list.name);
        }
    } else if let Some(matches) = matches.subcommand_matches("create") {
        let list_name = matches.value_of("NAME").unwrap();
        let list = List::create(client, board_id, list_name)?;
        println!("{:?}", list);
    } else {
        println!("{}", matches.usage());
    }

    Ok(())
}

fn board_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    if let Some(matches) = matches.subcommand_matches("get") {
        if let Some(board_name) = matches.value_of("name") {
            let ignore_case = matches.is_present("ignore_case");

            if let Some(mut board) = get_board_by_name(&client, board_name, ignore_case)? {
                if let Some(matches) = matches.subcommand_matches("lists") {
                    list_subcommand(client, matches, &board.id)?;
                } else if matches.subcommand_matches("close").is_some() {
                    board.closed = true;
                    Board::update(client, &board)?;
                    println!("Closed board {} with id {}", board.id, board.name);
                } else {
                    let lists = Board::get_all_lists(client, &board.id)?;

                    render_board(&board);
                    println!();

                    for list in lists {
                        render_list(&list);
                        println!();
                    }
                }
            } else {
                println!("Could not find target board: '{}'", board_name);
            }
        } else {
            println!("You must specify a filter");
        }
    } else if matches.subcommand_matches("ls").is_some() {
        let boards = Board::get_all(&client)?;
        for board in boards {
            println!("{}", board.name);
        }
    } else if let Some(matches) = matches.subcommand_matches("create") {
        let board_name = matches.value_of("NAME").unwrap();
        let board = Board::create(&client, board_name)?;
        println!("{:?}", board);
    } else {
        println!("{}", matches.usage());
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
        if card.desc.is_empty() {
            println!("<No Description>");
        } else {
            println!("{}", card.desc);
        }
    }
}

fn get_card_by_name(
    client: &Client,
    list_id: &str,
    name: &str,
    ignore_case: bool,
) -> Result<Option<Card>, Box<dyn Error>> {
    let cards = List::get_all_cards(client, list_id)?;

    let re = RegexBuilder::new(name)
        .case_insensitive(ignore_case)
        .build()
        .expect("Invalid Regex");

    let mut cards = cards
        .into_iter()
        .filter(|c| re.is_match(&c.name))
        .collect::<Vec<Card>>();

    if cards.len() == 1 {
        Ok(cards.pop())
    } else if cards.len() > 1 {
        bail!("More than one Card found. Specify a more precise filter.");
    } else {
        Ok(None)
    }
}

fn get_list_by_name(
    client: &Client,
    board_id: &str,
    name: &str,
    ignore_case: bool,
) -> Result<Option<List>, Box<dyn Error>> {
    let lists = Board::get_all_lists(client, board_id)?;

    let re = RegexBuilder::new(name)
        .case_insensitive(ignore_case)
        .build()
        .expect("Invalid Regex");

    let mut lists = lists
        .into_iter()
        .filter(|l| re.is_match(&l.name))
        .collect::<Vec<List>>();

    if lists.len() == 1 {
        Ok(lists.pop())
    } else if lists.len() > 1 {
        bail!("More than one List found. Specify a more precise filter.");
    } else {
        Ok(None)
    }
}

fn get_board_by_name(
    client: &Client,
    name: &str,
    ignore_case: bool,
) -> Result<Option<Board>, Box<dyn Error>> {
    let boards = Board::get_all(client)?;

    let re = RegexBuilder::new(name)
        .case_insensitive(ignore_case)
        .build()
        .expect("Invalid Regex");

    let mut boards = boards
        .into_iter()
        .filter(|b| re.is_match(&b.name))
        .collect::<Vec<Board>>();

    if boards.len() == 1 {
        Ok(boards.pop())
    } else if boards.len() > 1 {
        bail!("More than one Board found. Specify a more precise filter");
    } else {
        Ok(None)
    }
}
