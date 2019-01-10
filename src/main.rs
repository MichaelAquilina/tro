mod trello;

#[macro_use]
extern crate serde_derive;

extern crate clap;
extern crate console;
extern crate indoc;

use clap::{App, Arg, ArgMatches, SubCommand};
use console::{style, StyledObject};
use indoc::indoc;
use std::env;

fn main() {
    let app = App::new("trello-rs")
        .about(indoc!(
            "

            Trello command line tool.

            Begin by setting the environment variables:
            * TRELLO_API_TOKEN
            * TRELLO_API_DEVELOPER_KEY

            These can be retrieved from https://trello.com/app-key/
            "
        ))
        .subcommand(
            SubCommand::with_name("boards")
                .about("List all available boards")
                .arg(Arg::with_name("starred").short("s").long("starred")),
        )
        .subcommand(
            SubCommand::with_name("board")
                .about("View target Board")
                .arg(
                    Arg::with_name("board_id")
                        .short("i")
                        .long("id")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("board_name")
                        .short("n")
                        .long("name")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("card").about("View target card").arg(
                Arg::with_name("card_id")
                    .short("i")
                    .long("id")
                    .takes_value(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("close")
                .about("Close target trello item")
                .arg(Arg::with_name("target_type").index(1).required(true))
                .arg(Arg::with_name("target_id").index(2).required(true)),
        );

    let matches = app.get_matches();

    let token = env::var("TRELLO_API_TOKEN");
    let key = env::var("TRELLO_API_DEVELOPER_KEY");

    if token.is_err() || key.is_err() {
        println!("TRELLO_API_TOKEN and TRELLO_API_DEVELOPER_KEY environment variables must be set");
        return;
    }

    let token = token.unwrap();
    let key = key.unwrap();

    if let Some(matches) = matches.subcommand_matches("boards") {
        boards(&matches, &token, &key);
    } else if let Some(matches) = matches.subcommand_matches("board") {
        board(&matches, &token, &key);
    } else if let Some(matches) = matches.subcommand_matches("card") {
        card(&matches, &token, &key);
    } else if let Some(matches) = matches.subcommand_matches("close") {
        close(&matches, &token, &key);
    } else {
        println!("No subcommand specified. Use help to for more information.");
    }
}

fn boards(matches: &ArgMatches, token: &str, key: &str) {
    let starred = matches.is_present("starred");

    let boards = match trello::Board::get_all(token, key) {
        Ok(b) => b,
        Err(e) => {
            println!("Could not retrieve boards");
            println!("{}", e);
            return;
        },
    };

    for b in boards {
        // TODO: Should be able to pass this directly as a filter to the API
        if starred && !b.starred.unwrap() {
            continue;
        }

        let text = &format!("{} ({})", b.name, b.id);
        let output = style(&text);
        println!("* {}", output);
    }
}

fn board(matches: &ArgMatches, token: &str, key: &str) {
    let board_id = matches.value_of("board_id");
    let board_name = matches.value_of("board_name");

    let board;
    if let Some(board_id) = board_id {
        board = match trello::Board::get(board_id, token, key) {
            Ok(b) => b,
            Err(e) => {
                println!("Could not retrieve board");
                println!("{}", e);
                return;
            },
        };
    } else if let Some(board_name) = board_name {
        // TODO: Should be handling case where board is not found gracefully
        board = match trello::Board::get_by_name(board_name, token, key) {
            Ok(b) => b.unwrap(),
            Err(e) => {
                println!("Could not retrieve board");
                println!("{}", e);
                return;
            },
        };
    } else {
        println!("You must supply either a board id (--id) or a board name (--name)");
        return;
    }

    let title = format!("{} ({})", board.name, board.id);
    print_header(&title, "=");

    println!("{}", style(&board.url).bold().underlined());

    if let Some(desc_data) = board.desc_data {
        println!("{}", desc_data);
    }

    let lists = match trello::List::get_all(&board.id, token, key) {
        Ok(l) => l,
        Err(e) => {
            println!("Unable to retrieve board lists");
            println!("{}", e);
            return;
        },
    };

    for l in lists {
        println!("");
        print_header(&format!("{} ({})", l.name, l.id), "-");

        if let Some(cards) = l.cards {
            for c in cards {
                let labels: Vec<StyledObject<&String>> = c
                    .labels
                    .iter()
                    .map(|l| l.get_colored_name().bold())
                    .collect();

                println!("* {} ({}) {:?}", c.name, c.id, labels);
            }
        }
    }
}

fn card(matches: &ArgMatches, token: &str, key: &str) {
    let card_id = matches.value_of("card_id");

    let card;
    if let Some(card_id) = card_id {
        card = match trello::Card::get(card_id, token, key) {
            Ok(c) => c,
            Err(e) => {
                println!("Unable to retrieve card");
                println!("{}", e);
                return;
            },
        };
    } else {
        println!("You must supply a card id (--id)");
        return;
    }

    print_header(&card.name, "=");

    println!("{}", &card.desc);
}

fn close(matches: &ArgMatches, token: &str, key: &str) {
    let target_type = matches.value_of("target_type").unwrap();
    let target_id = matches.value_of("target_id").unwrap();

    if target_type == "card" {
        let card = match trello::Card::close(target_id, token, key) {
            Ok(c) => c,
            Err(e) => {
                println!("An error occurred while closing the card");
                println!("{}", e);
                return;
            },
        };
        println!("Closed '{}'", card.name);
    } else if target_type == "board" {
        let board = match trello::Board::close(target_id, token, key) {
            Ok(b) => b,
            Err(e) => {
                println!("An error occurred while closing the board");
                println!("{}", e);
                return;
            },
        };
        println!("Closed '{}'", board.name);
    } else if target_type == "list" {
        let list = match trello::List::close(target_id, token, key) {
            Ok(l) => l,
            Err(e) => {
                println!("An error occurred while closing the list");
                println!("{}", e);
                return;
            },
        };
        println!("Closed '{}'", list.name);
    } else {
        println!("Unknown target type");
    }
}

fn print_header(text: &str, header_char: &str) {
    println!("{}", text);
    println!("{}", header_char.repeat(text.len()));
}
