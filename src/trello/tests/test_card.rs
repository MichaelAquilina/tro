use super::*;
use chrono::{TimeZone, Utc};

#[test]
fn test_new() {
    let card = Card::new("A", "B", "C", None, "https://trello.com/my/card", None);
    let expected = Card {
        id: String::from("A"),
        name: String::from("B"),
        desc: String::from("C"),
        labels: None,
        due: None,
        closed: false,
        url: String::from("https://trello.com/my/card"),
    };
    assert_eq!(card, expected);
}

#[test]
fn test_render() {
    let card = Card::new(
        "aaaaa",
        "My Fav Card",
        "this is a nice card",
        None,
        "",
        None,
    );

    let expected = "My Fav Card\n===========\nthis is a nice card";
    assert_eq!(card.render(), expected);
}

#[test]
fn test_simple_render() {
    let card = Card {
        id: String::from("1234"),
        name: String::from("Fire Monkey"),
        desc: String::from(""),
        closed: false,
        url: String::from(""),
        labels: None,
        due: None,
    };

    let expected = "Fire Monkey";
    assert_eq!(card.simple_render(), expected);
}

#[test]
fn test_simple_render_with_description() {
    let card = Card {
        id: String::from("1234"),
        name: String::from("Ice Snail"),
        desc: String::from("Some details which should not be shown"),
        closed: false,
        url: String::from(""),
        labels: None,
        due: None,
    };

    let expected = "Ice Snail \u{1b}[2m[...]\u{1b}[0m";
    assert_eq!(card.simple_render(), expected);
}

#[test]
fn test_simple_render_with_labels() {
    let card = Card {
        id: String::from("1234"),
        name: String::from("Lightning Goat"),
        desc: String::from(""),
        closed: false,
        url: String::from(""),
        labels: Some(vec![Label::new("", "Animals", "green")]),
        due: None,
    };

    let expected = "Lightning Goat \u{1b}[48;2;97;189;79;37m Animals \u{1b}[0m";
    assert_eq!(card.simple_render(), expected);
}

#[test]
fn test_simple_render_closed() {
    let card = Card {
        id: String::from("1234"),
        name: String::from("Earth Seagull"),
        desc: String::from(""),
        closed: true,
        url: String::from(""),
        labels: None,
        due: None,
    };

    let expected = "Earth Seagull \u{1b}[31m[Closed]\u{1b}[0m";
    assert_eq!(card.simple_render(), expected);
}

#[test]
fn test_get() -> Result<()> {
    let _m = mockito::mock("GET", "/1/cards/CARD-FOO?key=some-key&token=some-token")
        .with_status(200)
        .with_body(
            json!({
                "name": "Card Foo",
                "desc": "foozy card",
                "id": "CARD-FOO",
                "closed": false,
                "url": "https://card.foo/123",
            })
            .to_string(),
        )
        .create();

    let client = Client::new(&mockito::server_url(), "some-token", "some-key");
    let result = Card::get(&client, "CARD-FOO")?;

    let expected = Card::new(
        "CARD-FOO",
        "Card Foo",
        "foozy card",
        None,
        "https://card.foo/123",
        None,
    );

    assert_eq!(result, expected);

    Ok(())
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
        &Card::new("", "Laundry", "Desky", None, "", None),
    )?;
    let expected = Card::new(
        "88888",
        "Laundry",
        "Desky",
        None,
        "https://example.com/1/12/",
        None,
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
        None,
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
        "/1/lists/DEADBEEF/cards/?key=some-key&token=some-secret-token&fields=id%2Cname%2Cdesc%2Clabels%2Cclosed%2Cdue%2Curl",
    )
    .with_status(200)
    .with_body(
        json!([
            {
                "name": "Water the plants",
                "id": "abc-def",
                "desc": "",
                "closed": false,
                "url": "",
                "due": "2020-06-28T06:06:27-00:00",
            },
            {
                "name": "Feed the dog",
                "id": "123-456",
                "desc": "for a good boy",
                "closed": false,
                "url": "",
                "due": "2020-06-28T06:06:27-00:00",
            },
        ])
        .to_string(),
    )
    .create();

    let client = Client::new(&mockito::server_url(), "some-secret-token", "some-key");
    let result = Card::get_all(&client, "DEADBEEF")?;
    let expected = vec![
        Card::new(
            "abc-def",
            "Water the plants",
            "",
            None,
            "",
            Some(Utc.ymd(2020, 6, 28).and_hms(6, 6, 27)),
        ),
        Card::new(
            "123-456",
            "Feed the dog",
            "for a good boy",
            None,
            "",
            Some(Utc.ymd(2020, 6, 28).and_hms(6, 6, 27)),
        ),
    ];

    assert_eq!(result, expected);
    Ok(())
}
