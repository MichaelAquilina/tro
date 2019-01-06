mod trello;

#[macro_use]
extern crate serde_derive;

extern crate clap;
extern crate console;
extern crate indoc;

use clap::{App, Arg, SubCommand};
use console::{style, StyledObject};
use indoc::indoc;
use std::env;

fn print_header(text: &str, header_char: &str) {
    println!("{}", text);
    println!("{}", header_char.repeat(text.len()));
}

fn boards(token: &str, key: &str) {
    let boards = trello::Board::get_all(token, key);

    for b in boards {
        let text = &format!("{} ({})", b.name, b.id);
        let mut output = style(&text);
        if b.starred {
            output = output.yellow();
        }
        println!("* {}", output);
    }
}

fn board(board_id: Option<&str>, board_name: Option<&str>, token: &str, key: &str) {
    let board;
    if let Some(board_id) = board_id {
        board = trello::Board::get(board_id, token, key);
    } else if let Some(board_name) = board_name {
        board = trello::Board::get_by_name(board_name, token, key).unwrap();
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

    let lists = trello::List::get_all(&board.id, token, key);
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
        .subcommand(SubCommand::with_name("boards").about("List all available boards"))
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

    if let Some(_) = matches.subcommand_matches("boards") {
        boards(&token, &key);
    } else if let Some(matches) = matches.subcommand_matches("board") {
        let board_id = matches.value_of("board_id");
        let board_name = matches.value_of("board_name");

        board(board_id, board_name, &token, &key);
    }
}
