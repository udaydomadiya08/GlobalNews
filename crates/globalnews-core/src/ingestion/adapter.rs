use super::error::IngestionError;
use globalnews_contracts::FeedFormat;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde_json::Value;
use std::collections::HashMap;

/// Raw, unnormalized news document or bulletin item extracted directly from a feed payload.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct RawFeedItem {
    pub guid: Option<String>,
    pub title: String,
    pub link: String,
    pub description: Option<String>,
    pub content: Option<String>,
    pub pub_date_raw: Option<String>,
    pub authors: Vec<String>,
    pub categories: Vec<String>,
    pub extra_metadata: HashMap<String, String>,
}

/// Extensible interface implemented by feed parsers for different formats.
pub trait FeedAdapter: Send + Sync {
    fn format(&self) -> FeedFormat;
    fn parse(&self, content: &[u8], base_url: &str) -> Result<Vec<RawFeedItem>, IngestionError>;
}

// ===========================================================================
// 1. RSS 2.0 Adapter
// ===========================================================================

pub struct Rss2Adapter;

impl FeedAdapter for Rss2Adapter {
    fn format(&self) -> FeedFormat {
        FeedFormat::Rss2
    }

    fn parse(&self, content: &[u8], base_url: &str) -> Result<Vec<RawFeedItem>, IngestionError> {
        // Guard against malicious XXE DOCTYPE payloads
        let content_str = String::from_utf8_lossy(content);
        if content_str.contains("<!ENTITY") || content_str.contains("SYSTEM \"http") {
            return Err(IngestionError::XmlParse("Malicious XML entity expansion (XXE) detected".into()));
        }

        let mut reader = Reader::from_reader(content);
        reader.config_mut().trim_text(true);

        let mut items = Vec::new();
        let mut in_item = false;
        let mut current_item = RawFeedItem::default();
        let mut current_tag = String::new();
        let mut text_buffer = String::new();
        let mut open_tags: Vec<String> = Vec::new();

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    if tag_name.eq_ignore_ascii_case("item") {
                        in_item = true;
                        current_item = RawFeedItem::default();
                    }
                    current_tag = tag_name.clone();
                    open_tags.push(tag_name);
                    text_buffer.clear();
                }
                Ok(Event::Text(ref e)) => {
                    if in_item {
                        if let Ok(t) = e.unescape() {
                            text_buffer.push_str(&t);
                        }
                    }
                }
                Ok(Event::CData(ref e)) => {
                    if in_item {
                        let text = String::from_utf8_lossy(e.as_ref()).to_string();
                        text_buffer.push_str(&text);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    if let Some(pos) = open_tags.iter().rposition(|t| t.eq_ignore_ascii_case(&tag_name)) {
                        open_tags.truncate(pos);
                    }
                    if in_item {
                        let val = text_buffer.trim().to_string();
                        match tag_name.to_ascii_lowercase().as_str() {
                            "title" => current_item.title = val,
                            "link" => current_item.link = val,
                            "description" => current_item.description = Some(val),
                            "pubdate" | "date" => current_item.pub_date_raw = Some(val),
                            "guid" => current_item.guid = Some(val),
                            "author" | "creator" => {
                                if !val.is_empty() {
                                    current_item.authors.push(val);
                                }
                            }
                            "category" => {
                                if !val.is_empty() {
                                    current_item.categories.push(val);
                                }
                            }
                            "item" => {
                                in_item = false;
                                if !current_item.title.is_empty() || !current_item.link.is_empty() {
                                    if current_item.link.is_empty() && !base_url.is_empty() {
                                        current_item.link = base_url.to_string();
                                    }
                                    items.push(current_item.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                    current_tag.clear();
                    text_buffer.clear();
                }
                Ok(Event::Eof) => {
                    if in_item || !open_tags.is_empty() {
                        return Err(IngestionError::XmlParse("Malformed RSS XML: unclosed tags at EOF".into()));
                    }
                    break;
                }
                Err(e) => return Err(IngestionError::XmlParse(format!("Malformed RSS XML: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(items)
    }
}

// ===========================================================================
// 2. Atom 1.0 Adapter
// ===========================================================================

pub struct AtomAdapter;

impl FeedAdapter for AtomAdapter {
    fn format(&self) -> FeedFormat {
        FeedFormat::Atom
    }

    fn parse(&self, content: &[u8], base_url: &str) -> Result<Vec<RawFeedItem>, IngestionError> {
        let content_str = String::from_utf8_lossy(content);
        if content_str.contains("<!ENTITY") || content_str.contains("SYSTEM \"http") {
            return Err(IngestionError::XmlParse("Malicious XML entity expansion (XXE) detected".into()));
        }

        let mut reader = Reader::from_reader(content);
        reader.config_mut().trim_text(true);

        let mut items = Vec::new();
        let mut in_entry = false;
        let mut current_item = RawFeedItem::default();
        let mut current_tag = String::new();
        let mut text_buffer = String::new();
        let mut open_tags: Vec<String> = Vec::new();

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    if tag_name.eq_ignore_ascii_case("entry") {
                        in_entry = true;
                        current_item = RawFeedItem::default();
                    }
                    current_tag = tag_name.clone();
                    open_tags.push(tag_name);
                    text_buffer.clear();
                }
                Ok(Event::Empty(ref e)) => {
                    // Atom links are frequently <link href="..." rel="alternate"/>
                    let tag_name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    if in_entry && tag_name.eq_ignore_ascii_case("link") {
                        let mut href = String::new();
                        let mut is_alternate = false;
                        let mut has_rel = false;

                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            if key.eq_ignore_ascii_case("href") {
                                href = val;
                            } else if key.eq_ignore_ascii_case("rel") {
                                has_rel = true;
                                if val.eq_ignore_ascii_case("alternate") {
                                    is_alternate = true;
                                }
                            }
                        }

                        if !href.is_empty() && (!has_rel || is_alternate || current_item.link.is_empty()) {
                            current_item.link = href;
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_entry {
                        if let Ok(t) = e.unescape() {
                            text_buffer.push_str(&t);
                        }
                    }
                }
                Ok(Event::CData(ref e)) => {
                    if in_entry {
                        let text = String::from_utf8_lossy(e.as_ref()).to_string();
                        text_buffer.push_str(&text);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    if let Some(pos) = open_tags.iter().rposition(|t| t.eq_ignore_ascii_case(&tag_name)) {
                        open_tags.truncate(pos);
                    }
                    if in_entry {
                        let val = text_buffer.trim().to_string();
                        match tag_name.to_ascii_lowercase().as_str() {
                            "title" => current_item.title = val,
                            "id" => current_item.guid = Some(val),
                            "summary" => current_item.description = Some(val),
                            "content" => current_item.content = Some(val),
                            "published" => current_item.pub_date_raw = Some(val),
                            "updated" => {
                                if current_item.pub_date_raw.is_none() {
                                    current_item.pub_date_raw = Some(val);
                                }
                            }
                            "name" => {
                                if !val.is_empty() {
                                    current_item.authors.push(val);
                                }
                            }
                            "category" => {
                                if !val.is_empty() {
                                    current_item.categories.push(val);
                                }
                            }
                            "entry" => {
                                in_entry = false;
                                if !current_item.title.is_empty() || !current_item.link.is_empty() {
                                    if current_item.link.is_empty() && !base_url.is_empty() {
                                        current_item.link = base_url.to_string();
                                    }
                                    if current_item.description.is_none() {
                                        current_item.description = current_item.content.clone();
                                    }
                                    items.push(current_item.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                    current_tag.clear();
                    text_buffer.clear();
                }
                Ok(Event::Eof) => {
                    if in_entry || !open_tags.is_empty() {
                        return Err(IngestionError::XmlParse("Malformed Atom XML: unclosed tags at EOF".into()));
                    }
                    break;
                }
                Err(e) => return Err(IngestionError::XmlParse(format!("Malformed Atom XML: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(items)
    }
}

// ===========================================================================
// 3. USGS Earthquake GeoJSON Adapter
// ===========================================================================

pub struct UsgsEarthquakeAdapter;

impl FeedAdapter for UsgsEarthquakeAdapter {
    fn format(&self) -> FeedFormat {
        FeedFormat::JsonStructured
    }

    fn parse(&self, content: &[u8], base_url: &str) -> Result<Vec<RawFeedItem>, IngestionError> {
        let json: Value = serde_json::from_slice(content)
            .map_err(|e| IngestionError::JsonParse(format!("Failed to parse USGS GeoJSON: {}", e)))?;

        let features = json
            .get("features")
            .and_then(|f| f.as_array())
            .ok_or_else(|| IngestionError::JsonParse("Missing features array in GeoJSON".into()))?;

        let mut items = Vec::new();

        for feature in features {
            let props = match feature.get("properties") {
                Some(p) => p,
                None => continue,
            };

            let title = props
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("Earthquake Alert")
                .to_string();

            let url = props
                .get("url")
                .and_then(|u| u.as_str())
                .unwrap_or(base_url)
                .to_string();

            let mag = props.get("mag").and_then(|m| m.as_f64()).unwrap_or(0.0);
            let place = props.get("place").and_then(|p| p.as_str()).unwrap_or("Unknown location");

            let time_ms = props.get("time").and_then(|t| t.as_i64()).unwrap_or(0);
            let pub_date_raw = if time_ms > 0 {
                chrono::DateTime::from_timestamp_millis(time_ms)
                    .map(|dt| dt.to_rfc3339())
            } else {
                None
            };

            let guid = feature
                .get("id")
                .and_then(|id| id.as_str())
                .map(|s| s.to_string());

            let description = format!(
                "Magnitude {:.1} earthquake detected at {}. Source: USGS Earthquake Hazards Program.",
                mag, place
            );

            let mut extra = HashMap::new();
            extra.insert("magnitude".into(), mag.to_string());
            extra.insert("place".into(), place.to_string());

            if let Some(coords) = feature
                .get("geometry")
                .and_then(|g| g.get("coordinates"))
                .and_then(|c| c.as_array())
            {
                if coords.len() >= 2 {
                    extra.insert("longitude".into(), coords[0].to_string());
                    extra.insert("latitude".into(), coords[1].to_string());
                }
            }

            items.push(RawFeedItem {
                guid,
                title,
                link: url,
                description: Some(description),
                content: None,
                pub_date_raw,
                authors: vec!["USGS Earthquake Hazards Program".into()],
                categories: vec!["earthquake".into(), "seismic".into(), "natural_disaster".into()],
                extra_metadata: extra,
            });
        }

        Ok(items)
    }
}
