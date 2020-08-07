// I personally find the return syntax a lot more visually obvious
// when scanning code
#![allow(clippy::needless_return)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

#[cfg(test)]
mod test_find;

mod cli;
mod find;
mod subcommands;

use colored::*;
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};
use std::env;
use std::error::Error;
use std::process;
use trello::{ClientConfig, TrelloClient};

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
    let matches = clap_app!(tro =>
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
        (@arg log_level: -l --("log-level") +takes_value possible_values(&["TRACE", "DEBUG", "INFO", "WARN", "ERROR"]) default_value[ERROR] "Specify the log level")
        (@subcommand version =>
            (about: "Print tro version")
        )
        (@subcommand setup =>
            (about: "Setup tro")
        )
        (@subcommand me =>
            (about: "Show currently logged in user")
            (@arg detailed: -d --detailed "Display detailed information")
        )
        (@subcommand show =>
            (about: "Show object contents")
            (@arg board_name: !required "Board Name to retrieve")
            (@arg list_name: !required "List Name to retrieve")
            (@arg card_name: !required "Card Name to retrieve")
            (@arg case_sensitive: -c --("case-sensitive") "Use case sensitive names when searching")
            (@arg label_filter: -f --filter +takes_value "Filter by label")
            (@arg interactive: -i --interactive "Enables interactive mode")
        )
        (@subcommand search =>
            (about: "Search Trello cards")
            (long_about: "
Searches Trello cards.
See the link below for details about how to write queries when searching with Trello.
https://help.trello.com/article/808-searching-for-cards-all-boards")
            (@arg query: +required +multiple "Trello Query String")
            (@arg partial: -p --partial "Allow partial matches")
            (@arg cards_limit: --limit +takes_value "Specify the max number of cards to return")
            (@arg interactive: -i --interactive "Enables interactive mode")
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
            (@arg label_name: required_unless("interactive") +multiple "Label name to apply")
            (@arg delete: -d --delete conflicts_with("interactive") "Delete specified label")
            (@arg case_sensitive: -c --("case-sensitive") "Use case sensitive names when searching")
            (@arg interactive: -i --interactive "Enables interactive mode")
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
            (@arg board_name: required_unless("interactive") "Board Name to retrieve")
            (@arg list_name: !required "List Name to retrieve")
            (@arg card_name: !required "Card Name to retrieve")
            (@arg case_sensitive: -c --("case-sensitive") "Use case sensitive names when searching")
            (@arg interactive: -i --interactive "Enables interactive mode")
        )
        (@subcommand create =>
            (about: "Create objects")
            (@arg board_name: !required "Board Name to retrieve")
            (@arg list_name: !required "List Name to retrieve")
            (@arg case_sensitive: -c --("case-sensitive") "Use case sensitive names when searching")
            (@arg show: --show -s "Show the item once created")
            (@arg label: --label -l +takes_value +multiple "Apply labels to card on creation")
            (@arg name: +takes_value --name -n "Specify the name of the object being created without a prompt")
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
        unknown => panic!("Unknown log level '{}'", unknown),
    };

    CombinedLogger::init(vec![TermLogger::new(
        log_level,
        Config::default(),
        TerminalMode::Mixed,
    )
    .unwrap()])
    .unwrap();

    // Escape code to re-show the cursor in case
    // ctrl-c was pressed during an interactive prompt
    // where the cursor is temporarily hidden
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
        subcommands::me_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("show") {
        subcommands::show_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("search") {
        subcommands::search_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("attach") {
        subcommands::attach_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("attachments") {
        subcommands::attachments_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("label") {
        subcommands::label_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("url") {
        subcommands::url_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("close") {
        subcommands::close_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("open") {
        subcommands::open_subcommand(&client, &matches)?;
    } else if let Some(matches) = matches.subcommand_matches("create") {
        subcommands::create_subcommand(&client, &matches)?;
    } else {
        println!("{}", matches.usage());
    }
    Ok(())
}
