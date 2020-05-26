use super::*;

use mockito::Matcher;

#[test]
fn test_empty() -> Result<()> {
    let _m = mockito::mock("GET", "/1/search/")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("query".into(), "foo".into()),
            Matcher::UrlEncoded("partial".into(), "false".into()),
            Matcher::UrlEncoded("key".into(), "some-key".into()),
            Matcher::UrlEncoded("token".into(), "some-token".into()),
            Matcher::UrlEncoded("cards_limit".into(), "20".into()),
        ]))
        .with_status(200)
        .with_body(
            json!({
                "cards": [],
                "boards": [],
            })
            .to_string(),
        )
        .create();

    let options = SearchOptions {
        boards_limit: None,
        cards_limit: Some(20),
        partial: false,
    };

    let client = Client::new(&mockito::server_url(), "some-token", "some-key");
    let result = search(&client, "foo", &options)?;

    let expected = SearchResult {
        boards: vec![],
        cards: vec![],
    };

    assert_eq!(result, expected);
    Ok(())
}
