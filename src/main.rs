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
use std::io::{Read, Write};
use std::process;
use std::{env, fs};
use std::{thread, time};
use tempfile::Builder;
use trello::{Attachment, Board, Card, Client, Label, List, TrelloError, TrelloObject};

#[derive(Deserialize, Debug)]
struct TrelloConfig {
    host: String,
    token: String,
    key: String,
}

// TODO: Search all (including archived) cards
// TODO: Label card on creation
// TODO: Enable truecolor support for labels
// TODO: Move usage documentation to this file so that it can be doctested
// TODO: move command (move a card within the same list, to another list etc...)
// TODO: re-open command (in case something was closed by mistake)
// TODO: Tests for all the subcommands
// TODO: Unified/Streamlined CLI interface
// TODO: Better Trello API interface
fn main() {
    if let Err(error) = start() {
        eprintln!("An Error occurred:");
        if let Some(error) = error.source() {
            eprintln!("{}", error.description());
        } else {
            eprintln!("{}", error.description());
        }
        process::exit(2);
    }
}

fn start() -> Result<(), Box<dyn Error>> {
    let matches = clap_app!(TrelloCLI =>
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
        (@arg log_level: -l --("log-level") +takes_value possible_values(&["TRACE", "DEBUG", "INFO", "WARN", "ERROR"]) default_value[ERROR] "Specify the log level")
        (@subcommand version =>
            (about: "Print TrelloCLI version")
        )
        (@subcommand show =>
            (about: "Show object contents")
            (@arg board_name: !required "Board Name to retrieve")
            (@arg list_name: !required "List Name to retrieve")
            (@arg card_name: !required "Card Name to retrieve")
            (@arg case_sensitive: -c --("case-sensitive") "Use case sensitive names when searching")
            (@arg label_filter: -f --filter +takes_value "Filter by label")
        )
        (@subcommand attach =>
            (about: "Attach a file to a card")
            (@arg board_name: +required "Board name to retrieve")
            (@arg list_name: +required "List name to retrieve")
            (@arg card_name: +required "Card name to retrieve")
            (@arg case_sensitive: -c --("case-sensitive") "Use case sensitive names when searching")
            (@arg path: +required "Path of file to upload")
        )
        (@subcommand attachments =>
            (about: "View attachments")
            (@arg board_name: +required "Board name to retrieve")
            (@arg list_name: +required "List name to retrieve")
            (@arg card_name: +required "Card name to retrieve")
            (@arg case_sensitive: -c --("case-sensitive") "Use case sensitive names when searching")
        )
        (@subcommand label =>
            (about: "Apply or remove a label on a card")
            (@arg board_name: +required "Board name to retrieve")
            (@arg list_name: +required "List name to retrieve")
            (@arg card_name: +required "Card name to retrieve")
            (@arg label_name: +required "Label name to apply")
            (@arg delete: -d --delete "Delete specified label")
            (@arg case_sensitive: -c --("case-sensitive") "Use case sensitive names when searching")
        )
        (@subcommand url =>
            (about: "Display object url")
            (@arg board_name: !required "Board Name to retrieve")
            (@arg list_name: !required "List Name to retrieve")
            (@arg card_name: !required "Card Name to retrieve")
            (@arg case_sensitive: -c --("case-sensitive") "Use case sensitive names when searching")
        )
        (@subcommand open =>
            (about: "Open objects that have been closed")
            (@arg type: +required possible_values(&["board", "list", "card"]) "Type of object")
            (@arg id: +required "Id of the object to re-open")
        )
        (@subcommand close =>
            (about: "Close objects")
            (@arg board_name: +required "Board Name to retrieve")
            (@arg list_name: !required "List Name to retrieve")
            (@arg card_name: !required "Card Name to retrieve")
            (@arg case_sensitive: -c --("case-sensitive") "Use case sensitive names when searching")
            (@arg show: -s --show "Show the board associated with the closed object once done")
        )
        (@subcommand create =>
            (about: "Create objects")
            (@arg board_name: !required "Board Name to retrieve")
            (@arg list_name: !required "List Name to retrieve")
            (@arg case_sensitive: -c --("case-sensitive") "Use case sensitive names when searching")
            (@arg show: --show -s "Show the item once created")
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
        "WARN" => LevelFilter::Warn,
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

    if matches.subcommand_matches("version").is_some() {
        eprintln!(env!("CARGO_PKG_VERSION"));
    } else if let Some(matches) = matches.subcommand_matches("show") {
        show_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("attach") {
        attach_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("attachments") {
        attachments_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("label") {
        label_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("url") {
        url_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("close") {
        close_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("open") {
        open_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("create") {
        create_subcommand(&client, &matches)?;
    } else {
        println!("{}", matches.usage());
    }
    Ok(())
}

fn load_config() -> Result<TrelloConfig, Box<dyn Error>> {
    let mut config_path = dirs::config_dir().ok_or("Unable to determine config directory")?;
    config_path.push("tro/config.toml");

    let path = config_path
        .to_str()
        .ok_or("Could not convert Path to string")?;

    debug!("Loading configuration from {:?}", config_path);
    let contents = fs::read_to_string(path)?;

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
/// This function will upload any changes written by the editor to Trello. This includes
/// when the editor is not closed but content is saved.
fn edit_card(client: &Client, card: &Card) -> Result<(), Box<dyn Error>> {
    let mut file = Builder::new().suffix(".md").tempfile()?;
    let editor_env = env::var("EDITOR").unwrap_or(String::from("vi"));

    debug!("Using editor: {}", editor_env);
    debug!("Editing card: {:?}", card);

    writeln!(file, "{}", card.render())?;

    let mut new_card = card.clone();

    // Outer retry loop - reopen editor if last upload attempt failed
    loop {
        let mut editor = process::Command::new(&editor_env)
            .arg(file.path())
            .spawn()?;
        let mut result: Option<Result<Card, TrelloError>> = None;

        // Inner watch loop - look out for card changes to upload
        loop {
            const SLEEP_TIME: u64 = 500;
            debug!("Sleeping for {}ms", SLEEP_TIME);
            thread::sleep(time::Duration::from_millis(SLEEP_TIME));

            let mut buf = String::new();
            file.reopen()?.read_to_string(&mut buf)?;

            // Trim end because a lot of editors will use auto add new lines at the end of the file
            // FIXME: An error here would break the retry loop completely
            let contents = Card::parse(buf.trim_end())?;

            // if no upload attempts
            // if previous loop had a failure
            // if card in memory is different to card in file
            if result.is_none()
                || result.as_ref().unwrap().is_err()
                || &new_card.name != &contents.name
                || &new_card.desc != &contents.desc
            {
                new_card.name = contents.name;
                new_card.desc = contents.desc;

                debug!("Updating card: {:?}", new_card);
                result = Some(Card::update(client, &new_card));

                match &result {
                    Some(Ok(_)) => debug!("Updated card"),
                    Some(Err(e)) => debug!("Error updating card {:?}", e),
                    None => panic!("This should be impossible"),
                };
            }

            if let Some(ecode) = editor.try_wait()? {
                debug!("Exiting editor loop with code: {}", ecode);
                break;
            }
        }

        match &result {
            None => {
                debug!("Exiting retry loop due to no result being ever retrieved");
                break;
            }
            Some(Ok(_)) => {
                debug!("Exiting retry loop due to successful last update");
                break;
            }
            Some(Err(e)) => {
                eprintln!("An error occurred while trying to update the card.");
                eprintln!("{}", e.description());
                eprintln!();
                get_input("Press entry to re-enter editor")?;
            }
        }
    }

    Ok(())
}

fn get_input(text: &str) -> Result<String, rustyline::error::ReadlineError> {
    let mut rl = rustyline::Editor::<()>::new();
    rl.bind_sequence(
        rustyline::KeyPress::ControlLeft,
        rustyline::Cmd::Move(rustyline::Movement::BackwardWord(1, rustyline::Word::Big)),
    );
    rl.bind_sequence(
        rustyline::KeyPress::ControlRight,
        rustyline::Cmd::Move(rustyline::Movement::ForwardWord(
            1,
            rustyline::At::Start,
            rustyline::Word::Big,
        )),
    );
    rl.readline(text)
}

fn attach_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    debug!("Running attach subcommand with {:?}", matches);

    let params = get_trello_params(matches);
    let result = get_trello_object(client, &params)?;

    let path = matches.value_of("path").unwrap();

    let card = result.card.ok_or("Unable to find card")?;

    let attachment = Attachment::apply(client, &card.id, path)?;

    println!("{}", attachment.render());

    Ok(())
}

fn attachments_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    debug!("Running attachments subcommand with {:?}", matches);

    let params = get_trello_params(matches);
    let result = get_trello_object(client, &params)?;

    let card = result.card.ok_or("Unable to find card")?;

    let attachments = Attachment::get_all(client, &card.id)?;

    for att in attachments {
        println!("{}", &att.url);
    }

    Ok(())
}

fn label_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    debug!("Running label subcommand with {:?}", matches);

    let params = get_trello_params(matches);
    let result = get_trello_object(client, &params)?;

    let labels = Label::get_all(&client, &result.board.ok_or("Unable to retrieve board")?.id)?;
    let card = result.card.ok_or("Unable to find card")?;

    let label_name = matches.value_of("label_name").unwrap();
    let delete = matches.is_present("delete");

    let label = get_object_by_name(&labels, label_name, params.ignore_case)?;
    let card_has_label = card
        .labels
        .ok_or("Unable to retrieve Card labels")?
        .iter()
        .find(|l| &l.id == &label.id)
        .is_some();

    if delete {
        if !card_has_label {
            eprintln!(
                "Label [{}] does not exist on '{}'",
                &label.colored_name(),
                &card.name.green(),
            );
        } else {
            Card::remove_label(client, &card.id, &label.id)?;

            eprintln!(
                "Removed [{}] label from '{}'",
                &label.colored_name(),
                &card.name.green(),
            );
        }
    } else {
        if card_has_label {
            eprintln!(
                "Label [{}] already exists on '{}'",
                &label.colored_name(),
                &card.name.green()
            );
        } else {
            Card::apply_label(client, &card.id, &label.id)?;

            eprintln!(
                "Applied [{}] label to '{}'",
                &label.colored_name(),
                &card.name.green()
            );
        }
    }

    Ok(())
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

fn show_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    debug!("Running show subcommand with {:?}", matches);

    let label_filter = matches.value_of("label_filter");

    let params = get_trello_params(matches);
    debug!("Trello Params: {:?}", params);

    let result = get_trello_object(client, &params)?;
    trace!("result: {:?}", result);

    if let Some(card) = result.card {
        edit_card(client, &card)?;
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

    Ok(())
}

fn close_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    debug!("Running close subcommand with {:?}", matches);

    let params = get_trello_params(matches);
    let result = get_trello_object(client, &params)?;

    let show = matches.is_present("show");

    trace!("result: {:?}", result);

    if let Some(mut card) = result.card {
        card.closed = true;
        Card::update(client, &card)?;

        // FIXME: Bug shows the board with closed card
        if show {
            println!("{}", result.board.unwrap().render());
            println!();
        }

        eprintln!("Closed card: '{}'", &card.name.green());
        eprintln!("id: {}", &card.id);
    } else if let Some(mut list) = result.list {
        list.closed = true;
        List::update(client, &list)?;

        // FIXME: Bug shows the board with the closed list
        if show {
            println!("{}", result.board.unwrap().render());
            println!();
        }

        eprintln!("Closed list: '{}'", &list.name.green());
        eprintln!("id: {}", &list.id);
    } else if let Some(mut board) = result.board {
        board.closed = true;
        Board::update(client, &board)?;
        eprintln!("Closed board: '{}'", &board.name.green());
        eprintln!("id: {}", &board.id);
    }

    Ok(())
}

fn open_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    debug!("Running open subcommand with {:?}", matches);

    let id = matches.value_of("id").ok_or("Id not provided")?;
    let object_type = matches.value_of("type").ok_or("type not provided")?;

    if object_type == "board" {
        debug!("Re-opening board with id {}", &id);
        let board = Board::open(client, &id)?;

        eprintln!("Opened board: {}", &board.name.green());
        eprintln!("id: {}", &board.id);
    } else if object_type == "list" {
        debug!("Re-opening list with id {}", &id);
        let list = List::open(client, &id)?;

        eprintln!("Opened list: {}", &list.name.green());
        eprintln!("id: {}", &list.id);
    } else if object_type == "card" {
        debug!("Re-openning card with id {}", &id);
        let card = Card::open(client, &id)?;

        eprintln!("Opened card: {}", &card.name.green());
        eprintln!("id: {}", &card.id);
    } else {
        bail!("Unknown object_type {}", object_type);
    }

    Ok(())
}

fn create_subcommand(client: &Client, matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    debug!("Running create subcommand with {:?}", matches);

    let params = get_trello_params(matches);
    let result = get_trello_object(client, &params)?;

    let show = matches.is_present("show");

    trace!("result: {:?}", result);

    if let Some(list) = result.list {
        let name = get_input("Card name: ")?;

        let card = Card::create(client, &list.id, &Card::new("", &name, "", None, ""))?;

        if show {
            edit_card(client, &card)?;
        }
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

/// Searches through a collection of Trello objects and tries
/// to match one and only one object to the name pattern provided.
/// * If no matches are found, an Error is returned
/// * If more than match is found, an Error is returned
/// * If only one item is matched, then it is returned
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
