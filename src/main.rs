// I personally find the return syntax a lot more visually obvious
// when scanning code
#![allow(clippy::needless_return)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate simple_error;
#[macro_use]
extern crate log;
extern crate simplelog;

#[cfg(test)]
mod test_find;

mod find;
mod subcommands;

use serde::Deserialize;
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};
use std::error::Error;
use std::process;
use std::{env, fs};
use trello::Client;

#[derive(Deserialize, Debug)]
struct TrelloConfig {
    host: String,
    token: String,
    key: String,
}

// TODO: Label card on creation
// TODO: Enable truecolor support for labels
// TODO: Move usage documentation to this file so that it can be doctested
// TODO: move command (move a card within the same list, to another list etc...)
// TODO: Tests for all the subcommands
// TODO: Unified/Streamlined CLI interface
// TODO: Better Trello API interface
fn main() {
    if let Err(error) = start() {
        eprintln!("An Error occurred:");
        if let Some(error) = error.source() {
            eprintln!("{}", error.description());
            debug!("{:?}", error);
        } else {
            eprintln!("{}", error.description());
            debug!("{:?}", error);
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
        (@subcommand search =>
            (about: "Search Trello cards and boards")
            (@arg query: +required "Trello Query String")
            (@arg partial: -p --partial "Allow partial matches")
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
