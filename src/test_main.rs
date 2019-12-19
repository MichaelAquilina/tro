use super::*;

mod test_get_board_by_name {
    use super::*;
    use mockito;
    use serde_json::json;

    #[test]
    fn test_empty() {
        let _m = mockito::mock(
            "GET",
            "/1/members/me/boards/?key=KEY&token=TOKEN&filter=open",
        )
        .with_status(200)
        .with_body(json!([]).to_string())
        .create();

        let client = Client::new(&mockito::server_url(), "TOKEN", "KEY");

        let result = get_board_by_name(&client, "foobar").expect("");

        assert_eq!(result, None);
    }

    #[test]
    fn test_not_found() {
        let _m = mockito::mock(
            "GET",
            "/1/members/me/boards/?key=KEY&token=TOKEN&filter=open",
        )
        .with_status(200)
        .with_body(
            json!([{
                "name": "red",
                "id": "R35",
                "url": "",
                "closed": false,
            }])
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "TOKEN", "KEY");

        let result = get_board_by_name(&client, "foobar").expect("");

        assert_eq!(result, None);
    }

    #[test]
    fn test_more_than_one() {
        let _m = mockito::mock(
            "GET",
            "/1/members/me/boards/?key=KEY&token=TOKEN&filter=open",
        )
        .with_status(200)
        .with_body(
            json!([
                {
                    "name": "red",
                    "id": "R35",
                    "url": "",
                    "closed": false,
                },
                {
                    "name": "red",
                    "id": "R44",
                    "url": "",
                    "closed": false,
                }
            ])
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "TOKEN", "KEY");

        let result = get_board_by_name(&client, "red");

        // TODO How do I correctly assert type of Error and value?
        assert!(result.is_err());
    }

    #[test]
    fn test_found() {
        let _m = mockito::mock(
            "GET",
            "/1/members/me/boards/?key=KEY&token=TOKEN&filter=open",
        )
        .with_status(200)
        .with_body(
            json!([{
                "name": "red",
                "id": "R35",
                "url": "",
                "closed": false,
            }])
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "TOKEN", "KEY");

        let result = get_board_by_name(&client, "red").expect("");

        let expected = Board {
            name: "red".to_string(),
            id: "R35".to_string(),
            url: "".to_string(),
            closed: false,
            lists: None,
        };
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_regex() {
        let _m = mockito::mock(
            "GET",
            "/1/members/me/boards/?key=KEY&token=TOKEN&filter=open",
        )
        .with_status(200)
        .with_body(
            json!([{
                "name": "Red Green Blue 🖌️",
                "id": "R35",
                "url": "",
                "closed": false,
            }])
            .to_string(),
        )
        .create();

        let client = Client::new(&mockito::server_url(), "TOKEN", "KEY");

        let result = get_board_by_name(&client, "Red .*").expect("");

        let expected = Board {
            name: "Red Green Blue 🖌️".to_string(),
            id: "R35".to_string(),
            url: "".to_string(),
            closed: false,
            lists: None,
        };
        assert_eq!(result, Some(expected));
    }
}
