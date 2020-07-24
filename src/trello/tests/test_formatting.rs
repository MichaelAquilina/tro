use super::*;

use formatting::header;

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
    assert_eq!(result, String::from("foo 🔴\n------"));
}
