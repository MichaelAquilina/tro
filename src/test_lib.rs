use super::{header, Board, Card, Client, List};
use mockito;
use serde_json::json;
use std::error::Error;

mod header_tests {
    use super::*;

    #[test]
    fn test_empty() {
        let result = header("", "-");
        assert_eq!(result, String::from("\n"));
    }

    #[test]
    fn test_correct() {
        let result = header("foobar", "=");
        assert_eq!(result, String::from("foobar\n======"));
    }

    #[test]
    fn test_emoticons() {
        let result = header("foo 🔴", "-");
        assert_eq!(result, String::from("foo 🔴\n-----"));
    }
}

mod card_tests {
    use super::*;

    #[test]
    fn test_create() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "POST",
            "/1/cards/?key=some-key&token=some-token&name=Laundry&idList=FOOBAR",
        )
        .with_status(200)
        .with_body(
            json!({
                "name": "Laundry",
                "desc": "",
                "id": "88888",
                "closed": false,
            })
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "some-token", "some-key");
        let result = Card::create(&client, "FOOBAR", "Laundry")?;
        let expected = Card {
            id: String::from("88888"),
            name: String::from("Laundry"),
            desc: String::from(""),
            closed: false,
        };
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_update() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "PUT",
            "/1/cards/MY-CARD-ID/?key=some-key&token=some-token&name=Laundry&desc=hello&closed=true",
        )
        .with_status(200)
        .with_body(
            json!({
                "name": "Laundry",
                "desc": "hello",
                "id": "MY-CARD-ID",
                "closed": true,
            })
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "some-token", "some-key");

        let card = Card {
            id: "MY-CARD-ID".to_string(),
            name: "Laundry".to_string(),
            desc: "hello".to_string(),
            closed: true,
        };

        let result = Card::update(&client, &card)?;
        assert_eq!(result, card);
        Ok(())
    }
}

mod list_tests {
    use super::*;

    #[test]
    fn test_create() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "POST",
            "/1/lists/?key=some-key&token=some-token&name=Today&idBoard=LEONSK",
        )
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

        let client = Client::new(&mockito::server_url(), "some-token", "some-key");
        let result = List::create(&client, "LEONSK", "Today")?;
        let expected = List {
            id: String::from("MTLDA"),
            name: String::from("Today"),
            cards: None,
            closed: false,
        };
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_update() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "PUT",
            "/1/lists/MY-LIST-ID/?key=some-key&token=some-token&name=Today&closed=true",
        )
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

        let client = Client::new(&mockito::server_url(), "some-token", "some-key");

        let list = List {
            id: "MY-LIST-ID".to_string(),
            name: "Today".to_string(),
            cards: None,
            closed: true,
        };

        let result = List::update(&client, &list)?;
        assert_eq!(result, list);
        Ok(())
    }

    #[test]
    fn test_get_all_cards() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "GET",
            "/1/lists/DEADBEEF/cards/?key=some-key&token=some-secret-token&fields=id%2Cname%2Cdesc%2Cclosed",
        )
        .with_status(200)
        .with_body(
            json!([
                {"name": "Water the plants", "id": "abc-def", "desc": "", "closed": false},
                {"name": "Feed the dog", "id": "123-456", "desc": "for a good boy", "closed": false},
            ])
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "some-secret-token", "some-key");
        let result = List::get_all_cards(&client, "DEADBEEF")?;
        let expected = vec![
            Card {
                name: String::from("Water the plants"),
                id: String::from("abc-def"),
                desc: String::from(""),
                closed: false,
            },
            Card {
                name: String::from("Feed the dog"),
                id: String::from("123-456"),
                desc: String::from("for a good boy"),
                closed: false,
            },
        ];

        assert_eq!(result, expected);
        Ok(())
    }
}

mod board_tests {
    use super::*;

    #[test]
    fn test_create() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "POST",
            "/1/boards/?key=some-key&token=some-token&name=MY-TEST-BOARD",
        )
        .with_status(200)
        .with_body(
            json!({
                "name": "MY-TEST-BOARD",
                "id": "231dgfe4r343",
                "closed": false,
            })
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "some-token", "some-key");
        let result = Board::create(&client, "MY-TEST-BOARD")?;
        let expected = Board {
            id: String::from("231dgfe4r343"),
            name: String::from("MY-TEST-BOARD"),
            closed: false,
            lists: None,
        };
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_update() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "PUT",
            "/1/boards/MY-BOARD-ID/?key=some-key&token=some-token&name=TODO&closed=true",
        )
        .with_status(200)
        .with_body(
            json!({
                "name": "TODO",
                "id": "MY-BOARD-ID",
                "closed": true,
            })
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "some-token", "some-key");

        let board = Board {
            id: "MY-BOARD-ID".to_string(),
            name: "TODO".to_string(),
            closed: true,
            lists: None,
        };

        let result = Board::update(&client, &board)?;
        assert_eq!(result, board);
        Ok(())
    }

    #[test]
    fn test_get_all() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "GET",
            "/1/members/me/boards/?key=some-key&token=some-secret-token&filter=open&fields=id%2Cname%2Cclosed",
        )
        .with_status(200)
        .with_body(
            json!([
                {"name": "TODO", "id": "abc-def", "closed": false},
                {"name": "foo", "id": "123-456", "closed": false},
            ])
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "some-secret-token", "some-key");
        let result = Board::get_all(&client)?;
        let expected = vec![
            Board {
                name: String::from("TODO"),
                id: String::from("abc-def"),
                closed: false,
                lists: None,
            },
            Board {
                name: String::from("foo"),
                id: String::from("123-456"),
                closed: false,
                lists: None,
            },
        ];

        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_get() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "GET",
            "/1/boards/some-board-id?key=KEY&token=TOKEN&fields=id%2Cname%2Cclosed",
        )
        .with_status(200)
        .with_body(
            json!({
                "name": "My Favourite Board",
                "id": "some-board-id",
                "closed": false,
            })
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "TOKEN", "KEY");
        let result = Board::get(&client, "some-board-id")?;
        let expected = Board {
            name: String::from("My Favourite Board"),
            id: String::from("some-board-id"),
            closed: false,
            lists: None,
        };
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_get_all_lists() -> Result<(), Box<dyn Error>> {
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

        let client = Client::new(&mockito::server_url(), "some-token", "some-key");
        let result = Board::get_all_lists(&client, "some-board-id", false)?;
        let expected = vec![
            List {
                name: String::from("Red"),
                id: String::from("823-123"),
                cards: None,
                closed: false,
            },
            List {
                name: String::from("Green"),
                id: String::from("222-222"),
                cards: None,
                closed: false,
            },
        ];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_get_all_lists_with_cards() -> Result<(), Box<dyn Error>> {
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
                        {"id": "card1", "name": "apple", "desc": "", "closed": false},
                    ],
                },
            ])
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "some-token", "some-key");
        let result = Board::get_all_lists(&client, "some-board-id", true)?;
        let expected = vec![
            List {
                name: String::from("Red"),
                id: String::from("823-123"),
                cards: Some(vec![]),
                closed: false,
            },
            List {
                name: String::from("Green"),
                id: String::from("222-222"),
                cards: Some(vec![Card {
                    id: "card1".to_string(),
                    name: "apple".to_string(),
                    desc: "".to_string(),
                    closed: false,
                }]),
                closed: false,
            },
        ];
        assert_eq!(result, expected);
        Ok(())
    }
}
