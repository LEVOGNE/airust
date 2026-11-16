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
    /// Crawl a URL and extract text content
    pub async fn run(
        config: &BotConfig,
        db: &Arc<Database>,
        run_id: i64,
    ) -> Result<(i64, i64), String> {
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

                            // Extract links for next depth
                            if depth < config.max_depth {
                                if let Ok(links) = Self::extract_links(&url, &page.content).await {
                                    for link in links {
                                        if !visited.contains(&link) {
                                            queue.push((link, depth + 1));
                                        }
                                    }
                                }
                            }

                            // Rate limiting
                            tokio::time::sleep(std::time::Duration::from_millis(
                                config.rate_limit_ms,
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
                // Default: single page
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
        let client = reqwest::Client::builder()
            .user_agent(concat!("AIRust/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Client error: {}", e))?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Fetch error: {}", e))?;

        // Skip non-HTML content (PDFs, images, etc.)
        if let Some(ct) = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
        {
            if !ct.contains("text/html")
                && !ct.contains("text/plain")
                && !ct.contains("application/xhtml")
            {
                return Err(format!("Skipping non-HTML content: {}", ct));
            }
        }

        let html = response
            .text()
            .await
            .map_err(|e| format!("Read error: {}", e))?;

        let document = scraper::Html::parse_document(&html);

        // Extract title
        let title_selector = scraper::Selector::parse("title").unwrap();
        let title = document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        // Extract main text content (skip scripts, styles, nav)
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

        // SHA256 hash for deduplication
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

    /// Clean extracted text: remove binary garbage, web junk, normalize whitespace
    fn clean_text(raw: &str) -> String {
        // Binary check: if > 10% replacement chars (U+FFFD) → discard entirely
        let total_chars = raw.chars().count();
        if total_chars > 0 {
            let replacement_count = raw.chars().filter(|&c| c == '\u{FFFD}').count();
            if replacement_count as f64 / total_chars as f64 > 0.10 {
                return String::new();
            }
            // Also check for high ratio of non-printable chars (binary data)
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
                // Collapse multiple spaces into one
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
                // Remove lines < 3 chars (menu remnants like "X", "→", "|")
                if line.len() < 3 {
                    return false;
                }
                // Remove junk patterns
                for pattern in &junk_patterns {
                    if line.contains(pattern) {
                        return false;
                    }
                }
                true
            })
            .collect();

        // Collapse multiple empty lines into max one
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
        // Re-fetch to get links from HTML (we already have the content but need HTML for links)
        let client = reqwest::Client::builder()
            .user_agent(concat!("AIRust/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Client error: {}", e))?;

        let response = client
            .get(base_url)
            .send()
            .await
            .map_err(|e| format!("Fetch error: {}", e))?;

        let html = response
            .text()
            .await
            .map_err(|e| format!("Read error: {}", e))?;

        let document = scraper::Html::parse_document(&html);
        let a_selector = scraper::Selector::parse("a[href]").unwrap();
        let base = url::Url::parse(base_url).map_err(|e| format!("URL parse error: {}", e))?;

        let mut links = Vec::new();
        for el in document.select(&a_selector) {
            if let Some(href) = el.value().attr("href") {
                if let Ok(resolved) = base.join(href) {
                    // Only follow same-host links
                    if resolved.host() == base.host() {
                        links.push(resolved.to_string());
                    }
                }
            }
        }

        Ok(links)
    }
}
