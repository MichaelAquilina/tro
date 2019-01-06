mod trello;

#[macro_use]
extern crate serde_derive;

extern crate clap;
extern crate console;
extern crate indoc;

use clap::{App, Arg, SubCommand};
use console::StyledObject;
use indoc::indoc;
use std::env;

fn cards(board_name: &str, token: &str, key: &str) {
    let boards = trello::get_boards(token, key);

    for (index, b) in boards.iter().enumerate() {
        if b.name.to_lowercase() == board_name {
            println!("{}: {}", index, b.name);
            let lists = trello::get_lists(&b.id, token, key);
            for l in lists {
                println!("");
                let title = format!("{} ({})", l.name, l.id);

                println!("{}", title);
                println!("{}", "-".repeat(title.chars().count()));

                for c in l.cards.unwrap() {
                    let labels: Vec<StyledObject<&String>> = c
                        .labels
                        .iter()
                        .map(|l| l.get_colored_name().bold())
                        .collect();

                    println!("{} {:?}", c.name, labels);
                }
            }

            break;
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
        .subcommand(
            SubCommand::with_name("board")
                .about("View target Board")
                .arg(
                    Arg::with_name("board_name")
                        .help("Name of the board")
                        .index(1)
                        .required(true),
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

    if let Some(matches) = matches.subcommand_matches("board") {
        let board_name = matches.value_of("board_name").unwrap().to_lowercase();
        cards(&board_name, &token, &key);
    }
}
