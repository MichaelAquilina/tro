use super::*;

#[test]
fn test_new() {
    let card = Card::new("A", "B", "C", None, "https://trello.com/my/card");
    let expected = Card {
        id: String::from("A"),
        name: String::from("B"),
        desc: String::from("C"),
        labels: None,
        closed: false,
        url: String::from("https://trello.com/my/card"),
    };
    assert_eq!(card, expected);
}

#[test]
fn test_render() {
    let card = Card::new("aaaaa", "My Fav Card", "this is a nice card", None, "");

    let expected = "My Fav Card\n===========\nthis is a nice card";
    assert_eq!(card.render(), expected);
}

#[test]
fn test_create() -> Result<()> {
    let _m = mockito::mock("POST", "/1/cards/?key=some-key&token=some-token")
        .match_body("name=Laundry&desc=Desky&idList=FOOBAR")
        .with_status(200)
        .with_body(
            json!({
                "name": "Laundry",
                "desc": "Desky",
                "id": "88888",
                "closed": false,
                "url": "https://example.com/1/12/",
            })
            .to_string(),
        )
        .create();

    let client = Client::new(&mockito::server_url(), "some-token", "some-key");
    let result = Card::create(
        &client,
        "FOOBAR",
        &Card::new("", "Laundry", "Desky", None, ""),
    )?;
    let expected = Card::new(
        "88888",
        "Laundry",
        "Desky",
        None,
        "https://example.com/1/12/",
    );

    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn test_update() -> Result<()> {
    let _m = mockito::mock("PUT", "/1/cards/MY-CARD-ID/?key=some-key&token=some-token")
        .match_body("name=Laundry&desc=hello&closed=true")
        .with_status(200)
        .with_body(
            json!({
                "name": "Laundry",
                "desc": "hello",
                "id": "MY-CARD-ID",
                "closed": true,
                "url": "https://trello.com/abcdef",
            })
            .to_string(),
        )
        .create();

    let client = Client::new(&mockito::server_url(), "some-token", "some-key");

    let mut card = Card::new(
        "MY-CARD-ID",
        "Laundry",
        "hello",
        None,
        "https://trello.com/abcdef",
    );
    card.closed = true;

    let result = Card::update(&client, &card)?;
    assert_eq!(result, card);
    Ok(())
}

#[test]
fn test_get_all() -> Result<()> {
    let _m = mockito::mock(
        "GET",
        "/1/lists/DEADBEEF/cards/?key=some-key&token=some-secret-token&fields=id%2Cname%2Cdesc%2Clabels%2Cclosed%2Curl",
    )
    .with_status(200)
    .with_body(
        json!([
            {"name": "Water the plants", "id": "abc-def", "desc": "", "closed": false, "url": ""},
            {"name": "Feed the dog", "id": "123-456", "desc": "for a good boy", "closed": false, "url": ""},
        ])
        .to_string(),
    )
    .create();

    let client = Client::new(&mockito::server_url(), "some-secret-token", "some-key");
    let result = Card::get_all(&client, "DEADBEEF")?;
    let expected = vec![
        Card::new("abc-def", "Water the plants", "", None, ""),
        Card::new("123-456", "Feed the dog", "for a good boy", None, ""),
    ];

    assert_eq!(result, expected);
    Ok(())
}
