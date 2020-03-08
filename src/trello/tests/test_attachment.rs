use super::*;

#[test]
fn test_get_all() -> Result<()> {
    let _m = mockito::mock(
        "GET",
        "/1/cards/FOO-CARD/attachments?key=sekret&token=my-token&fields=id%2Cname%2Curl",
    )
    .with_status(200)
    .with_body(
        json!([{
            "name": "IMG_2000.png",
            "id": "0012310",
            "url": "https://example.com/1/12/IMG_2000.png",
        }])
        .to_string(),
    )
    .create();

    let client = Client::new(&mockito::server_url(), "my-token", "sekret");
    let result = Attachment::get_all(&client, "FOO-CARD")?;

    let expected = [Attachment {
        id: String::from("0012310"),
        name: String::from("IMG_2000.png"),
        url: String::from("https://example.com/1/12/IMG_2000.png"),
    }];

    assert_eq!(result, expected);

    Ok(())
}

#[test]
fn test_apply() -> Result<()> {
    let _m = mockito::mock("POST", "/1/cards/CARD-23/attachments?key=KEY&token=TOKEN")
        .with_status(200)
        .with_body(
            json!({
                "id": "my-attachment",
                "name": "My Attachment",
                "url": "https://some-example.com/attachment.txt",
            })
            .to_string(),
        )
        .create();

    let mut file1 = NamedTempFile::new()?;
    file1.write_all("some data".as_bytes())?;

    let path = file1.path().to_str().unwrap();

    let client = Client::new(&mockito::server_url(), "TOKEN", "KEY");

    let result = Attachment::apply(&client, "CARD-23", path)?;

    assert_eq!(
        result,
        Attachment {
            id: String::from("my-attachment"),
            name: String::from("My Attachment"),
            url: String::from("https://some-example.com/attachment.txt"),
        }
    );

    Ok(())
}
