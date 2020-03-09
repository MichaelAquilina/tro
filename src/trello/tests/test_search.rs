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

    let client = Client::new(&mockito::server_url(), "some-token", "some-key");
    let result = search(&client, "foo", false)?;

    let expected = SearchResult {
        boards: vec![],
        cards: vec![],
    };

    assert_eq!(result, expected);
    Ok(())
}
