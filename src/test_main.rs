use super::*;

mod test_get_trello_object {
    use super::*;
    use clap::ArgMatches;

    #[test]
    fn test_empty() -> Result<(), Box<dyn Error>> {
        let matches = ArgMatches::new();
        let client = Client::new("", "", "");

        let result = get_trello_object(&client, &matches)?;
        let expected = TrelloResult {
            board: None,
            list: None,
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
            Err(simple_error::SimpleError::new(
                "Object not found. Specify a more precise filter than 'foobar'"
            ))
        );
    }

    #[test]
    fn test_not_found() {
        let boards = vec![Card::new("red", "", "1")];
        let result = get_object_by_name(&boards, "foobar", false);

        assert_eq!(
            result,
            Err(simple_error::SimpleError::new(
                "Object not found. Specify a more precise filter than 'foobar'"
            ))
        );
    }

    #[test]
    fn test_more_than_one() {
        let boards = vec![Board::new("1", "red", None), Board::new("2", "red", None)];
        let result = get_object_by_name(&boards, "red", false);

        assert_eq!(
            result,
            Err(simple_error::SimpleError::new(
                "More than one object found. Specify a more precise filter than 'red'"
            ))
        );
    }

    #[test]
    fn test_found() -> Result<(), Box<dyn Error>> {
        let boards = vec![
            Board::new("33", "green", None),
            Board::new("R35", "red", None),
        ];
        let result = get_object_by_name(&boards, "red", false)?;

        let expected = &boards[1];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_case_insensitive() -> Result<(), Box<dyn Error>> {
        let boards = vec![List::new("R35", "red", None)];
        let result = get_object_by_name(&boards, "RED", true)?;

        let expected = &boards[0];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_regex() -> Result<(), Box<dyn Error>> {
        let boards = vec![Board::new("R35", "Red Green Blue 🖌️", None)];
        let result = get_object_by_name(&boards, "Red .*", false)?;

        let expected = &boards[0];
        assert_eq!(result, expected);
        Ok(())
    }
}
