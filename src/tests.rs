use super::{Board, Card, Client, List};
use mockito;
use serde_json::json;
use std::error::Error;

mod list_tests {
    use super::*;

    #[test]
    fn test_get_all_cards() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "GET",
            "/1/lists/DEADBEEF/cards/?key=some-key&token=some-secret-token",
        )
        .with_status(200)
        .with_body(
            json!([
                {"name": "Water the plants", "id": "abc-def", "desc": ""},
                {"name": "Feed the dog", "id": "123-456", "desc": "for a good boy"},
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
            },
            Card {
                name: String::from("Feed the dog"),
                id: String::from("123-456"),
                desc: String::from("for a good boy"),
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
                "url": "https://example.com/1/2",
                "id": "231dgfe4r343",
            })
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "some-token", "some-key");
        let result = Board::create(&client, "MY-TEST-BOARD")?;
        let expected = Board {
            id: String::from("231dgfe4r343"),
            name: String::from("MY-TEST-BOARD"),
            url: String::from("https://example.com/1/2"),
            lists: None,
        };
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_get_all() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "GET",
            "/1/members/me/boards?key=some-key&token=some-secret-token&filter=open",
        )
        .with_status(200)
        .with_body(
            json!([
                {"name": "TODO", "id": "abc-def", "url": "http://bit.ly/12"},
                {"name": "foo", "id": "123-456", "url": ""},
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
                url: String::from("http://bit.ly/12"),
                lists: None,
            },
            Board {
                name: String::from("foo"),
                id: String::from("123-456"),
                url: String::from(""),
                lists: None,
            },
        ];

        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_get() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock("GET", "/1/boards/some-board-id?key=KEY&token=TOKEN")
            .with_status(200)
            .with_body(
                json!({
                    "name": "My Favourite Board",
                    "id": "some-board-id",
                    "url": "https://trello.com/boards/some-board-id",
                })
                .to_string(),
            )
            .create();

        let client = Client::new(&mockito::server_url(), "TOKEN", "KEY");
        let result = Board::get(&client, "some-board-id")?;
        let expected = Board {
            name: String::from("My Favourite Board"),
            id: String::from("some-board-id"),
            url: String::from("https://trello.com/boards/some-board-id"),
            lists: None,
        };
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_get_all_lists() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "GET",
            "/1/boards/some-board-id/lists?key=some-key&token=some-token&cards=open",
        )
        .with_status(200)
        .with_body(
            json!([
                {"name": "Red", "id": "823-123"},
                {"name": "Green", "id": "222-222"},
            ])
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "some-token", "some-key");
        let result = Board::get_all_lists(&client, "some-board-id")?;
        let expected = vec![
            List {
                name: String::from("Red"),
                id: String::from("823-123"),
                cards: None,
            },
            List {
                name: String::from("Green"),
                id: String::from("222-222"),
                cards: None,
            },
        ];
        assert_eq!(result, expected);
        Ok(())
    }
}
