use std::fmt::Debug;

pub trait TrelloObject: Debug {
    fn get_type() -> String;

    fn get_name(&self) -> &str;

    fn get_fields() -> &'static [&'static str];
}

/// Provides the ability for an object to be rendered
/// to the command line
pub trait Renderable {
    fn render(&self) -> String;

    fn simple_render(&self) -> String;
}
