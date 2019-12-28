#[macro_use]
extern crate clap;
#[macro_use]
extern crate simple_error;
#[macro_use]
extern crate log;
extern crate simplelog;

#[cfg(test)]
mod test_main;

use clap::ArgMatches;
use regex::RegexBuilder;
use serde::Deserialize;
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};
use std::error::Error;
use std::fs;
use trello::{Board, Card, Client, List, TrelloObject};

#[derive(Deserialize, Debug)]
struct TrelloConfig {
    host: String,
    token: String,
    key: String,
}

// TODO: Better caching between subcommands (i.e. dont re-retrieve data)
// TODO: Better render for "get" subcommands. Make render a trait method?

fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap_app!(myapp =>
        (version: "1.0")
        (author: "Michael Aquilina")
        (about: "Trello CLI interface")
        (@arg log_level: -l --("log-level") +takes_value default_value[ERROR] "Specify the log level")
        (@subcommand show =>
            (about: "Shortcut subcommand to show object contents")
            (@arg board_name: +required "Board Name to retrieve")
            (@arg list_name: !required "List Name to retrieve")
            (@arg card_name: !required "Card Name to retrieve")
            (@arg ignore_case: -i --("ignore-case") "Ignore case when searching")
        )
        (@subcommand close =>
            (about: "Shortcut subcommand to close objects")
            (@arg board_name: +required "Board Name to retrieve")
            (@arg list_name: !required "List Name to retrieve")
            (@arg card_name: !required "Card Name to retrieve")
            (@arg ignore_case: -i --("ignore-case") "Ignore case when searching")
        )
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
                (@arg name: +takes_value "Specify board by name. Supports regex patterns.")
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
                        (@arg name: +takes_value "Specify list by name. Supports regex patterns.")
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
                                (@arg name: +takes_value "Specify card by name. Supports regex patterns.")
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

    let log_level = match matches
        .value_of("log_level")
        .unwrap()
        .to_uppercase()
        .as_str()
    {
        "TRACE" => LevelFilter::Trace,
        "DEBUG" => LevelFilter::Debug,
        "INFO" => LevelFilter::Info,
        "ERROR" => LevelFilter::Error,
        unknown => panic!("Unknown log level '{}'", unknown),
    };

    CombinedLogger::init(vec![TermLogger::new(
        log_level,
        Config::default(),
        TerminalMode::Mixed,
    )
    .unwrap()])
    .unwrap();

    let config = load_config()?;
    let client = Client::new(&config.host, &config.token, &config.key);

    debug!("Loaded configuration: {:?}", config);

    if let Some(matches) = matches.subcommand_matches("boards") {
        board_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("show") {
        show_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("close") {
        close_subcommand(&client, &matches)?;
    } else {
        println!("{}", matches.usage());
    }
    Ok(())
}

fn load_config() -> Result<TrelloConfig, Box<dyn Error>> {
    let mut config_path = dirs::config_dir().expect("Unable to determine config directory");
    config_path.push("tro/config.toml");

    let contents = fs::read_to_string(config_path.to_str().unwrap())?;

    Ok(toml::from_str(&contents)?)
}

struct TrelloResult {
    board: Option<Board>,
    list: Option<List>,
    card: Option<Card>,
}

fn get_trello_object(
    client: &Client,
    matches: &ArgMatches,
) -> Result<TrelloResult, Box<dyn Error>> {
    let board_name = matches.value_of("board_name").unwrap();
    let boards = Board::get_all(&client)?;
    let ignore_case = matches.is_present("ignore_case");

    if let Some(board) = get_object_by_name(boards, &board_name, ignore_case)? {
        if let Some(list_name) = matches.value_of("list_name") {
            let lists = Board::get_all_lists(client, &board.id, true)?;
            if let Some(list) = get_object_by_name(lists, &list_name, ignore_case)? {
                if let Some(card_name) = matches.value_of("card_name") {
                    let cards = List::get_all_cards(client, &list.id)?;

                    if let Some(card) = get_object_by_name(cards, &card_name, ignore_case)? {
                        return Ok(TrelloResult {
                            board: Some(board),
                            list: Some(list),
                            card: Some(card),
                        });
                    } else {
                        bail!("Card not found, specify a more precise filter");
                    }
                } else {
                    return Ok(TrelloResult {
                        board: Some(board),
                        list: Some(list),
                        card: None,
                    });
                }
            } else {
                bail!("List not found, specify a more precise filter");
            }
        } else {
            return Ok(TrelloResult {
                board: Some(board),
                list: None,
                card: None,
            });
        }
    } else {
        bail!("Board not found, specify a more precise filter");
    }
}

fn show_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    debug!("Running show subcommand with {:?}", matches);

    let result = get_trello_object(client, matches)?;

    if let Some(card) = result.card {
        println!("{}", card.render());
    } else if let Some(list) = result.list {
        println!("{}", list.render());
    } else if let Some(board) = result.board {
        println!("{}", board.render());
    }

    Ok(())
}

fn close_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    debug!("Running close subcommand with {:?}", matches);

    let result = get_trello_object(client, matches)?;

    if let Some(mut card) = result.card {
        card.closed = true;
        Card::update(client, &card)?;
    } else if let Some(mut list) = result.list {
        list.closed = true;
        List::update(client, &list)?;
    } else if let Some(mut board) = result.board {
        board.closed = true;
        Board::update(client, &board)?;
    }

    Ok(())
}

