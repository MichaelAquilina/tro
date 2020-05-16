use crate::trello_object::{Renderable, TrelloObject};

use serde::Deserialize;

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub id: String,
    pub full_name: String,
}

impl Renderable for Member {
    fn render(&self) -> String {
        return format!("{}", self.full_name);
    }
}

// https://developer.atlassian.com/cloud/trello/guides/rest-api/object-definitions/#action-object
#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub id: String,
    #[serde(rename = "type")]
    pub action_type: String, // TODO: Could this be an enum?
    pub date: String,
    pub member_creator: Member,
}

impl Renderable for Action {
    fn render(&self) -> String {
        format!(
            "{} on {} by {}",
            &self.action_type,
            &self.date,
            &self.member_creator.render()
        )
    }
}

impl TrelloObject for Action {
    fn get_type() -> String {
        String::from("Action")
    }

    fn get_name(&self) -> &str {
        &self.id
    }

    fn get_fields() -> &'static [&'static str] {
        &["id", "type", "date", "memberCreator"]
    }
}
