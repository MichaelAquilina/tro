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
use colored::*;
use regex::RegexBuilder;
use serde::Deserialize;
use simple_error::SimpleError;
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};
use std::error::Error;
use std::io::{stdin, Read, Write};
use std::process;
use std::{env, fs};
use tempfile::Builder;
use trello::{Board, Card, Client, List, TrelloObject};

const CARD_DESCRIPTION_PLACEHOLDER: &str = "<Enter Description Here>";
const CARD_NAME_PLACEHOLDER: &str = "<Enter Card Name Here>";

#[derive(Deserialize, Debug)]
struct TrelloConfig {
    host: String,
    token: String,
    key: String,
}

// TODO: Move usage documentation to this file so that it can be doctested
// TODO: Upload card changes on editor write rather than close
// TODO: move command (move a card within the same list, to another list etc...)
// TODO: re-open command (in case something was closed by mistake)
// TODO: Edit Labels in show card
// TODO: Tests for all the subcommands
// TODO: Unified/Streamlined CLI interface
// TODO: Better Trello API interface
fn main() {
    if let Err(error) = start() {
        eprintln!("An Error occurred:");
        if let Some(error) = error.source() {
            eprintln!("{}", error);
        } else {
            eprintln!("{}", error);
        }
        process::exit(2);
    }
}

fn start() -> Result<(), Box<dyn Error>> {
    let matches = clap_app!(TrelloCLI =>
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
        (@arg log_level: -l --("log-level") +takes_value default_value[ERROR] "Specify the log level")
        (@arg ignore_case: -c --("case-sensitive") "Use case sensitive names when searching")
        (@subcommand show =>
            (about: "Show object contents")
            (@arg board_name: !required "Board Name to retrieve")
            (@arg list_name: !required "List Name to retrieve")
            (@arg card_name: !required "Card Name to retrieve")
            (@arg new: -n --new requires("list_name") conflicts_with("card_name") "Create new Card")
            (@arg label_filter: -f --filter +takes_value "Filter by label")
        )
        (@subcommand url =>
            (about: "Display object url")
            (@arg board_name: !required "Board Name to retrieve")
            (@arg list_name: !required "List Name to retrieve")
            (@arg card_name: !required "Card Name to retrieve")
        )
        (@subcommand close =>
            (about: "Close objects")
            (@arg board_name: +required "Board Name to retrieve")
            (@arg list_name: !required "List Name to retrieve")
            (@arg card_name: !required "Card Name to retrieve")
        )
        (@subcommand create =>
            (about: "Create objects")
            (@arg board_name: !required "Board Name to retrieve")
            (@arg list_name: !required "List Name to retrieve")
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
        unknown => bail!("Unknown log level '{}'", unknown),
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
    } else if let Some(matches) = matches.subcommand_matches("url") {
        url_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("close") {
        close_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("create") {
        create_subcommand(&client, &matches)?;
    } else {
        println!("{}", matches.usage());
    }
    Ok(())
}

fn load_config() -> Result<TrelloConfig, Box<dyn Error>> {
    let mut config_path = dirs::config_dir().expect("Unable to determine config directory");
    config_path.push("tro/config.toml");

    debug!("Loading configuration from {:?}", config_path);
    let contents = fs::read_to_string(config_path.to_str().unwrap())?;

    Ok(toml::from_str(&contents)?)
}

#[derive(Debug, PartialEq)]
struct TrelloResult {
    board: Option<Board>,
    list: Option<List>,
    card: Option<Card>,
}

#[derive(Debug, PartialEq)]
struct TrelloParams<'a> {
    board_name: Option<&'a str>,
    list_name: Option<&'a str>,
    card_name: Option<&'a str>,
    ignore_case: bool,
}

fn get_trello_params<'a>(matches: &'a ArgMatches) -> TrelloParams<'a> {
    TrelloParams {
        board_name: matches.value_of("board_name"),
        list_name: matches.value_of("list_name"),
        card_name: matches.value_of("card_name"),
        ignore_case: !matches.is_present("case_sensitive"),
    }
}

