use std::fmt::Debug;

pub trait TrelloObject: Debug {
    fn get_type() -> String;

    fn get_name(&self) -> &str;

    fn get_fields() -> &'static [&'static str];
}

/// Provides the ability for an object to be rendered
/// to the command line
pub trait Renderable {
    /// Render aims to render as much detail as possible and
    /// can render to multiple lines
    fn render(&self) -> String;

    /// Simple render aims to output to a single line
    fn simple_render(&self) -> String;
}
