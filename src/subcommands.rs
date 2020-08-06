use crate::{cli, find};
use clap::ArgMatches;
use colored::*;
use std::error::Error;
use trello::{
    search, Attachment, Board, Card, Client, Label, List, Member, Renderable, SearchOptions,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn setup_subcommand(matches: &ArgMatches) -> Result<()> {
    debug!("Running setup subcommand with {:?}", matches);

    println!("{}", "Welcome to tro!".green().bold());
    println!();
    println!(
        "Please generate a Developer {} and {} from https://trello.com/app-key/",
        "key".green(),
        "token".green()
    );
    println!("and enter them below");
    println!();

    let key = cli::get_input("Enter Developer API Key: ")?;
    let token = cli::get_input("Enter Token: ")?;

    let client = Client {
        host: Client::default_host(),
        key,
        token,
    };

    println!();

    match Member::me(&client) {
        Ok(member) => {
            client.save_config()?;
            println!(
                "Successfully logged in as {} with tro!",
                member.username.green()
            );
        }
        Err(_) => {
            println!(
                "{}",
                "Unable to validate credentials. Please re-check and try again".red()
            );
        }
    };

    Ok(())
}

pub fn me_subcommand(client: &Client, matches: &ArgMatches) -> Result<()> {
    debug!("Running me subcommand with {:?}", matches);

    let detailed = matches.is_present("detailed");

    let member = Member::me(client)?;

    if detailed {
        println!("username: {}", member.username);
        println!("full name: {}", member.full_name);
        println!("id: {}", member.id);
    } else {
        println!("{}", member.username);
    }

    Ok(())
}

pub fn show_subcommand(client: &Client, matches: &ArgMatches) -> Result<()> {
    debug!("Running show subcommand with {:?}", matches);

    let label_filter = matches.value_of("label_filter");
    let interactive = matches.is_present("interactive");

    let params = find::get_trello_params(matches);
    debug!("Trello Params: {:?}", params);

    let result = find::get_trello_object(client, &params)?;
    trace!("result: {:?}", result);

    if interactive {
        if result.card.is_some() {
            eprintln!("Cannot use interactive code if a card pattern is specified");
        } else if let Some(list) = result.list {
            let cards = Card::get_all(client, &list.id)?;

            if let Some(index) = cli::select_trello_object(&cards)? {
                cli::edit_card(client, &cards[index])?;
            }
        } else if let Some(board) = result.board {
            let lists = List::get_all(client, &board.id, true)?;

            if let Some(index) = cli::select_trello_object(&lists)? {
                // TODO: Allow label filtering
                println!("{}", &lists[index].render());
            }
        } else {
            let mut boards = Board::get_all(client)?;

            if let Some(index) = cli::select_trello_object(&boards)? {
                boards[index].retrieve_nested(client)?;
                println!("{}", &boards[index].render());
            }
        }
    } else if let Some(card) = result.card {
        cli::edit_card(client, &card)?;
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

pub fn open_subcommand(client: &Client, matches: &ArgMatches) -> Result<()> {
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
        panic!("Unknown object_type {}", object_type);
    }

    Ok(())
}

// TODO: The three functions below can be generalised using traits
fn close_board(client: &Client, board: &mut Board) -> Result<()> {
    board.closed = true;
    Board::update(client, board)?;

    eprintln!("Closed board: '{}'", &board.name.green());
    eprintln!("id: {}", &board.id);

    Ok(())
}

fn close_list(client: &Client, list: &mut List) -> Result<()> {
    list.closed = true;
    List::update(client, list)?;

    eprintln!("Closed list: '{}'", &list.name.green());
    eprintln!("id: {}", &list.id);

    Ok(())
}

fn close_card(client: &Client, card: &mut Card) -> Result<()> {
    card.closed = true;
    Card::update(client, card)?;

    eprintln!("Closed card: '{}'", &card.name.green());
    eprintln!("id: {}", &card.id);

    Ok(())
}

pub fn close_subcommand(client: &Client, matches: &ArgMatches) -> Result<()> {
    debug!("Running close subcommand with {:?}", matches);

    let params = find::get_trello_params(matches);
    let result = find::get_trello_object(client, &params)?;

    let interactive = matches.is_present("interactive");

    trace!("result: {:?}", result);

    if interactive {
        if result.card.is_some() {
            eprintln!("Cannot run interactive mode if you specify a card pattern");
        } else if let Some(list) = result.list {
            let mut cards = Card::get_all(client, &list.id)?;

            for index in cli::multiselect_trello_object(&cards, &[])? {
                close_card(client, &mut cards[index])?;
            }
        } else if let Some(board) = result.board {
            let mut lists = List::get_all(client, &board.id, false)?;

            for index in cli::multiselect_trello_object(&lists, &[])? {
                close_list(client, &mut lists[index])?;
            }
        } else {
            let mut boards = Board::get_all(client)?;

            for index in cli::multiselect_trello_object(&boards, &[])? {
                close_board(client, &mut boards[index])?;
            }
        }
    } else if let Some(mut card) = result.card {
        close_card(client, &mut card)?;
    } else if let Some(mut list) = result.list {
        close_list(client, &mut list)?;
    } else if let Some(mut board) = result.board {
        close_board(client, &mut board)?;
    }

    Ok(())
}

pub fn create_subcommand(client: &Client, matches: &ArgMatches) -> Result<()> {
    debug!("Running create subcommand with {:?}", matches);

    let params = find::get_trello_params(matches);
    let result = find::get_trello_object(client, &params)?;

    let show = matches.is_present("show");

    trace!("result: {:?}", result);

    if let Some(list) = result.list {
        let name = match matches.value_of("name") {
            Some(n) => String::from(n),
            None => cli::get_input("Card name: ")?,
        };

        let card = Card::create(client, &list.id, &Card::new("", &name, "", None, "", None))?;

        if let Some(label_names) = matches.values_of("label") {
            let labels =
                Label::get_all(&client, &result.board.ok_or("Unable to retrieve board")?.id)?;
            for name in label_names {
                let label = match find::get_object_by_name(&labels, name, true) {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!("{}", e);
                        continue;
                    }
                };
                match Label::apply(client, &card.id, &label.id) {
                    Ok(_) => eprintln!("Applied {} label", &label.simple_render(),),
                    Err(e) => eprintln!("Unable to apply {} label: {}", &label.simple_render(), e),
                };
            }
        }

        if show {
            cli::edit_card(client, &card)?;
        }
    } else if let Some(board) = result.board {
        let name = match matches.value_of("name") {
            Some(n) => String::from(n),
            None => cli::get_input("List name: ")?,
        };

        List::create(client, &board.id, &name)?;
    } else {
        let name = match matches.value_of("name") {
            Some(n) => String::from(n),
            None => cli::get_input("Board name: ")?,
        };

        Board::create(client, &name)?;
    }

    Ok(())
}
pub fn attachments_subcommand(client: &Client, matches: &ArgMatches) -> Result<()> {
    debug!("Running attachments subcommand with {:?}", matches);

    let params = find::get_trello_params(matches);
    let result = find::get_trello_object(client, &params)?;

    let card = result.card.ok_or("Unable to find card")?;

    let attachments = Attachment::get_all(client, &card.id)?;

    for att in attachments {
        println!("{}", &att.url);
    }

    Ok(())
}

pub fn attach_subcommand(client: &Client, matches: &ArgMatches) -> Result<()> {
    debug!("Running attach subcommand with {:?}", matches);

    let params = find::get_trello_params(matches);
    let result = find::get_trello_object(client, &params)?;

    let path = matches.value_of("path").unwrap();

    let card = result.card.ok_or("Unable to find card")?;

    let attachment = Attachment::apply(client, &card.id, path)?;

    println!("{}", attachment.render());

    Ok(())
}

pub fn url_subcommand(client: &Client, matches: &ArgMatches) -> Result<()> {
    debug!("Running url subcommand with {:?}", matches);

    let params = find::get_trello_params(matches);
    let result = find::get_trello_object(client, &params)?;

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

// Because clap interprets parameters that start with "-" as flags
// we need to provide an alternative way for users to specify the
// "negative" search operator. In this case, we allow for '~' to
// be specified as the negative search operator
fn replace_negative_prefix(query: &str) -> String {
    if query.starts_with('~') {
        query.replacen('~', "-", 1)
    } else {
        query.to_string()
    }
}

pub fn search_subcommand(client: &Client, matches: &ArgMatches) -> Result<()> {
    debug!("Running search subcommand with {:?}", matches);

    let query = matches
        .values_of("query")
        .ok_or("Missing query value")?
        .map(|s| replace_negative_prefix(s))
        .collect::<Vec<String>>()
        .join(" ");
    let partial = matches.is_present("partial");
    let interactive = matches.is_present("interactive");

    let cards_limit = if let Some(v) = matches.value_of("limit") {
        Some(v.parse()?)
    } else {
        None
    };

    let params = SearchOptions {
        cards_limit,
        // Seems that 0 cannot be passed
        // so we just pass the lowest accepted value instead
        boards_limit: Some(1),
        partial,
    };

    let results = search(client, &query, &params)?;

    if interactive {
        if let Some(index) = cli::select_trello_object(&results.cards)? {
            cli::edit_card(client, &results.cards[index])?;
        }
    } else if !&results.cards.is_empty() {
        for card in &results.cards {
            println!(
                "{} {}",
                card.simple_render(),
                format!("id: {}", card.id).green()
            );
        }
    }

    Ok(())
}

fn delete_label(client: &Client, card: &Card, label: &Label) -> Result<()> {
    Label::remove(client, &card.id, &label.id)?;

    eprintln!(
        "Removed {} label from '{}'",
        &label.simple_render(),
        &card.name.green(),
    );

    Ok(())
}

fn apply_label(client: &Client, card: &Card, label: &Label) -> Result<()> {
    Label::apply(client, &card.id, &label.id)?;

    eprintln!(
        "Applied {} label to '{}'",
        &label.simple_render(),
        &card.name.green()
    );

    Ok(())
}

pub fn label_subcommand(client: &Client, matches: &ArgMatches) -> Result<()> {
    debug!("Running label subcommand with {:?}", matches);

    let params = find::get_trello_params(matches);
    let result = find::get_trello_object(client, &params)?;

    let interactive = matches.is_present("interactive");
    let delete = matches.is_present("delete");
    let label_names = matches.values_of("label_name");

    let card = result.card.ok_or("Unable to find card")?;
    let card_labels = card.labels.as_ref().ok_or("Unable to get card labels")?;

    if delete {
        let labels = card_labels;
        let label_names = label_names.ok_or("Label names must be specified")?;

        for name in label_names {
            let label = match find::get_object_by_name(&labels, name, true) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            };

            delete_label(client, &card, &label)?;
        }
    } else {
        let board = result.board.ok_or("Unable to retrieve board")?;
        let labels = Label::get_all(&client, &board.id)?;

        if interactive {
            let selected_labels = cli::multiselect_trello_object(&labels, card_labels)?
                .into_iter()
                .map(|i| &labels[i])
                .collect::<Vec<&Label>>();

            for label in &selected_labels {
                if !card_labels.contains(label) {
                    apply_label(client, &card, label)?;
                }
            }

            for label in card_labels {
                if !selected_labels.contains(&label) {
                    delete_label(client, &card, label)?;
                }
            }
        } else {
            let label_names = label_names.ok_or("Label names must be specified")?;

            for name in label_names {
                let label = match find::get_object_by_name(&labels, name, true) {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!(
                            "Label with pattern '{}' not found or is already assigned",
                            name
                        );
                        debug!("{}", e);
                        continue;
                    }
                };

                apply_label(client, &card, &label)?;
            }
        }
    }

    Ok(())
}
