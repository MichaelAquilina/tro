mod trello;

#[macro_use]
extern crate serde_derive;

use std::env;

fn main() {
    let token = env::var("TRELLO_API_TOKEN").unwrap();
    let key = env::var("TRELLO_API_DEVELOPER_KEY").unwrap();

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
                let labels: Vec<&String> = c.labels.iter().map(|l| &l.name).collect();

                println!("{} {:?}", c.name, labels);
            }
            break;
        }
    }
}