fn card_subcommand(
    client: &Client,
    matches: &ArgMatches,
    list_id: &str,
) -> Result<(), Box<dyn Error>> {
    debug!("Running card subcommand with {:?}", matches);

    if let Some(matches) = matches.subcommand_matches("get") {
        if let Some(card_name) = matches.value_of("name") {
            let ignore_case = matches.is_present("ignore_case");
            let cards = List::get_all_cards(&client, list_id)?;

            if let Some(mut card) = get_object_by_name(cards, card_name, ignore_case)? {
                if matches.subcommand_matches("close").is_some() {
                    card.closed = true;
                    Card::update(client, &card)?;
                    println!("Closed card {} with id {}", card.name, card.id);
                } else {
                    println!("{}", card.render());
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
    debug!("Running list subcommand with {:?}", matches);

    if let Some(matches) = matches.subcommand_matches("get") {
        if let Some(list_name) = matches.value_of("name") {
            let ignore_case = matches.is_present("ignore_case");
            let lists = Board::get_all_lists(&client, board_id, true)?;

            if let Some(mut list) = get_object_by_name(lists, list_name, ignore_case)? {
                if let Some(matches) = matches.subcommand_matches("cards") {
                    card_subcommand(client, matches, &list.id)?;
                } else if matches.subcommand_matches("close").is_some() {
                    list.closed = true;
                    List::update(client, &list)?;
                    println!("Closed list {} with id {}", list.name, list.id);
                } else {
                    println!("{}", list.render());
                }
            } else {
                println!("Could not find a list with the name: {}", list_name);
            }
        } else {
            println!("Must specify a filter to target list");
        }
    } else if matches.subcommand_matches("ls").is_some() {
        let lists = Board::get_all_lists(client, board_id, true)?;
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
    debug!("Running board subcommand with {:?}", matches);

    if let Some(matches) = matches.subcommand_matches("get") {
        if let Some(board_name) = matches.value_of("name") {
            let ignore_case = matches.is_present("ignore_case");
            let boards = Board::get_all(&client)?;

            if let Some(mut board) = get_object_by_name(boards, board_name, ignore_case)? {
                if let Some(matches) = matches.subcommand_matches("lists") {
                    list_subcommand(client, matches, &board.id)?;
                } else if matches.subcommand_matches("close").is_some() {
                    board.closed = true;
                    Board::update(client, &board)?;
                    println!("Closed board {} with id {}", board.id, board.name);
                } else {
                    println!("{}", board.render());
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

fn get_object_by_name<T: TrelloObject>(
    boards: Vec<T>,
    name: &str,
    ignore_case: bool,
) -> Result<Option<T>, simple_error::SimpleError> {
    let re = RegexBuilder::new(name)
        .case_insensitive(ignore_case)
        .build()
        .expect("Invalid Regex");

    let mut boards = boards
        .into_iter()
        .filter(|b| re.is_match(&b.get_name()))
        .collect::<Vec<T>>();

    if boards.len() == 1 {
        Ok(boards.pop())
    } else if boards.len() > 1 {
        bail!(
            "More than one object found for '{}'. Specify a more precise filter",
            name
        );
    } else {
        Ok(None)
    }
}