fn get_trello_object(
    client: &Client,
    params: &TrelloParams,
) -> Result<TrelloResult, Box<dyn Error>> {
    let board_name = match params.board_name {
        Some(bn) => bn,
        None => {
            return Ok(TrelloResult {
                board: None,
                list: None,
                card: None,
            })
        }
    };
    let boards = Board::get_all(&client)?;
    let mut board = get_object_by_name(&boards, &board_name, params.ignore_case)?.clone();

    // This should retrieve everything at once
    // This means better performance as it's less HTTP requests. But it does
    // mean we might retrieve more than we actually need in memory.
    board.retrieve_nested(client)?;

    if let Some("-") = params.list_name {
        if let Some(card_name) = params.card_name {
            let lists = board.lists.as_ref().unwrap();

            let card = get_card_from_lists(&lists, &card_name, params.ignore_case)?;
            return Ok(TrelloResult {
                board: Some(board.clone()),
                list: None,
                card: Some(card.clone()),
            });
        } else {
            bail!("Card name must be specified with list '-' wildcard");
        }
    } else if let Some(list_name) = params.list_name {
        let lists = &board.lists.as_ref().unwrap();
        let list = get_object_by_name(lists, &list_name, params.ignore_case)?.clone();

        if let Some(card_name) = params.card_name {
            let cards = &list.cards.as_ref().unwrap();

            let card = get_object_by_name(&cards, &card_name, params.ignore_case)?.clone();
            return Ok(TrelloResult {
                board: Some(board),
                list: Some(list),
                card: Some(card),
            });
        } else {
            return Ok(TrelloResult {
                board: Some(board),
                list: Some(list),
                card: None,
            });
        }
    } else {
        return Ok(TrelloResult {
            board: Some(board),
            list: None,
            card: None,
        });
    }
}

/// Opens the users chosen editor (specified by the $EDITOR environment variable)
/// to edit a specified card. If $EDITOR is not set, the default editor will fallback
/// to vi.
///
/// Once the editor is closed, a new card is populated and returned based on the
/// contents of what was written by the editor.
fn edit_card(card: &mut Card) -> Result<(), Box<dyn Error>> {
    let mut file = Builder::new().suffix(".md").tempfile()?;
    let editor_env = env::var("EDITOR").unwrap_or(String::from("vi"));

    debug!("Using editor: {}", editor_env);
    debug!("Editing card: {:?}", card);

    writeln!(file, "{}", card.render())?;

    let editor = process::Command::new(editor_env)
        .arg(file.path())
        .status()?;

    debug!("editor exited with {:?}", editor);

    let mut buf = String::new();
    file.reopen()?.read_to_string(&mut buf)?;

    // Trim end because a lot of editors will auto add new lines at the end of the file
    let card_contents = Card::parse(buf.trim_end())?;
    card.name = card_contents.name;
    card.desc = card_contents.desc;

    debug!("New card: {:?}", card);

    Ok(())
}

fn get_input(text: &str) -> Result<String, Box<dyn Error>> {
    eprint!("{}", text);

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    Ok(String::from(input.trim_end()))
}

fn url_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    debug!("Running url subcommand with {:?}", matches);

    let params = get_trello_params(matches);
    let result = get_trello_object(client, &params)?;

    if let Some(card) = result.card {
        println!("{}", card.url);
    } else if result.list.is_some() {
        // Lists do not have a target url
        // We can display the parent board url instead
        println!("{}", result.board.unwrap().url);
    } else if let Some(board) = result.board {
        println!("{}", board.url);
    }
    Ok(())
}

fn show_card(client: &Client, card: &Card, list_id: &str) -> Result<(), Box<dyn Error>> {
    let mut new_card = card.clone();
    let is_new_card = new_card.id == "";

    loop {
        edit_card(&mut new_card)?;

        if &new_card == card {
            // no changes detected
            return Ok(());
        }

        // if nothing is edited by the user, remove it
        if new_card.desc == CARD_DESCRIPTION_PLACEHOLDER {
            new_card.desc = String::from("");
        }

        if new_card.name != CARD_NAME_PLACEHOLDER {
            let result = if is_new_card {
                Card::create(client, list_id, &new_card)
            } else {
                Card::update(client, &new_card)
            };

            match result {
                Err(e) => {
                    eprintln!("An error occurred. Press enter to retry");
                    get_input(&e.source().unwrap().to_string())?;
                }
                Ok(card) => {
                    eprintln!("'{}'", new_card.name.green());
                    eprintln!("id: {}", new_card.id);
                    break;
                }
            }
        } else {
            eprintln!("Card name not entered. Aborting.");
            break;
        }
    }
    Ok(())
}

