use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use ego_tree::NodeRef;
use log::{info, warn};
use scraper::element_ref::ElementRef;
use scraper::html::Html;
use scraper::node::Node;
use scraper::selector::Selector;
use selectors::attr::CaseSensitivity::CaseSensitive;

use crate::entry::Entry;

#[derive(Debug)]
pub struct Lwn {
    title: String,
    summary: String,
}

pub type LwnEntry = Entry<Lwn>;

pub fn parse(raw: &str) -> Vec<LwnEntry> {
    let document = Html::parse_document(raw);
    let selector = Selector::parse(".BlurbListing").expect("failed to parse selector");
    document
        .select(&selector)
        .map(to_entry)
        .filter(|e| e.is_some())
        .map(|e| e.unwrap())
        .collect()
}

fn to_entry(blurb: ElementRef) -> Option<LwnEntry> {
    let headline = prev_sibling_tag(&blurb)
        .map(|n| extract_headline(&n))
        .flatten();
    let title = match headline {
        Some(text) => text,
        _ => {
            info!("Headline is not found for current blurb. Skip processing it.");
            return None;
        }
    };

    let summary = match extract_summary(&blurb) {
        Some(text) => text,
        _ => {
            info!(
                "Summary is not found for blurb `{}`. Skip processing it.",
                title
            );
            return None;
        }
    };

    let created = match extract_created(&blurb) {
        Some(ts) => ts,
        _ => {
            info!(
                "Timestamp cannot be parsed for blurb `{}`. Skip processing it.",
                title
            );
            return None;
        }
    };

    Some(LwnEntry {
        source: "lwn.net",
        ref_id: "aaa".to_owned(),
        created: created,
        last_update: created,
        data: Lwn { title, summary },
    })
}

fn extract_headline(node: &NodeRef<Node>) -> Option<String> {
    let is_headline = match *node.value() {
        Node::Element(ref e) => e.has_class("Headline", CaseSensitive),
        _ => false,
    };
    if is_headline {
        let eref = ElementRef::wrap(*node).expect("node must be Element");
        Some(eref.inner_html())
    } else {
        None
    }
}

fn extract_summary(blurb: &ElementRef) -> Option<String> {
    blurb
        .first_child()
        .map(|n| skip_until_tag(&n))
        .flatten()
        .map(|n| n.next_sibling())
        .flatten()
        .map(|n| skip_until_tag(&n))
        .flatten()
        .map(|e| e.inner_html().trim().to_owned())
}

fn extract_created(blurb: &ElementRef) -> Option<DateTime<Utc>> {
    blurb
        .first_child()
        .map(|n| skip_until_tag(&n))
        .flatten()
        .map(|e| e.inner_html())
        .map(|text| extract_timestamp(&text))
        .flatten()
}

fn extract_timestamp(text: &str) -> Option<DateTime<Utc>> {
    // Example string: [Security] Posted Jan 15, 2021 17:24 UTC (Fri) by jake
    let start = match text.find("Posted ") {
        Some(p) => p + "Posted ".len(),
        None => {
            warn!("Cannot find the beginning of timestamp in `{}`", text);
            return None;
        }
    };
    let end = match text.find(" by ") {
        Some(p) => p,
        _ => {
            warn!("Cannot find the end of timestamp in `{}`", text);
            return None;
        }
    };

    let ts_text = format!("{} 00", &text[start..end]);
    info!("{}", ts_text);
    let timestamp = NaiveDateTime::parse_from_str(&ts_text, "%b %d, %Y %H:%M %Z (%a) %S")
        .map(|t| Utc.from_utc_datetime(&t));
    match timestamp {
        Ok(t) => Some(t),
        Err(e) => {
            warn!("Failed to parse timestamp text `{}`. Reason: {}", text, e);
            return None;
        }
    }
}

fn prev_sibling_tag<'a>(eref: &ElementRef<'a>) -> Option<ElementRef<'a>> {
    let mut cur = eref.prev_sibling();
    while cur.is_some() {
        let cur_node = cur.unwrap();
        match cur_node.value() {
            Node::Element(_) => return ElementRef::wrap(cur_node),
            _ => {
                cur = cur_node.prev_sibling();
            }
        }
    }
    None
}

fn skip_until_tag<'a>(nref: &NodeRef<'a, Node>) -> Option<ElementRef<'a>> {
    let mut cur = Some(*nref);
    while cur.is_some() {
        let cur_node = cur.unwrap();
        match cur_node.value() {
            Node::Element(_) => return ElementRef::wrap(cur_node),
            _ => {
                cur = cur_node.next_sibling();
            }
        }
    }
    None
}
