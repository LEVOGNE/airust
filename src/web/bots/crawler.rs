use sha2::{Digest, Sha256};
use std::sync::Arc;

use super::models::BotConfig;
use crate::web::db::Database;

pub struct CrawlerBot;

#[derive(Debug, Clone)]
pub struct CrawledPage {
    pub url: String,
    pub title: String,
    pub content: String,
    pub content_hash: String,
}

impl CrawlerBot {
    pub async fn run(
        config: &BotConfig,
        db: &Arc<Database>,
        run_id: i64,
    ) -> Result<(i64, i64), String> {
        if config.max_depth > 5 {
            return Err("Crawler depth is limited to 5".to_owned());
        }
        let mut items_found: i64 = 0;
        let mut items_added: i64 = 0;

        match config.mode.as_str() {
            "single_page" => {
                let page = Self::fetch_page(&config.url).await?;
                items_found = 1;
                let inserted = db.insert_raw_data(
                    run_id,
                    Some(&page.url),
                    Some(&page.title),
                    &page.content,
                    &page.content_hash,
                )?;
                if inserted > 0 {
                    items_added = 1;
                }
            }
            "links_follow" => {
                let mut visited = std::collections::HashSet::new();
                let mut queue = vec![(config.url.clone(), 0u32)];

                while let Some((url, depth)) = queue.pop() {
                    if depth > config.max_depth || visited.contains(&url) {
                        continue;
                    }
                    if visited.len() >= 100 {
                        break;
                    }
                    visited.insert(url.clone());

                    match Self::fetch_page(&url).await {
                        Ok(page) => {
                            items_found += 1;
                            let inserted = db.insert_raw_data(
                                run_id,
                                Some(&page.url),
                                Some(&page.title),
                                &page.content,
                                &page.content_hash,
                            )?;
                            if inserted > 0 {
                                items_added += 1;
                            }

                            if depth < config.max_depth {
                                if let Ok(links) = Self::extract_links(&url, &page.content).await {
                                    for link in links.into_iter().take(100) {
                                        if !visited.contains(&link) {
                                            queue.push((link, depth + 1));
                                        }
                                    }
                                }
                            }

                            tokio::time::sleep(std::time::Duration::from_millis(
                                config.rate_limit_ms.clamp(100, 30_000),
                            ))
                            .await;
                        }
                        Err(e) => {
                            eprintln!("Crawl error for {}: {}", url, e);
                        }
                    }
                }
            }
            _ => {
                let page = Self::fetch_page(&config.url).await?;
                items_found = 1;
                let inserted = db.insert_raw_data(
                    run_id,
                    Some(&page.url),
                    Some(&page.title),
                    &page.content,
                    &page.content_hash,
                )?;
                if inserted > 0 {
                    items_added = 1;
                }
            }
        }

        Ok((items_found, items_added))
    }

    async fn fetch_page(url: &str) -> Result<CrawledPage, String> {
        let html = fetch_public_html(url).await?;

        let document = scraper::Html::parse_document(&html);

        let title_selector = scraper::Selector::parse("title").unwrap();
        let title = document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        let body_selector = scraper::Selector::parse("body").unwrap();
        let skip_tags = [
            "script", "style", "nav", "header", "footer", "noscript", "aside", "form", "button",
            "svg", "canvas", "iframe", "select", "input",
        ];

        let mut text_parts = Vec::new();
        if let Some(body) = document.select(&body_selector).next() {
            Self::extract_text(body, &skip_tags, &mut text_parts);
        }

        let raw_content = text_parts.join("\n").trim().to_string();
        let content = Self::clean_text(&raw_content);

        if content.is_empty() {
            return Err("Content empty after cleaning (likely binary data)".to_string());
        }

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let content_hash = format!("{:x}", hasher.finalize());

        Ok(CrawledPage {
            url: url.to_string(),
            title,
            content,
            content_hash,
        })
    }

    fn clean_text(raw: &str) -> String {
        let total_chars = raw.chars().count();
        if total_chars > 0 {
            let replacement_count = raw.chars().filter(|&c| c == '\u{FFFD}').count();
            if replacement_count as f64 / total_chars as f64 > 0.10 {
                return String::new();
            }

            let non_printable = raw
                .chars()
                .filter(|c| {
                    !c.is_alphanumeric()
                        && !c.is_whitespace()
                        && !c.is_ascii_punctuation()
                        && *c != 'ä'
                        && *c != 'ö'
                        && *c != 'ü'
                        && *c != 'Ä'
                        && *c != 'Ö'
                        && *c != 'Ü'
                        && *c != 'ß'
                        && *c != 'é'
                        && *c != 'è'
                        && *c != 'à'
                        && *c != 'ñ'
                })
                .count();
            if non_printable as f64 / total_chars as f64 > 0.10 {
                return String::new();
            }
        }

        let junk_patterns = [
            "Bild wird geladen",
            "Cookie",
            "cookie",
            "Loading...",
            "loading...",
            "Accept all",
            "Alle akzeptieren",
            "Alle ablehnen",
            "Mehr erfahren",
            "Read more",
            "JavaScript",
            "javascript",
            "Enable JavaScript",
            "Zur Hauptnavigation",
            "Zum Inhalt springen",
            "Skip to content",
            "Skip to main",
        ];

        let lines: Vec<String> = raw
            .lines()
            .map(|line| {
                let mut result = String::new();
                let mut prev_space = false;
                for c in line.trim().chars() {
                    if c.is_whitespace() {
                        if !prev_space {
                            result.push(' ');
                            prev_space = true;
                        }
                    } else {
                        result.push(c);
                        prev_space = false;
                    }
                }
                result
            })
            .filter(|line| {
                if line.len() < 3 {
                    return false;
                }

                for pattern in &junk_patterns {
                    if line.contains(pattern) {
                        return false;
                    }
                }
                true
            })
            .collect();

        let mut result = String::new();
        let mut prev_empty = false;
        for line in &lines {
            if line.trim().is_empty() {
                if !prev_empty {
                    result.push('\n');
                    prev_empty = true;
                }
            } else {
                result.push_str(line);
                result.push('\n');
                prev_empty = false;
            }
        }

        result.trim().to_string()
    }