fn show_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    debug!("Running show subcommand with {:?}", matches);

    let label_filter = matches.value_of("label_filter");

    let params = get_trello_params(matches);
    let result = get_trello_object(client, &params)?;

    trace!("result: {:?}", result);

    // TODO: Upload data every time the editor saves the file
    // rather than just when it is closed

    if matches.is_present("new") {
        let card = Card::new(
            "",
            CARD_NAME_PLACEHOLDER,
            CARD_DESCRIPTION_PLACEHOLDER,
            None,
            "",
        );
        // we can safely unwrap the list due to the way we've setup clap
        let list_id = &result.list.unwrap().id;

        show_card(client, &card, list_id)?;
    } else {
        if let Some(card) = result.card {
            show_card(client, &card, "")?;
        } else if let Some(list) = result.list {
            let list = match label_filter {
                Some(label_filter) => list.filter(label_filter),
                None => list,
            };
            println!("{}", list.render());
        } else if let Some(mut board) = result.board {
            board.retrieve_nested(client)?;
            let board = match label_filter {
                Some(label_filter) => board.filter(label_filter),
                None => board,
            };

            println!("{}", board.render());
        } else {
            println!("Open Boards");
            println!("===========");
            println!();

            let boards = Board::get_all(client)?;
            for b in boards {
                println!("* {}", b.name);
            }
        }
    }

    Ok(())
}

fn close_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    debug!("Running close subcommand with {:?}", matches);

    let params = get_trello_params(matches);
    let result = get_trello_object(client, &params)?;

    trace!("result: {:?}", result);

    if let Some(mut card) = result.card {
        card.closed = true;
        Card::update(client, &card)?;
        eprintln!("Closed card '{}'", &card.name.green());
        eprintln!("id: {}", &card.id);
    } else if let Some(mut list) = result.list {
        list.closed = true;
        List::update(client, &list)?;
        eprintln!("Closed list '{}'", &list.name.green());
        eprintln!("id: {}", &list.id);
    } else if let Some(mut board) = result.board {
        board.closed = true;
        Board::update(client, &board)?;
        eprintln!("Closed board '{}'", &board.name.green());
        eprintln!("id: {}", &board.id);
    }

    Ok(())
}

fn create_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    debug!("Running create subcommand with {:?}", matches);

    let params = get_trello_params(matches);
    let result = get_trello_object(client, &params)?;

    trace!("result: {:?}", result);

    if let Some(list) = result.list {
        let name = get_input("Card name: ")?;

        Card::create(client, &list.id, &Card::new("", &name, "", None, ""))?;
    } else if let Some(board) = result.board {
        let name = get_input("List name: ")?;

        List::create(client, &board.id, &name)?;
    } else {
        let name = get_input("Board name: ")?;

        Board::create(client, &name)?;
    }

    Ok(())
}

/// TODO: Find a way to make this generic
/// Retrieves a card by name from a collection of lists.
/// This is different from get_object_by_name which only
/// retrieves a single object from a single collection
fn get_card_from_lists<'a>(
    lists: &'a Vec<List>,
    card_name: &str,
    ignore_case: bool,
) -> Result<&'a Card, SimpleError> {
    let re = RegexBuilder::new(card_name)
        .case_insensitive(ignore_case)
        .build()
        .expect("Invalid Regex");

    let mut result = vec![];
    for list in lists {
        let cards = list
            .cards
            .as_ref()
            .unwrap()
            .iter()
            .filter(|c| re.is_match(&c.name))
            .collect::<Vec<&Card>>();
        result.extend(cards);
    }

    if result.len() == 1 {
        return Ok(result.pop().unwrap());
    } else if result.len() > 1 {
        bail!(
            "Multiple cards found. Specify a more precise filter than '{}' (Found {})",
            card_name,
            result
                .iter()
                .map(|c| format!("'{}'", c.get_name()))
                .collect::<Vec<String>>()
                .join(", ")
        );
    } else {
        bail!(
            "Card not found. Specify a more precise filter than '{}'",
            card_name
        );
    }
}

fn get_object_by_name<'a, T: TrelloObject>(
    objects: &'a Vec<T>,
    name: &str,
    ignore_case: bool,
) -> Result<&'a T, SimpleError> {
    let re = RegexBuilder::new(name)
        .case_insensitive(ignore_case)
        .build()
        .expect("Invalid Regex");

    let mut objects = objects
        .into_iter()
        .filter(|o| re.is_match(&o.get_name()))
        .collect::<Vec<&T>>();

    if objects.len() == 1 {
        Ok(objects.pop().unwrap())
    } else if objects.len() > 1 {
        bail!(
            "More than one {} found. Specify a more precise filter than '{}' (Found {})",
            T::get_type(),
            name,
            objects
                .iter()
                .map(|t| format!("'{}'", t.get_name()))
                .collect::<Vec<String>>()
                .join(", ")
        );
    } else {
        bail!(
            "{} not found. Specify a more precise filter than '{}'",
            T::get_type(),
            name
        );
    }
}
