use super::*;

mod test_get_object_by_name {
    use super::*;

    #[test]
    fn test_empty() {
        let boards: Vec<Board> = vec![];
        let result = get_object_by_name(boards, "foobar", false).expect("");

        assert_eq!(result, None);
    }

    #[test]
    fn test_not_found() {
        let boards = vec![Card {
            name: "red".to_string(),
            desc: "".to_string(),
            id: "1".to_string(),
            closed: false,
        }];
        let result = get_object_by_name(boards, "foobar", false).expect("");

        assert_eq!(result, None);
    }

    #[test]
    fn test_more_than_one() {
        let boards = vec![
            Board {
                name: "red".to_string(),
                id: "1".to_string(),
                closed: false,
            },
            Board {
                name: "red".to_string(),
                id: "2".to_string(),
                closed: false,
            },
        ];
        let result = get_object_by_name(boards, "red", false);

        assert_eq!(
            result,
            Err(simple_error::SimpleError::new(
                "More than one object found for 'red'. Specify a more precise filter"
            ))
        );
    }

    #[test]
    fn test_found() {
        let boards = vec![Board {
            name: "red".to_string(),
            id: "R35".to_string(),
            closed: false,
        }];
        let result = get_object_by_name(boards, "red", false).expect("");

        let expected = Board {
            name: "red".to_string(),
            id: "R35".to_string(),
            closed: false,
        };

        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_case_insensitive() {
        let boards = vec![List {
            name: "red".to_string(),
            id: "R35".to_string(),
            closed: false,
        }];
        let result = get_object_by_name(boards, "RED", true).expect("");

        let expected = List {
            name: "red".to_string(),
            id: "R35".to_string(),
            closed: false,
        };

        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_regex() {
        let boards = vec![Board {
            name: "Red Green Blue 🖌️".to_string(),
            id: "R35".to_string(),
            closed: false,
        }];
        let result = get_object_by_name(boards, "Red .*", false).expect("");

        let expected = Board {
            name: "Red Green Blue 🖌️".to_string(),
            id: "R35".to_string(),
            closed: false,
        };

        assert_eq!(result, Some(expected));
    }
}
