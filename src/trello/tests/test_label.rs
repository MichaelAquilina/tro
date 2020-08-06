use super::*;

#[test]
fn test_get_all() -> Result<()> {
    let _m = mockito::mock(
        "GET",
        "/1/boards/123-456/labels?key=some-key&token=some-token&fields=id%2Cname%2Ccolor",
    )
    .with_status(200)
    .with_body(
        json!([
            {"name": "Tech", "color": "purple", "id": "1"},
            {"name": "Bills", "color": "orange", "id": "2"},
        ])
        .to_string(),
    )
    .create();

    let config = ClientConfig::new(&mockito::server_url(), "some-token", "some-key");
    let client = TrelloClient::new(config);

    let result = Label::get_all(&client, "123-456")?;
    let expected = vec![
        Label::new("1", "Tech", "purple"),
        Label::new("2", "Bills", "orange"),
    ];

    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn test_apply() -> Result<()> {
    let _m = mockito::mock(
        "POST",
        "/1/cards/SOME-CARD-ID/idLabels?key=some-key&token=some-token",
    )
    .match_body("value=MY-LABEL-ID")
    .with_status(200)
    .with_body(json!({}).to_string())
    .create();

    let config = ClientConfig::new(&mockito::server_url(), "some-token", "some-key");
    let client = TrelloClient::new(config);

    Label::apply(&client, "SOME-CARD-ID", "MY-LABEL-ID")?;

    Ok(())
}

#[test]
fn test_remove() -> Result<()> {
    let _m = mockito::mock(
        "DELETE",
        "/1/cards/FOO-CARD/idLabels/BAR-LABEL?key=some-key&token=some-token",
    )
    .with_status(200)
    .create();

    let config = ClientConfig::new(&mockito::server_url(), "some-token", "some-key");
    let client = TrelloClient::new(config);

    Label::remove(&client, "FOO-CARD", "BAR-LABEL")?;

    Ok(())
}
