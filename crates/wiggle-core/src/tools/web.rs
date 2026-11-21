//! Web tools: web_search, web_fetch, and the three browser-agent tools
//! backed by the CDP controller in wiggle-browser.

use std::sync::Arc;

use serde_json::json;

use super::{ToolContext, ToolDef, ToolOutput};
use crate::CoreError;

fn browser_controller(ctx: &ToolContext) -> wiggle_browser::BrowserController {
    let config = ctx.engine.config().browser.clone();
    wiggle_browser::BrowserController::new(wiggle_browser::BrowserConfig {
        executable: config.executable,
        headless: config.headless,
        persist_session: config.persist_session,
    })
}

pub fn definitions() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "web_fetch",
            description: "Fetch a URL and return its content as markdown-ish plain text (HTML tags stripped).",
            parameters: super::obj_schema(json!({
                "url": { "type": "string" },
                "max_chars": { "type": "integer", "description": "Default 20000" }
            })),
            mutating: false,
            run: Arc::new(|_ctx, args| Box::pin(async move {
                let url = args["url"].as_str().ok_or_else(|| CoreError::Tool("missing 'url'".into()))?;
                let max = args["max_chars"].as_u64().unwrap_or(20_000) as usize;
                let response = reqwest::get(url)
                    .await
                    .map_err(|e| CoreError::Tool(format!("fetch {url}: {e}")))?
                    .error_for_status()
                    .map_err(|e| CoreError::Tool(format!("fetch {url}: {e}")))?;
                let body = response.text().await.map_err(|e| CoreError::Tool(e.to_string()))?;
                Ok(ToolOutput::text(strip_html(&body, max)))
            })),
        },
        ToolDef {
            name: "web_search",
            description: "Search the web using the configured search engine (HTML results, parsed to title + URL + snippet).",
            parameters: super::obj_schema(json!({
                "query": { "type": "string" }
            })),
            mutating: false,
            run: Arc::new(|ctx, args| Box::pin(async move {
                let query = args["query"].as_str().ok_or_else(|| CoreError::Tool("missing 'query'".into()))?;
                let engine_url = ctx.engine.config().browser.search_engine.clone();
                let url = format!("{engine_url}{}", urlencode(query));
                let response = reqwest::Client::new()
                    .get(&url)
                    .header("user-agent", "Mozilla/5.0 (compatible; TervixaWiggle/1.0)")
                    .send()
                    .await
                    .map_err(|e| CoreError::Tool(format!("search: {e}")))?
                    .error_for_status()
                    .map_err(|e| CoreError::Tool(format!("search: {e}")))?;
                let body = response.text().await.map_err(|e| CoreError::Tool(e.to_string()))?;
                Ok(ToolOutput::text(extract_search_results(&body, 10)))
            })),
        },
        ToolDef {
            name: "browser_navigate",
            description: "Navigate the built-in browser to a URL and return the page's compact DOM accessibility tree with [ref=eN] element ids.",
            parameters: super::obj_schema(json!({
                "url": { "type": "string" }
            })),
            mutating: false,
            run: Arc::new(|ctx, args| Box::pin(async move {
                let url = args["url"].as_str().ok_or_else(|| CoreError::Tool("missing 'url'".into()))?.to_string();
                let mut browser = browser_controller(&ctx);
                let snapshot = browser.navigate(url).await?;
                Ok(ToolOutput::text(snapshot.to_compact_tree()))
            })),
        },
        ToolDef {
            name: "browser_actuate",
            description: "Actuate a page element: click, type, select or scroll. Address elements by the [ref=eN] ids from browser_navigate/browser_inspect.",
            parameters: super::obj_schema(json!({
                "ref": { "type": "string", "description": "Element ref like e12" },
                "action": { "type": "string", "description": "click | type | select | scroll" },
                "text": { "type": "string", "description": "Text for type, value for select" }
            })),
            mutating: true,
            run: Arc::new(|ctx, args| Box::pin(async move {
                let reference = args["ref"].as_str().ok_or_else(|| CoreError::Tool("missing 'ref'".into()))?.to_string();
                let action = args["action"].as_str().unwrap_or("click").to_string();
                let text = args["text"].as_str().unwrap_or_default().to_string();
                let mut browser = browser_controller(&ctx);
                let snapshot = browser.actuate(reference, action, text).await?;
                Ok(ToolOutput::text(format!("actuated (see tree below)\n{}", snapshot.to_compact_tree())))
            })),
        },
        ToolDef {
            name: "browser_inspect",
            description: "Inspect the current browser page: compact DOM tree, recent console messages and failed network requests.",
            parameters: super::obj_schema(json!({})),
            mutating: false,
            run: Arc::new(|ctx, _args| Box::pin(async move {
                let mut browser = browser_controller(&ctx);
                let report = browser.inspect().await?;
                Ok(ToolOutput::text(report))
            })),
        },
    ]
}

