use std::fmt::Debug;

pub trait TrelloObject: Debug {
    fn get_type() -> String;

    fn get_name(&self) -> &str;

    fn get_fields() -> &'static [&'static str];

    fn render(&self) -> String;
}
