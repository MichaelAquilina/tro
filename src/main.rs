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
    let boards = trello::get_boards(token, key);

    for b in boards {
        let text = &format!("{} ({})", b.name, b.id);
        let mut output = style(&text);
        if b.starred {
            output = output.yellow();
        }
        println!("{}", output);
    }
}

fn board(board_id: &str, token: &str, key: &str) {
    let board = trello::get_board(board_id, token, key);

    let mut title = String::new();

    if board.starred {
        title.push_str("★ ");
    }

    title.push_str(&format!("{} ({})", board.name, board.id));

    print_header(&title, "=");

    println!("{}", board.url);

    if let Some(desc_data) = board.desc_data {
        println!("{}", desc_data);
    }

    let lists = trello::get_lists(board_id, token, key);
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

                println!("{} ({}) {:?}", c.name, c.id, labels);
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
                .arg(Arg::with_name("board_id").index(1).required(true)),
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
        let board_id = matches.value_of("board_id").unwrap();
        board(&board_id, &token, &key);
    }
}
