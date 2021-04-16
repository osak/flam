use crate::client;
use chrono::{DateTime, NaiveDateTime, Utc};
use log::info;
use reqwest::Url;
use std::error;
use thiserror::Error;
use xml::{
    common::Position,
    name::OwnedName,
    reader::{EventReader, XmlEvent},
};

use crate::error::FlamError;

/// RSS client
pub struct Client {
    client: client::http::Client,
}

#[derive(Debug, Clone)]
pub struct Item {
    title: String,
    link: String,
    created: DateTime<Utc>,
    author: String,
    description: String,
}

impl Default for Item {
    fn default() -> Item {
        Item {
            title: "".to_owned(),
            link: "".to_owned(),
            created: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
            author: "".to_owned(),
            description: "".to_owned(),
        }
    }
}

impl From<xml::reader::Error> for FlamError {
    fn from(e: xml::reader::Error) -> Self {
        FlamError {
            source: Box::new(e),
        }
    }
}

/// Current context of RSS document parsing
#[derive(Debug, PartialEq, Eq)]
enum Context {
    Item,
    Title,
    Link,
    DcDate,
    DcCreator,
    Description,
}

impl Client {
    pub fn new(config: &client::http::Config) -> Result<Client, FlamError> {
        let client = client::http::Client::new(config)?;
        Ok(Client { client })
    }

    pub async fn get(&self, uri: Url) -> Result<Vec<Item>, FlamError> {
        let body = self.client.get(uri).await?;
        let reader = EventReader::from_str(&body);

        let mut result: Vec<Item> = Vec::new();
        let mut current_item = Item::default();
        let mut current_context: Option<Context> = None;
        for e in reader {
            match e {
                Ok(XmlEvent::StartElement { name, .. }) => {
                    let context = self.resolve_context(&name);

                    // Prepare a new item to fill in on reading an open <item> tag
                    if context == Some(Context::Item) {
                        current_item = Item::default();
                    }

                    // Update current context
                    current_context = context.or(current_context);
                }
                Ok(XmlEvent::EndElement { name, .. }) => {
                    let context = self.resolve_context(&name);

                    match context {
                        Some(Context::Item) => result.push(current_item.clone()),
                        _ => (),
                    }

                    if context == current_context {
                        current_context = None;
                    }
                }
                Ok(XmlEvent::Characters(body)) => match &current_context {
                    Some(Context::Title) => current_item.title += body.trim(),
                    Some(Context::Link) => current_item.link += body.trim(),
                    Some(Context::DcDate) => {
                        if let Ok(t) = DateTime::parse_from_rfc3339(&body.trim()) {
                            current_item.created = t.with_timezone(&Utc);
                        }
                    }
                    Some(Context::DcCreator) => current_item.author += body.trim(),
                    Some(Context::Description) => current_item.description += body.trim(),
                    _ => (),
                },
                Err(err) => return Err(err.into()),
                _ => (),
            }
        }
        Ok(result)
    }

    fn resolve_context(&self, name: &OwnedName) -> Option<Context> {
        match name.namespace_ref() {
            Some("http://purl.org/dc/elements/1.1/") => match name.local_name.as_str() {
                "date" => Some(Context::DcDate),
                "creator" => Some(Context::DcCreator),
                _ => None,
            },
            _ => match name.local_name.as_str() {
                "item" => Some(Context::Item),
                "title" => Some(Context::Title),
                "link" => Some(Context::Link),
                "description" => Some(Context::Description),
                _ => None,
            },
        }
    }
}
