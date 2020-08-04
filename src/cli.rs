use anyhow::Result;
use std::env;
use std::io::{Read, Write};
use std::process;
use std::{thread, time};
use trello::Renderable;
use trello::{Card, CardContents, Client, TrelloError, TrelloObject};

pub fn multiselect_trello_object<T: TrelloObject + Renderable + PartialEq>(
    objects: &[T],
    selected: &[T],
) -> Result<Vec<usize>> {
    let result = dialoguer::MultiSelect::new()
        .items_checked(
            &objects
                .iter()
                .map(|o| (o.simple_render(), selected.contains(o)))
                .collect::<Vec<(String, bool)>>(),
        )
        .with_prompt(format!("Select {}s using space key", T::get_type()))
        .interact()?;

    Ok(result)
}

pub fn select_trello_object<T: TrelloObject + Renderable>(
    objects: &[T],
) -> Result<Option<usize>, std::io::Error> {
    let result = dialoguer::Select::new()
        .items(
            &objects
                .iter()
                .map(|o| o.simple_render())
                .collect::<Vec<String>>(),
        )
        .with_prompt(format!("Select {}", T::get_type()))
        .interact_opt()?;

    Ok(result)
}

pub fn get_input(text: &str) -> Result<String, rustyline::error::ReadlineError> {
    let mut rl = rustyline::Editor::<()>::new();
    rl.bind_sequence(
        rustyline::KeyPress::ControlLeft,
        rustyline::Cmd::Move(rustyline::Movement::BackwardWord(1, rustyline::Word::Big)),
    );
    rl.bind_sequence(
        rustyline::KeyPress::ControlRight,
        rustyline::Cmd::Move(rustyline::Movement::ForwardWord(
            1,
            rustyline::At::Start,
            rustyline::Word::Big,
        )),
    );
    rl.readline(text)
}

/// Opens the users chosen editor (specified by the $EDITOR environment variable)
/// to edit a specified card. If $EDITOR is not set, the default editor will fallback
/// to vi.
///
/// This function will upload any changes written by the editor to Trello. This includes
/// when the editor is not closed but content is saved.
pub fn edit_card(client: &Client, card: &Card) -> Result<()> {
    let mut file = tempfile::Builder::new().suffix(".md").tempfile()?;
    let editor_env = env::var("EDITOR").unwrap_or_else(|_| String::from("vi"));

    debug!("Using editor: {}", editor_env);
    debug!("Editing card: {:?}", card);

    writeln!(file, "{}", card.render())?;

    let mut new_card = card.clone();

    // Outer retry loop - reopen editor if last upload attempt failed
    loop {
        let mut editor = process::Command::new(&editor_env)
            .arg(file.path())
            .spawn()?;
        let mut result: Option<Result<Card, TrelloError>> = None;

        // Inner watch loop - look out for card changes to upload
        loop {
            const SLEEP_TIME: u64 = 500;
            debug!("Sleeping for {}ms", SLEEP_TIME);
            thread::sleep(time::Duration::from_millis(SLEEP_TIME));

            let mut buf = String::new();
            file.reopen()?.read_to_string(&mut buf)?;

            // Trim end because a lot of editors will auto add new lines at the end of the file
            let contents: CardContents = match buf.trim_end().parse() {
                Ok(c) => c,
                Err(e) => {
                    debug!("Unable to parse Card Contents: {}", e);
                    if let Some(ecode) = editor.try_wait()? {
                        debug!("Editor closed (code {}), exiting watch loop", ecode);
                        result = Some(Err(e));
                        break;
                    } else {
                        // no need to break watch loop, error might be corrected on next save
                        continue;
                    }
                }
            };

            // if no upload attempts
            // if previous loop had a failure
            // if card in memory is different to card in file
            if result.is_none()
                || result.as_ref().unwrap().is_err()
                || new_card.name != contents.name
                || new_card.desc != contents.desc
            {
                new_card.name = contents.name;
                new_card.desc = contents.desc;

                debug!("Updating card: {:?}", new_card);
                result = Some(Card::update(client, &new_card));

                match &result {
                    Some(Ok(_)) => debug!("Updated card"),
                    Some(Err(e)) => debug!("Error updating card {:?}", e),
                    None => panic!("This should be impossible"),
                };
            }

            if let Some(ecode) = editor.try_wait()? {
                debug!("Exiting editor loop with code: {}", ecode);
                break;
            }
        }

        match &result {
            None => {
                debug!("Exiting retry loop due to no result being ever retrieved");
                break;
            }
            Some(Ok(_)) => {
                debug!("Exiting retry loop due to successful last update");
                break;
            }
            Some(Err(e)) => {
                eprintln!("An error occurred while trying to update the card.");
                eprintln!("{}", e);
                eprintln!();
                get_input("Press 'enter' to go back to your editor")?;
            }
        }
    }

    Ok(())
}
