use crate::find::*;
use std::error::Error;
use trello::{Board, Card, ClientConfig, List, TrelloClient};

type TestResult = Result<(), Box<dyn Error>>;

mod test_get_trello_object {
    use super::*;
    use mockito;
    use serde_json::json;

    #[test]
    fn test_empty() -> TestResult {
        let params = TrelloParams {
            board_name: None,
            list_name: None,
            card_name: None,
            ignore_case: false,
        };
        let config = ClientConfig::new("", "", "");
        let client = TrelloClient::new(config);

        let result = get_trello_object(&client, &params)?;
        let expected = TrelloResult {
            board: None,
            list: None,
            card: None,
        };
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_correct_output() -> TestResult {
        let _m1 = mockito::mock(
            "GET",
            "/1/members/me/boards/?key=key&token=token&filter=open&fields=id%2Cname%2Cclosed%2Curl",
        )
        .with_status(200)
        .with_body(
            json!([
                {"name": "TODO", "id": "abc-def", "closed": false, "url": ""},
                {"name": "foo", "id": "123-456", "closed": false, "url": ""},
            ])
            .to_string(),
        )
        .create();

        let _m2 = mockito::mock(
            "GET",
            "/1/boards/abc-def/lists?key=key&token=token&fields=id%2Cname%2Cclosed&cards=open",
        )
        .with_status(200)
        .with_body(
            json!([
                {"name": "Backlog", "id": "bcklg", "closed": false, "cards": [], "url": ""},
            ])
            .to_string(),
        )
        .create();

        let params = TrelloParams {
            board_name: Some("TODO"),
            list_name: Some("back"),
            card_name: None,
            ignore_case: true,
        };
        let config = ClientConfig::new(&mockito::server_url(), "token", "key");
        let client = TrelloClient::new(config);

        let result = get_trello_object(&client, &params)?;
        let expected = TrelloResult {
            board: Some(Board::new(
                "abc-def",
                "TODO",
                Some(vec![List::new("bcklg", "Backlog", Some(vec![]))]),
                "",
            )),
            list: Some(List::new("bcklg", "Backlog", Some(vec![]))),
            card: None,
        };

        assert_eq!(result, expected);

        Ok(())
    }
}

mod test_get_object_by_name {
    use super::*;

    #[test]
    fn test_empty() {
        let boards: Vec<Board> = vec![];
        let result = get_object_by_name(&boards, "foobar", false);

        assert_eq!(
            result,
            Err(FindError::NotFound(
                "Board not found. Specify a more precise filter than 'foobar'".to_string()
            ))
        );
    }

    #[test]
    fn test_not_found() {
        let boards = vec![Card::new("red", "", "1", None, "", None)];
        let result = get_object_by_name(&boards, "foobar", false);

        assert_eq!(
            result,
            Err(FindError::NotFound(
                "Card not found. Specify a more precise filter than 'foobar'".to_string()
            ))
        );
    }

    #[test]
    fn test_more_than_one() {
        let boards = vec![
            Board::new("1", "red", None, ""),
            Board::new("2", "reddish", None, ""),
        ];
        let result = get_object_by_name(&boards, "red", false);

        assert_eq!(
            result,
            Err(FindError::Multiple(
                "More than one Board found. Specify a more precise filter than 'red' (Found 'red', 'reddish')".to_string()
            ))
        );
    }

    #[test]
    fn test_found() -> TestResult {
        let boards = vec![
            Board::new("33", "green", None, ""),
            Board::new("R35", "red", None, ""),
        ];
        let result = get_object_by_name(&boards, "red", false)?;

        let expected = &boards[1];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_case_insensitive() -> TestResult {
        let boards = vec![List::new("R35", "red", None)];
        let result = get_object_by_name(&boards, "RED", true)?;

        let expected = &boards[0];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_regex() -> TestResult {
        let boards = vec![Board::new("R35", "Red Green Blue 🖌️", None, "")];
        let result = get_object_by_name(&boards, "Red .*", false)?;

        let expected = &boards[0];
        assert_eq!(result, expected);
        Ok(())
    }
}
