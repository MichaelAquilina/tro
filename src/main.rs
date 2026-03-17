use clap::{builder::PossibleValuesParser, Arg, Command};
#[macro_use]
extern crate log;
use log::debug;
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};
use std::env;
use std::error::Error;
use std::process;
use trello::{ClientConfig, TrelloClient};

use colored::*;

#[cfg(test)]
mod test_find;

mod cli;
mod find;
mod subcommands;

fn main() {
    if let Err(error) = start() {
        eprintln!("An Error occurred:");
        if let Some(error) = error.source() {
            eprintln!("{}", error);
            debug!("{:?}", error);
        } else {
            eprintln!("{}", error);
            debug!("{:?}", error);
        }
        process::exit(2);
    }
}

fn start() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("tro")
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("log-level")
                .short('l')
                .long("log-level")
                .value_parser(PossibleValuesParser::new(["TRACE", "DEBUG", "INFO", "WARN", "ERROR"]))
                .default_value("ERROR")
                .help("Specify the log level"),
        )
        .subcommand(Command::new("version").about("Print tro version"))
        .subcommand(Command::new("setup").about("Setup tro"))
        .subcommand(
            Command::new("me")
                .about("Show currently logged in user")
                .arg(Arg::new("detailed").short('d').long("detailed").help("Display detailed information")),
        )
        .subcommand(
            Command::new("show")
                .about("Show object contents")
                .arg(Arg::new("board_name").help("Board Name to retrieve"))
                .arg(Arg::new("list_name").help("List Name to retrieve"))
                .arg(Arg::new("card_name").help("Card Name to retrieve"))
                .arg(Arg::new("case_sensitive").short('c').long("case-sensitive").help("Use case sensitive names when searching"))
                .arg(Arg::new("label_filter").short('f').long("filter").value_name("LABEL").help("Filter by label"))
                .arg(Arg::new("interactive").short('i').long("interactive").help("Enables interactive mode"))
                .arg(Arg::new("no_headers").long("no-headers").help("Disables displaying headers")),
        )
        .subcommand(
            Command::new("move")
                .about("Move a card to a different list")
                .arg(Arg::new("board_name").required(true).help("Board Name"))
                .arg(Arg::new("list_name").required(true).help("List Name"))
                .arg(Arg::new("card_name").required(true).help("Card Name"))
                .arg(Arg::new("new_list_name").required(true).help("New List Name")),
        )
        .subcommand(
            Command::new("search")
                .about("Search Trello cards")
                .long_about("Searches Trello cards.\nSee the link below for details about how to write queries when searching with Trello.\nhttps://help.trello.com/article/808-searching-for-cards-all-boards")
                .arg(Arg::new("query").required(true).num_args(1..).help("Trello Query String"))
                .arg(Arg::new("partial").short('p').long("partial").help("Allow partial matches"))
                .arg(Arg::new("cards_limit").long("limit").value_name("LIMIT").help("Specify the max number of cards to return"))
                .arg(Arg::new("interactive").short('i').long("interactive").help("Enables interactive mode")),
        )
        .subcommand(
            Command::new("attach")
                .about("Attach a file to a card")
                .arg(Arg::new("board_name").required(true).help("Board name to retrieve"))
                .arg(Arg::new("list_name").required(true).help("List name to retrieve"))
                .arg(Arg::new("card_name").required(true).help("Card name to retrieve"))
                .arg(Arg::new("case_sensitive").short('c').long("case-sensitive").help("Use case sensitive names when searching"))
                .arg(Arg::new("path").required(true).help("Path of file to upload")),
        )
        .subcommand(
            Command::new("attachments")
                .about("View attachments")
                .arg(Arg::new("board_name").required(true).help("Board name to retrieve"))
                .arg(Arg::new("list_name").required(true).help("List name to retrieve"))
                .arg(Arg::new("card_name").required(true).help("Card name to retrieve"))
                .arg(Arg::new("case_sensitive").short('c').long("case-sensitive").help("Use case sensitive names when searching")),
        )
        .subcommand(
            Command::new("label")
                .about("Apply or remove a label on a card")
                .arg(Arg::new("board_name").required(true).help("Board name to retrieve"))
                .arg(Arg::new("list_name").required(true).help("List name to retrieve"))
                .arg(Arg::new("card_name").required(true).help("Card name to retrieve"))
                .arg(Arg::new("label_name").num_args(1..).help("Label name to apply"))
                .arg(Arg::new("delete").short('d').long("delete").help("Delete specified label"))
                .arg(Arg::new("case_sensitive").short('c').long("case-sensitive").help("Use case sensitive names when searching"))
                .arg(Arg::new("interactive").short('i').long("interactive").help("Enables interactive mode"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("url")
                .about("Display object url")
                .arg(Arg::new("board_name").help("Board Name to retrieve"))
                .arg(Arg::new("list_name").help("List Name to retrieve"))
                .arg(Arg::new("card_name").help("Card Name to retrieve"))
                .arg(Arg::new("case_sensitive").short('c').long("case-sensitive").help("Use case sensitive names when searching")),
        )
        .subcommand(
            Command::new("open")
                .about("Open objects that have been closed")
                .arg(Arg::new("type").required(true).value_parser(PossibleValuesParser::new(["board", "list", "card"])).help("Type of object"))
                .arg(Arg::new("id").required(true).help("Id of the object to re-open")),
        )
        .subcommand(
            Command::new("close")
                .about("Close objects")
                .arg(Arg::new("board_name").help("Board Name to retrieve"))
                .arg(Arg::new("list_name").help("List Name to retrieve"))
                .arg(Arg::new("card_name").help("Card Name to retrieve"))
                .arg(Arg::new("case_sensitive").short('c').long("case-sensitive").help("Use case sensitive names when searching"))
                .arg(Arg::new("interactive").short('i').long("interactive").help("Enables interactive mode")),
        )
        .subcommand(
            Command::new("create")
                .about("Create objects")
                .arg(Arg::new("board_name").help("Board Name to retrieve"))
                .arg(Arg::new("list_name").help("List Name to retrieve"))
                .arg(Arg::new("case_sensitive").short('c').long("case-sensitive").help("Use case sensitive names when searching"))
                .arg(Arg::new("show").long("show").short('s').help("Show the item once created"))
                .arg(Arg::new("label").long("label").short('l').num_args(1..).help("Apply labels to card on creation"))
                .arg(Arg::new("name").long("name").short('n').num_args(1..).help("Specify the name of the object being created without a prompt")),
        )
        .arg_required_else_help(true)
        .get_matches();

    let log_level = match matches
        .get_one::<String>("log-level")
        .unwrap()
        .to_uppercase()
        .as_str()
    {
        "TRACE" => LevelFilter::Trace,
        "DEBUG" => LevelFilter::Debug,
        "INFO" => LevelFilter::Info,
        "WARN" => LevelFilter::Warn,
        "ERROR" => LevelFilter::Error,
        unknown => unreachable!("Unknown log level '{}' (this is a clap bug)", unknown),
    };

    let term_logger = TermLogger::new(log_level, Config::default(), TerminalMode::Mixed)
        .ok_or_else(|| std::io::Error::other("Failed to initialize terminal logger"))?;
    CombinedLogger::init(vec![term_logger])?;

    ctrlc::set_handler(|| {
        println!("\x1b[?25h");
        process::exit(2);
    })?;

    if let Some(matches) = matches.subcommand_matches("setup") {
        subcommands::setup_subcommand(matches)?;
        return Ok(());
    }

    let config = match ClientConfig::load_config() {
        Ok(client) => client,
        Err(_) => {
            println!("Unable to load client configuration");
            println!("Please run {}", "tro setup".green());
            return Ok(());
        }
    };
    let client = TrelloClient::new(config);

    debug!("Loaded configuration: {:?}", client);

    if matches.subcommand_matches("version").is_some() {
        eprintln!(env!("CARGO_PKG_VERSION"));
    } else if let Some(matches) = matches.subcommand_matches("me") {
        subcommands::me_subcommand(&client, matches)?;
    } else if let Some(matches) = matches.subcommand_matches("show") {
        subcommands::show_subcommand(&client, matches)?;
    } else if let Some(matches) = matches.subcommand_matches("move") {
        subcommands::move_subcommand(&client, matches)?;
    } else if let Some(matches) = matches.subcommand_matches("search") {
        subcommands::search_subcommand(&client, matches)?;
    } else if let Some(matches) = matches.subcommand_matches("attach") {
        subcommands::attach_subcommand(&client, matches)?;
    } else if let Some(matches) = matches.subcommand_matches("attachments") {
        subcommands::attachments_subcommand(&client, matches)?;
    } else if let Some(matches) = matches.subcommand_matches("label") {
        subcommands::label_subcommand(&client, matches)?;
    } else if let Some(matches) = matches.subcommand_matches("url") {
        subcommands::url_subcommand(&client, matches)?;
    } else if let Some(matches) = matches.subcommand_matches("close") {
        subcommands::close_subcommand(&client, matches)?;
    } else if let Some(matches) = matches.subcommand_matches("open") {
        subcommands::open_subcommand(&client, matches)?;
    } else if let Some(matches) = matches.subcommand_matches("create") {
        subcommands::create_subcommand(&client, matches)?;
    }
    Ok(())
}
