use sha2::{Digest, Sha256};
use std::sync::Arc;

use super::models::BotConfig;
use crate::web::db::Database;

pub struct ProcessorBot;

#[derive(Debug, Clone)]
pub struct QAPair {
    pub input: String,
    pub output: String,
}

impl ProcessorBot {
    /// Process approved raw_data into Q&A pairs and store in approved_data
    pub fn process(db: &Arc<Database>, config: &BotConfig) -> Result<(i64, i64), String> {
        let pending = db.get_pending_raw_data()?;
        let mut items_found: i64 = 0;
        let mut items_added: i64 = 0;

        for raw in &pending {
            // Binary content check: skip garbage data
            if Self::is_binary_content(&raw.content) {
                eprintln!("Skipping binary content for raw_data id={}", raw.id);
                db.approve_raw_data(raw.id)?;
                continue;
            }

            // Auto-approve for processing
            db.approve_raw_data(raw.id)?;

            let cleaned = Self::clean_content(&raw.content);
            if cleaned.is_empty() {
                continue;
            }

            let mut pairs = match config.strategy.as_str() {
                "paragraph" => Self::strategy_paragraph(&cleaned, config.min_length),
                "sentence" => Self::strategy_sentence(&cleaned, config.min_length),
                _ => Self::strategy_heading_content(&cleaned, config.min_length),
            };

            // Generate page summary Q&A for every page (if title exists)
            if let Some(ref title) = raw.title {
                let summary_pairs = Self::generate_page_summary_qa(title, &cleaned);
                pairs.extend(summary_pairs);
            }

            items_found += pairs.len() as i64;

            for pair in pairs {
                // Deduplicate
                let mut hasher = Sha256::new();
                hasher.update(format!("{}:{}", pair.input, pair.output).as_bytes());
                let hash = format!("{:x}", hasher.finalize());

                // Check if we already have this exact Q&A
                let existing = db.insert_raw_data(
                    raw.bot_run_id,
                    raw.url.as_deref(),
                    Some(&pair.input),
                    &pair.output,
                    &hash,
                )?;

                if existing > 0 {
                    db.insert_approved_data(
                        Some(raw.id),
                        &pair.input,
                        &pair.output,
                        1.0,
                        raw.url.as_deref(),
                    )?;
                    items_added += 1;
                }
            }
        }

        Ok((items_found, items_added))
    }

    /// Check if content is binary garbage (PDF loaded as text, etc.)
    fn is_binary_content(text: &str) -> bool {
        let total = text.chars().count();
        if total == 0 {
            return true;
        }
        // Count non-printable characters (excluding normal whitespace)
        let non_printable = text
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
        // Count Unicode replacement chars
        let replacements = text.chars().filter(|&c| c == '\u{FFFD}').count();
        non_printable as f64 / total as f64 > 0.10 || replacements as f64 / total as f64 > 0.10
    }

    /// Clean content: normalize whitespace, remove junk
    fn clean_content(text: &str) -> String {
        if Self::is_binary_content(text) {
            return String::new();
        }
        // Normalize whitespace: collapse multiple spaces, trim lines
        let lines: Vec<String> = text
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
            .filter(|line| !line.trim().is_empty())
            .collect();

        lines.join("\n")
    }

    /// Generate a proper question from a heading
    fn heading_to_question(heading: &str) -> String {
        let h = heading.trim();

        // Already a question
        if h.ends_with('?') {
            return h.to_string();
        }

        // Common heading patterns and their question forms
        let lower = h.to_lowercase();

        if lower.starts_with("über ") || lower == "über uns" {
            return format!(
                "Was kann man über {} erfahren?",
                h[5..].trim_end_matches(':')
            );
        }
        if lower.starts_with("about ") || lower == "about us" {
            return format!("What is {}?", h[6..].trim_end_matches(':'));
        }
        if lower.starts_with("how to") || lower.starts_with("wie ") {
            return format!("{}?", h.trim_end_matches(':'));
        }
        if lower.starts_with("warum") || lower.starts_with("why") {
            return format!("{}?", h.trim_end_matches(':'));
        }
        if lower.starts_with("kontakt") || lower.starts_with("contact") {
            return format!("Wie erreicht man {}?", h.trim_end_matches(':'));
        }
        if lower.starts_with("faq") || lower.starts_with("häufige fragen") {
            return format!(
                "Welche häufigen Fragen gibt es zu {}?",
                h.trim_end_matches(':')
            );
        }

        // Default: "Was ist {heading}?"
        format!("Was ist {}?", h.trim_end_matches(':'))
    }

    /// Truncate text at a sentence boundary, max `max_len` characters
    fn truncate_at_sentence(text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            return text.to_string();
        }

        let truncated = &text[..max_len];
        // Find last sentence end within the limit
        let last_period = truncated.rfind(". ");
        let last_excl = truncated.rfind("! ");
        let last_quest = truncated.rfind("? ");

        let best = [last_period, last_excl, last_quest]
            .iter()
            .filter_map(|&pos| pos)
            .max();