pub fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Crude HTML -> text converter (no external scraper dependency).
pub fn strip_html(html: &str, max_chars: usize) -> String {
    let mut text = String::with_capacity(html.len() / 2);
    let mut in_tag = false;
    let mut in_script = false;
    let lower = html.to_lowercase();
    let mut chars = html.char_indices();
    while let Some((i, c)) = chars.next() {
        if !in_script && lower[i..].starts_with("<script") {
            in_script = true;
        }
        if in_script {
            if lower[i..].starts_with("</script>") {
                in_script = false;
            }
            continue;
        }
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => {
                text.push(c);
            }
            _ => {}
        }
    }
    // Collapse whitespace.
    let mut out = String::with_capacity(text.len());
    let mut last_space = false;
    for c in text.chars() {
        let is_space = c.is_whitespace();
        if is_space && last_space {
            continue;
        }
        last_space = is_space;
        out.push(if is_space { ' ' } else { c });
    }
    let decoded = decode_entities(&out);
    decoded.chars().take(max_chars).collect()
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

/// Parse naive search-result links out of an HTML results page.
pub fn extract_search_results(html: &str, limit: usize) -> String {
    let mut results = Vec::new();
    let mut idx = 0;
    while let Some(pos) = html[idx..].find("<a ") {
        let start = idx + pos;
        let Some(tag_end_rel) = html[start..].find('>') else { break };
        let tag_end = start + tag_end_rel;
        let tag = &html[start..=tag_end];
        let Some(href_pos) = tag.find("href=\"") else {
            idx = tag_end;
            continue;
        };
        let href_start = start + href_pos + 6;
        let Some(href_len) = html[href_start..].find('"') else { break };
        let href = &html[href_start..href_start + href_len];
        if href.starts_with("http") && !href.contains("duckduckgo.com") {
            let text_start = tag_end + 1;
            let text_end = html[text_start..].find("</a>").map(|i| text_start + i).unwrap_or(text_start);
            let title = strip_html(&html[text_start..text_end], 120);
            results.push(format!("{title}\n  {href}"));
            if results.len() >= limit {
                break;
            }
        }
        idx = tag_end + 1;
    }
    if results.is_empty() {
        "(no results parsed — fetch the URL directly with web_fetch)".into()
    } else {
        results.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_stripping() {
        let html = "<html><head><style>x{}</style></head><body><h1>Title</h1><p>Body &amp; more.</p><script>var hidden=1;</script></body></html>";
        let text = strip_html(html, 1000);
        assert!(text.contains("Title"));
        assert!(text.contains("Body & more."));
        assert!(!text.contains("hidden"));
        assert!(!text.contains('<'));
    }

    #[test]
    fn url_encoding() {
        assert_eq!(urlencode("a b&c"), "a%20b%26c");
    }

    #[test]
    fn search_extraction_pulls_links() {
        let html = r#"<a href="https://example.com/1">Example One</a><a href="/relative">skip</a><a href="https://example.com/2">Two</a>"#;
        let out = extract_search_results(html, 10);
        assert!(out.contains("https://example.com/1"));
        assert!(out.contains("Example One"));
        assert!(!out.contains("/relative"));
    }
}
