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

    if let Some(matches) = matches.subcommand_matches("show") {
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
