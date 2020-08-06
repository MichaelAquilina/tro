use super::*;

use colored::*;

#[test]
fn test_new() {
    let list = List::new("123", "my list", Some(vec![]));
    let expected = List {
        id: String::from("123"),
        name: String::from("my list"),
        cards: Some(vec![]),
        closed: false,
    };
    assert_eq!(list, expected);
}

#[test]
fn test_filter_none_cards() {
    let list = List::new("some-id", "some-name", None);

    assert_eq!(
        list.filter("my-label"),
        List::new("some-id", "some-name", None)
    );
}

#[test]
fn test_render_no_cards() {
    let list = List::new("aaaaa", "King Knight", None);
    let expected = "King Knight\n-----------".bold().to_string();
    assert_eq!(list.render(), expected);
}

#[test]
fn test_render_with_cards() {
    let list = List::new(
        "aaaaa",
        "King Knight",
        Some(vec![
            Card::new("", "hello", "", None, "", None),
            Card::new("", "world", "", None, "", None),
        ]),
    );

    let expected = format!(
        "{}\n{}",
        "King Knight\n-----------".bold(),
        "* hello\n* world"
    );
    assert_eq!(list.render(), expected);
}

#[test]
fn test_create() -> Result<()> {
    let _m = mockito::mock("POST", "/1/lists/?key=some-key&token=some-token")
        .match_body("name=Today&idBoard=LEONSK")
        .with_status(200)
        .with_body(
            json!({
                "name": "Today",
                "id": "MTLDA",
                "closed": false,
            })
            .to_string(),
        )
        .create();

    let config = ClientConfig::new(&mockito::server_url(), "some-token", "some-key");
    let client = TrelloClient::new(config);

    let result = List::create(&client, "LEONSK", "Today")?;
    let expected = List::new("MTLDA", "Today", None);
    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn test_update() -> Result<()> {
    let _m = mockito::mock("PUT", "/1/lists/MY-LIST-ID/?key=some-key&token=some-token")
        .with_body("name=Today&closed=True")
        .with_status(200)
        .with_body(
            json!({
                "name": "Today",
                "id": "MY-LIST-ID",
                "closed": true,
            })
            .to_string(),
        )
        .create();

    let config = ClientConfig::new(&mockito::server_url(), "some-token", "some-key");
    let client = TrelloClient::new(config);

    let mut list = List::new("MY-LIST-ID", "Today", None);
    list.closed = true;

    let result = List::update(&client, &list)?;
    assert_eq!(result, list);
    Ok(())
}

#[test]
fn test_get_all() -> Result<()> {
    let _m = mockito::mock(
        "GET",
        "/1/boards/some-board-id/lists?key=some-key&token=some-token&fields=id%2Cname%2Cclosed",
    )
    .with_status(200)
    .with_body(
        json!([
            {"name": "Red", "id": "823-123", "closed": false},
            {"name": "Green", "id": "222-222", "closed": false},
        ])
        .to_string(),
    )
    .create();

    let config = ClientConfig::new(&mockito::server_url(), "some-token", "some-key");
    let client = TrelloClient::new(config);

    let result = List::get_all(&client, "some-board-id", false)?;
    let expected = vec![
        List::new("823-123", "Red", None),
        List::new("222-222", "Green", None),
    ];
    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn test_get_all_with_cards() -> Result<()> {
    let _m = mockito::mock(
        "GET",
        "/1/boards/some-board-id/lists?key=some-key&token=some-token&fields=id%2Cname%2Cclosed&cards=open",
    )
    .with_status(200)
    .with_body(
        json!([
            {"name": "Red", "id": "823-123", "closed": false, "cards": []},
            {
                "name": "Green",
                "id": "222-222",
                "closed": false,
                "cards": [
                    {"id": "card1", "name": "apple", "desc": "", "closed": false, "url": ""},
                ],
            },
        ])
        .to_string(),
    )
    .create();

    let config = ClientConfig::new(&mockito::server_url(), "some-token", "some-key");
    let client = TrelloClient::new(config);

    let result = List::get_all(&client, "some-board-id", true)?;
    let expected = vec![
        List::new("823-123", "Red", Some(vec![])),
        List::new(
            "222-222",
            "Green",
            Some(vec![Card::new("card1", "apple", "", None, "", None)]),
        ),
    ];
    assert_eq!(result, expected);
    Ok(())
}
