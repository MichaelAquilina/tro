use super::*;

use colored::*;

#[test]
fn test_new() {
    let board = Board::new("888", "some board", Some(vec![]), "https://trello.com/09");
    let expected = Board {
        id: String::from("888"),
        name: String::from("some board"),
        lists: Some(vec![]),
        closed: false,
        url: String::from("https://trello.com/09"),
    };
    assert_eq!(board, expected);
}

#[test]
fn test_render_no_lists() {
    let board = Board::new("", "Knights", None, "");
    #[rustfmt::skip]
    let expected = [
        " Knights ",
        "=========",
    ].join("\n").bold().to_string();
    assert_eq!(board.render(true), expected);
}

#[test]
fn test_render_lists() {
    let board = Board::new(
        "",
        "Knights",
        Some(vec![
            List::new("", "King", None),
            List::new("", "Shovel", None),
        ]),
        "",
    );
    #[rustfmt::skip]
    let expected = [
        [
            " Knights ",
            "=========",
        ].join("\n").bold().to_string(),
        String::from(""),
        [
            "King",
            "----",
        ].join("\n").bold().to_string(),
        String::from(""),
        [
            "Shovel",
            "------",
        ].join("\n").bold().to_string(),
    ].join("\n");
    assert_eq!(board.render(true), expected);
}

#[test]
fn test_render_lists_and_cards() {
    let board = Board::new(
        "",
        "Knights",
        Some(vec![
            List::new(
                "",
                "King",
                Some(vec![Card::new("", "Gyro Boots", "", None, "", None)]),
            ),
            List::new(
                "",
                "Shovel",
                Some(vec![Card::new("", "Flare Wand", "Relic", None, "", None)]),
            ),
        ]),
        "",
    );
    #[rustfmt::skip]
    let expected = [
        [
            " Knights ",
            "=========",
        ].join("\n").bold().to_string(),
        String::from(""),
        [
            "King",
            "----",
        ].join("\n").bold().to_string(),
        String::from("* Gyro Boots"),
        String::from(""),
        [
            "Shovel",
            "------",
        ].join("\n").bold().to_string(),
        format!("* Flare Wand {}", "[...]".dimmed()),
    ].join("\n");
    assert_eq!(board.render(true), expected);
}

#[test]
fn test_create() -> Result<()> {
    let _m = mockito::mock("POST", "/1/boards/?key=some-key&token=some-token")
        .match_body("name=MYTESTBOARD")
        .with_status(200)
        .with_body(
            json!({
                "name": "MYTESTBOARD",
                "id": "231dgfe4r343",
                "closed": false,
                "url": "https://example.com/board",
            })
            .to_string(),
        )
        .create();

    let config = ClientConfig::new(&mockito::server_url(), "some-token", "some-key");
    let client = TrelloClient::new(config);

    let result = Board::create(&client, "MYTESTBOARD")?;
    let expected = Board::new(
        "231dgfe4r343",
        "MYTESTBOARD",
        None,
        "https://example.com/board",
    );

    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn test_update() -> Result<()> {
    let _m = mockito::mock(
        "PUT",
        "/1/boards/MY-BOARD-ID/?key=some-key&token=some-token",
    )
    .match_body("name=TODO&closed=true")
    .with_status(200)
    .with_body(
        json!({
            "name": "TODO",
            "id": "MY-BOARD-ID",
            "closed": true,
            "url": "",
        })
        .to_string(),
    )
    .create();

    let config = ClientConfig::new(&mockito::server_url(), "some-token", "some-key");
    let client = TrelloClient::new(config);

    let mut board = Board::new("MY-BOARD-ID", "TODO", None, "");
    board.closed = true;

    let result = Board::update(&client, &board)?;

    assert_eq!(result, board);
    Ok(())
}

#[test]
fn test_get_all() -> Result<()> {
    let _m = mockito::mock(
        "GET",
        "/1/members/me/boards/?key=some-key&token=some-secret-token&filter=open&fields=id%2Cname%2Cclosed%2Curl",
    )
    .with_status(200)
    .with_body(
        json!([
            {"name": "TODO", "id": "abc-def", "closed": false, "url": "bit.ly/1"},
            {"name": "foo", "id": "123-456", "closed": false, "url": "bit.ly/2"},
        ])
        .to_string(),
    )
    .create();

    let config = ClientConfig::new(&mockito::server_url(), "some-secret-token", "some-key");
    let client = TrelloClient::new(config);

    let result = Board::get_all(&client)?;
    let expected = vec![
        Board::new("abc-def", "TODO", None, "bit.ly/1"),
        Board::new("123-456", "foo", None, "bit.ly/2"),
    ];

    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn test_get() -> Result<()> {
    let _m = mockito::mock(
        "GET",
        "/1/boards/some-board-id?key=KEY&token=TOKEN&fields=id%2Cname%2Cclosed%2Curl",
    )
    .with_status(200)
    .with_body(
        json!({
            "name": "My Favourite Board",
            "id": "some-board-id",
            "closed": false,
            "url": "https://bit.ly/12",
        })
        .to_string(),
    )
    .create();

    let config = ClientConfig::new(&mockito::server_url(), "TOKEN", "KEY");
    let client = TrelloClient::new(config);

    let result = Board::get(&client, "some-board-id")?;
    let expected = Board::new(
        "some-board-id",
        "My Favourite Board",
        None,
        "https://bit.ly/12",
    );
    assert_eq!(result, expected);

    Ok(())
}
