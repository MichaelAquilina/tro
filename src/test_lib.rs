use super::{header, Board, Card, Client, List, TrelloObject};
use colored::*;
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
    fn test_new() {
        let card = Card::new("A", "B", "C");
        let expected = Card {
            id: String::from("A"),
            name: String::from("B"),
            desc: String::from("C"),
            labels: None,
            closed: false,
        };
        assert_eq!(card, expected);
    }

    #[test]
    fn test_render() {
        let card = Card::new("aaaaa", "My Fav Card", "this is a nice card");

        let expected = "My Fav Card\n-----------\nthis is a nice card";
        assert_eq!(card.render(), expected);
    }

    #[test]
    fn test_create() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "POST",
            "/1/cards/?key=some-key&token=some-token&name=Laundry&desc=Desky&idList=FOOBAR",
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
        let result = Card::create(&client, "FOOBAR", &Card::new("", "Laundry", "Desky"))?;
        let expected = Card::new("88888", "Laundry", "");

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

        let mut card = Card::new("MY-CARD-ID", "Laundry", "hello");
        card.closed = true;

        let result = Card::update(&client, &card)?;
        assert_eq!(result, card);
        Ok(())
    }
}

mod list_tests {
    use super::*;

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
            Some(vec![Card::new("", "hello", ""), Card::new("", "world", "")]),
        );

        let expected = format!(
            "{}\n{}",
            "King Knight\n-----------".bold(),
            "* hello\n* world"
        );
        assert_eq!(list.render(), expected);
    }

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
        let expected = List::new("MTLDA", "Today", None);
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

        let mut list = List::new("MY-LIST-ID", "Today", None);
        list.closed = true;

        let result = List::update(&client, &list)?;
        assert_eq!(result, list);
        Ok(())
    }

    #[test]
    fn test_get_all_cards() -> Result<(), Box<dyn Error>> {
        let _m = mockito::mock(
            "GET",
            "/1/lists/DEADBEEF/cards/?key=some-key&token=some-secret-token&fields=id%2Cname%2Cdesc%2Clabels%2Cclosed",
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
            Card::new("abc-def", "Water the plants", ""),
            Card::new("123-456", "Feed the dog", "for a good boy"),
        ];

        assert_eq!(result, expected);
        Ok(())
    }
}

mod board_tests {
    use super::*;

    #[test]
    fn test_new() {
        let board = Board::new("888", "some board", Some(vec![]));
        let expected = Board {
            id: String::from("888"),
            name: String::from("some board"),
            lists: Some(vec![]),
            closed: false,
        };
        assert_eq!(board, expected);
    }

    #[test]
    fn test_render_no_lists() {
        let board = Board::new("", "Knights", None);
        let expected = "Knights\n=======".bold().to_string();
        assert_eq!(board.render(), expected);
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
        );

        let expected = format!(
            "{}\n\n{}\n\n{}",
            "Knights\n=======".bold(),
            "King\n----".bold(),
            "Shovel\n------".bold(),
        );
        assert_eq!(board.render(), expected);
    }

    #[test]
    fn test_render_lists_and_cards() {
        let board = Board::new(
            "",
            "Knights",
            Some(vec![
                List::new("", "King", None),
                List::new("", "Shovel", Some(vec![Card::new("", "Flare Wand", "")])),
            ]),
        );

        let expected = format!(
            "{}\n\n{}\n\n{}\n{}",
            "Knights\n=======".bold(),
            "King\n----".bold(),
            "Shovel\n------".bold(),
            "* Flare Wand",
        );
        assert_eq!(board.render(), expected);
    }

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
        let expected = Board::new("231dgfe4r343", "MY-TEST-BOARD", None);

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

        let mut board = Board::new("MY-BOARD-ID", "TODO", None);
        board.closed = true;

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
            Board::new("abc-def", "TODO", None),
            Board::new("123-456", "foo", None),
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
        let expected = Board::new("some-board-id", "My Favourite Board", None);
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
            List::new("823-123", "Red", None),
            List::new("222-222", "Green", None),
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
            List::new("823-123", "Red", Some(vec![])),
            List::new(
                "222-222",
                "Green",
                Some(vec![Card::new("card1", "apple", "")]),
            ),
        ];
        assert_eq!(result, expected);
        Ok(())
    }
}
