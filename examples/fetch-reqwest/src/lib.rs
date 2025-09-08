// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#[allow(warnings)]
mod bindings;

use bindings::Guest;
use serde_json::Value;

struct Component;

impl Guest for Component {
    fn fetch(url: String) -> Result<String, String> {
        // Create a lightweight Tokio runtime to drive the async reqwest call.
        // We use a current-thread runtime to avoid spawning threads in WASI.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;

        rt.block_on(async move { fetch_with_reqwest(url).await })
    }
}

async fn fetch_with_reqwest(url: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!(
            "Request failed with status code: {}",
            response.status()
        ));
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let body = response.text().await.map_err(|e| e.to_string())?;

    if content_type.contains("application/json") {
        let json: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        Ok(json_to_markdown(&json))
    } else if content_type.contains("text/html") {
        Ok(html_to_markdown(&body))
    } else {
        Ok(body)
    }
}

fn html_to_markdown(html: &str) -> String {
    let mut markdown = String::new();
    let fragment = scraper::Html::parse_fragment(html);
    let text_selector = scraper::Selector::parse("h1, h2, h3, h4, h5, h6, p, a, div").unwrap();

    for element in fragment.select(&text_selector) {
        let tag_name = element.value().name();
        let text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
        
        if text.is_empty() {
            continue;
        }

        match tag_name {
            "h1" => markdown.push_str(&format!("# {}\n\n", text)),
            "h2" => markdown.push_str(&format!("## {}\n\n", text)),
            "h3" => markdown.push_str(&format!("### {}\n\n", text)),
            "h4" => markdown.push_str(&format!("#### {}\n\n", text)),
            "h5" => markdown.push_str(&format!("##### {}\n\n", text)),
            "h6" => markdown.push_str(&format!("###### {}\n\n", text)),
            "p" => markdown.push_str(&format!("{}\n\n", text)),
            "a" => {
                if let Some(href) = element.value().attr("href") {
                    markdown.push_str(&format!("[{}]({})\n\n", text, href));
                } else {
                    markdown.push_str(&format!("{}\n\n", text));
                }
            },
            _ => markdown.push_str(&format!("{}\n\n", text)),
        }
    }

    markdown.trim().to_string()
}

fn json_to_markdown(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut markdown = String::new();
            for (key, val) in map {
                markdown.push_str(&format!("### {}\n\n{}\n\n", key, json_to_markdown(val)));
            }
            markdown
        }
        Value::Array(arr) => {
            let mut markdown = String::new();
            for (i, val) in arr.iter().enumerate() {
                markdown.push_str(&format!("1. {}\n", json_to_markdown(val)));
                if i < arr.len() - 1 {
                    markdown.push('\n');
                }
            }
            markdown
        }
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
    }
}

bindings::export!(Component with_types_in bindings);
