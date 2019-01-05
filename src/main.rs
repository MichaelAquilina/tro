mod trello;

#[macro_use]
extern crate serde_derive;

extern crate console;

use console::StyledObject;
use std::env;

fn main() {
    let token = env::var("TRELLO_API_TOKEN");
    let key = env::var("TRELLO_API_DEVELOPER_KEY");

    if token.is_err() || key.is_err() {
        println!("TRELLO_API_TOKEN and TRELLO_API_DEVELOPER_KEY environment variables must be set");
        return;
    }

    let token = token.unwrap();
    let key = key.unwrap();

    let boards = trello::get_boards(&token, &key);

    for (index, b) in boards.iter().enumerate() {
        if b.name == "TODO" {
            println!("{}: {}", index, b.name);
            let lists = trello::get_lists(&b.id, &token, &key);
            for l in lists {
                println!("{} ({})", l.name, l.id);
            }

            let cards = trello::get_cards(&b.id, &token, &key);
            for c in cards {
                let labels: Vec<StyledObject<&String>> = c
                    .labels
                    .iter()
                    .map(|l| l.get_colored_name().bold())
                    .collect();

                println!("{} {:?}", c.name, labels);
            }
            break;
        }
    }
}
