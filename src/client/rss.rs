use std::fmt;

use crate::client;
use chrono::{DateTime, NaiveDateTime, Utc};
use log::info;
use reqwest::Url;
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

#[derive(Debug, Error)]
pub enum RssError {
    #[error("Context mismatch at ({0}, {1}): current context is {2}, but encountered a close tag for {3}")]
    ContextMismatch(u64, u64, Context, Context),
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

impl From<RssError> for FlamError {
    fn from(e: RssError) -> Self {
        FlamError {
            source: Box::new(e),
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
pub enum Context {
    Item,
    Title,
    Link,
    DcDate,
    DcCreator,
    Description,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Context::Item => "Item",
            Context::Title => "Title",
            Context::Link => "Link",
            Context::DcDate => "DcDate",
            Context::DcCreator => "DcCreator",
            Context::Description => "Description",
        };
        write!(f, "{}", text)
    }
}

impl Client {
    pub fn new(config: &client::http::Config) -> Result<Client, FlamError> {
        let client = client::http::Client::new(config)?;
        Ok(Client { client })
    }

    pub async fn get(&self, uri: Url) -> Result<Vec<Item>, FlamError> {
        let body = self.client.get(uri).await?;
        let mut reader = EventReader::from_str(&body);

        let mut result: Vec<Item> = Vec::new();
        let mut current_item = Item::default();
        let mut context_stack: Vec<Context> = Vec::new();
        loop {
            match reader.next() {
                Ok(XmlEvent::StartElement { name, .. }) => {
                    if let Some(context) = self.resolve_context(&name) {
                        // Prepare a new item to fill in on reading an open <item> tag
                        if context == Context::Item {
                            current_item = Item::default();
                        }

                        // Update the current context
                        context_stack.push(context);
                    }
                }
                Ok(XmlEvent::EndElement { name, .. }) => {
                    if let Some(context) = self.resolve_context(&name) {
                        // Current context should be ended by this closing element.
                        // If the closing tag is not matching with current context, RSS document is malformed.
                        let current_context = context_stack
                            .pop()
                            .expect("Current context must not be empty");
                        if current_context != context {
                            let pos = reader.position();
                            return Err(RssError::ContextMismatch(
                                pos.row,
                                pos.column,
                                current_context,
                                context,
                            )
                            .into());
                        }

                        if context == Context::Item {
                            result.push(current_item.clone());
                        }
                    }
                }
                Ok(XmlEvent::Characters(body)) => match context_stack.last() {
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
                Ok(XmlEvent::EndDocument) => break,
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