    fn extract_text(element: scraper::ElementRef, skip_tags: &[&str], parts: &mut Vec<String>) {
        for child in element.children() {
            match child.value() {
                scraper::Node::Text(text) => {
                    let t = text.trim();
                    if !t.is_empty() {
                        parts.push(t.to_string());
                    }
                }
                scraper::Node::Element(el) if !skip_tags.contains(&el.name()) => {
                    if let Some(child_ref) = scraper::ElementRef::wrap(child) {
                        Self::extract_text(child_ref, skip_tags, parts);
                    }
                }
                _ => {}
            }
        }
    }

    async fn extract_links(base_url: &str, _content: &str) -> Result<Vec<String>, String> {
        let html = fetch_public_html(base_url).await?;

        let document = scraper::Html::parse_document(&html);
        let a_selector = scraper::Selector::parse("a[href]").unwrap();
        let base = url::Url::parse(base_url).map_err(|e| format!("URL parse error: {}", e))?;

        let mut links = Vec::new();
        for el in document.select(&a_selector) {
            if let Some(href) = el.value().attr("href") {
                if let Ok(resolved) = base.join(href) {
                    if resolved.host() == base.host() {
                        links.push(resolved.to_string());
                    }
                }
            }
        }

        Ok(links)
    }
}

pub fn public_address(address: std::net::IpAddr) -> bool {
    match address {
        std::net::IpAddr::V4(ip) => {
            let [a, b, c, _] = ip.octets();
            !ip.is_private()
                && !ip.is_loopback()
                && !ip.is_link_local()
                && !ip.is_multicast()
                && !ip.is_broadcast()
                && !ip.is_documentation()
                && !ip.is_unspecified()
                && a != 0
                && a < 240
                && ip.octets() != [168, 63, 129, 16]
                && !(a == 100 && (64..=127).contains(&b))
                && !(a == 192 && b == 0 && c == 0)
                && !(a == 198 && (18..=19).contains(&b))
        }
        std::net::IpAddr::V6(ip) => {
            if let Some(mapped) = ip.to_ipv4_mapped() {
                return public_address(mapped.into());
            }
            let segments = ip.segments();
            segments[0] & 0xe000 == 0x2000
                && segments[0] != 0x2002
                && !(segments[0] == 0x2001 && segments[1] < 0x0200)
                && !(segments[0] == 0x2001 && segments[1] == 0x0db8)
        }
    }
}

pub fn public_url(value: &str) -> Result<url::Url, String> {
    let url =
        url::Url::parse(value).map_err(|_| "Crawler URL must be an absolute HTTP or HTTPS URL")?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("Crawler URLs require HTTP or HTTPS without credentials".to_owned());
    }
    let host = url.host_str().ok_or("Crawler URL requires a hostname")?;
    if host.eq_ignore_ascii_case("localhost") || host.trim_end_matches('.').ends_with(".localhost")
    {
        return Err("Crawler cannot access local hosts".to_owned());
    }
    if let Ok(address) = host.trim_matches(['[', ']']).parse() {
        if !public_address(address) {
            return Err("Crawler cannot access private or special-purpose addresses".to_owned());
        }
    }
    Ok(url)
}

async fn fetch_public_html(value: &str) -> Result<String, String> {
    let url = public_url(value)?;
    let host = url
        .host_str()
        .ok_or("Missing hostname")?
        .trim_matches(['[', ']']);
    let port = url.port_or_known_default().ok_or("Missing port")?;
    let addresses: Vec<_> = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::net::lookup_host((host, port)),
    )
    .await
    .map_err(|_| "Crawler DNS lookup timed out")?
    .map_err(|error| format!("DNS lookup failed: {error}"))?
    .collect();
    if addresses.is_empty()
        || addresses
            .iter()
            .any(|address| !public_address(address.ip()))
    {
        return Err("Crawler DNS result contains a private or special-purpose address".to_owned());
    }
    let client = reqwest::Client::builder()
        .no_proxy()
        .resolve_to_addrs(host, &addresses)
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(concat!("AIRust/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| format!("Client error: {error}"))?;
    let mut response = client
        .get(url.clone())
        .send()
        .await
        .map_err(|error| format!("Fetch error: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Crawler requires a successful response; redirects are disabled ({})",
            response.status()
        ));
    }
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if !["text/html", "text/plain", "application/xhtml+xml"]
        .iter()
        .any(|allowed| content_type.starts_with(allowed))
    {
        return Err("Crawler only accepts HTML and plain text".to_owned());
    }
    const LIMIT: usize = 2 * 1024 * 1024;
    if response
        .content_length()
        .is_some_and(|length| length > LIMIT as u64)
    {
        return Err("Crawler response exceeds 2 MiB".to_owned());
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| format!("Read error: {error}"))?
    {
        if body.len().saturating_add(chunk.len()) > LIMIT {
            return Err("Crawler response exceeds 2 MiB".to_owned());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(String::from_utf8_lossy(&body).into_owned())
}