        match best {
            Some(pos) => text[..=pos].trim().to_string(),
            None => {
                // No sentence boundary found, cut at last space
                match truncated.rfind(' ') {
                    Some(pos) => format!("{}...", text[..pos].trim()),
                    None => format!("{}...", truncated),
                }
            }
        }
    }

    /// Smart sentence splitting that doesn't break on abbreviations
    fn split_sentences(text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current = String::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut i = 0;

        // Common abbreviations that shouldn't split
        let abbrevs = [
            "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Nr.", "St.", "Str.", "z.B.", "d.h.", "u.a.",
            "o.ä.", "etc.", "bzw.", "inkl.", "zzgl.", "ca.", "max.", "min.", "tel.", "Tel.",
            "Fax.", "Abs.", "e.V.", "i.d.R.", "u.U.", "z.T.",
        ];

        while i < len {
            current.push(chars[i]);

            if (chars[i] == '.' || chars[i] == '!' || chars[i] == '?')
                && i + 1 < len
                && chars[i + 1].is_whitespace()
            {
                // Check if this period is part of an abbreviation
                let current_trimmed = current.trim();
                let is_abbrev = abbrevs.iter().any(|a| current_trimmed.ends_with(a));

                if !is_abbrev {
                    let sentence = current.trim().to_string();
                    if !sentence.is_empty() {
                        sentences.push(sentence);
                    }
                    current = String::new();
                }
            }
            i += 1;
        }

        // Remaining text
        let remainder = current.trim().to_string();
        if !remainder.is_empty() {
            sentences.push(remainder);
        }

        sentences
    }

    /// Generate question from a sentence (reformulate first sentence into a question)
    fn sentence_to_question(sentence: &str) -> String {
        let s = sentence
            .trim()
            .trim_end_matches('.')
            .trim_end_matches('!')
            .trim();
        if s.ends_with('?') {
            return s.to_string();
        }
        if s.is_empty() {
            return String::new();
        }
        // "Was ist/bedeutet {first_sentence}?"
        let lower = s.to_lowercase();
        if lower.starts_with("wir ")
            || lower.starts_with("unser")
            || lower.starts_with("die ")
            || lower.starts_with("das ")
            || lower.starts_with("der ")
        {
            format!("Was bedeutet: {}?", s)
        } else {
            format!("Was ist {}?", s)
        }
    }

    /// Strategy: Use headings as questions, following text as answers
    fn strategy_heading_content(content: &str, min_length: usize) -> Vec<QAPair> {
        let mut pairs = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Detect heading-like lines (short, no period at end, capitalized or all caps)
            let is_heading = !line.is_empty()
                && line.len() < 100
                && !line.ends_with('.')
                && (line
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false));

            if is_heading && i + 1 < lines.len() {
                // Collect following paragraph text
                let mut body = Vec::new();
                let mut j = i + 1;
                while j < lines.len() {
                    let next = lines[j].trim();
                    if next.is_empty() && !body.is_empty() {
                        break;
                    }
                    if !next.is_empty() {
                        body.push(next);
                    }
                    j += 1;
                }

                let full_answer = body.join(" ");
                let answer = Self::truncate_at_sentence(&full_answer, 500);

                if line.len() >= min_length && answer.len() >= min_length {
                    // Generate a proper question from the heading
                    let question = Self::heading_to_question(line);
                    pairs.push(QAPair {
                        input: question,
                        output: answer,
                    });
                }
                i = j;
            } else {
                i += 1;
            }
        }

        pairs
    }

    /// Strategy: Each paragraph becomes a Q&A pair
    fn strategy_paragraph(content: &str, min_length: usize) -> Vec<QAPair> {
        let mut pairs = Vec::new();
        let paragraphs: Vec<&str> = content.split("\n\n").collect();

        for para in paragraphs {
            let text = para.trim();
            if text.len() < min_length {
                continue;
            }

            let sentences = Self::split_sentences(text);
            if sentences.is_empty() {
                continue;
            }

            let first_sentence = sentences[0].trim();
            if first_sentence.len() >= min_length && text.len() > first_sentence.len() {
                let question = Self::sentence_to_question(first_sentence);
                let answer = Self::truncate_at_sentence(text, 500);
                if !question.is_empty() {
                    pairs.push(QAPair {
                        input: question,
                        output: answer,
                    });
                }
            }
        }

        pairs
    }

    /// Strategy: Each sentence pair becomes Q&A
    fn strategy_sentence(content: &str, min_length: usize) -> Vec<QAPair> {
        let mut pairs = Vec::new();
        let sentences = Self::split_sentences(content);
        let filtered: Vec<&String> = sentences.iter().filter(|s| s.len() >= min_length).collect();

        for chunk in filtered.chunks(2) {
            if chunk.len() == 2 {
                let question = Self::sentence_to_question(chunk[0]);
                let answer = Self::truncate_at_sentence(chunk[1], 500);
                if !question.is_empty() {
                    pairs.push(QAPair {
                        input: question,
                        output: answer,
                    });
                }
            }
        }

        pairs
    }

    /// Generate page summary Q&A from page title and content
    /// Makes pages findable by their title (e.g. "LE Signs" → finds the page)
    fn generate_page_summary_qa(title: &str, content: &str) -> Vec<QAPair> {
        let mut pairs = Vec::new();
        let title = title.trim();

        if title.is_empty() || title.len() < 3 || content.trim().is_empty() {
            return pairs;
        }

        // Get first paragraph as answer (max 500 chars, sentence boundary)
        let first_para = content.split("\n\n").next().unwrap_or(content).trim();

        let answer = if first_para.len() >= 20 {
            Self::truncate_at_sentence(first_para, 500)
        } else {
            // First paragraph too short, use more content
            Self::truncate_at_sentence(content, 500)
        };

        if answer.len() < 20 {
            return pairs;
        }

        // "Was ist {title}?"
        pairs.push(QAPair {
            input: format!("Was ist {}?", title),
            output: answer.clone(),
        });

        // Also add title directly as question (for direct search matches)
        if !title.ends_with('?') {
            pairs.push(QAPair {
                input: format!("{}?", title),
                output: answer,
            });
        }

        pairs
    }
}
