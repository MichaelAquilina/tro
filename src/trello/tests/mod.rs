mod test_attachment;
mod test_board;
mod test_card;
mod test_formatting;
mod test_label;
mod test_list;

use super::*;
use mockito;
use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;

type Result<T> = std::result::Result<T, TrelloError>;
