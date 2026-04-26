use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct AurComment {
    pub author: String,
    pub date: String,
    pub body: String,
}

pub async fn fetch_comments(client: &Client, pkg_name: &str) -> Result<Vec<AurComment>> {
    let url = format!("https://aur.archlinux.org/packages/{}", pkg_name);
    let html = client
        .get(&url)
        .header("User-Agent", "aurdash/0.1 (AUR TUI helper)")
        .send()
        .await?
        .text()
        .await?;

    Ok(parse_comments(&html))
}

fn parse_comments(html: &str) -> Vec<AurComment> {
    let document = Html::parse_document(html);
    let mut comments = Vec::new();

    // AUR comment structure:
    // <div class="comments package-comments">
    //   <h4 class="comment-header" id="comment-XXXXXX">
    //     <a href="/account/user">user</a> commented on YYYY-MM-DD ...
    //   </h4>
    //   <div class="article-content"><p>...</p></div>
    // </div>

    let header_sel = Selector::parse("h4.comment-header").unwrap_or_else(|_| {
        Selector::parse("h4").unwrap()
    });
    let body_sel = Selector::parse("div.article-content").unwrap_or_else(|_| {
        Selector::parse("div.comment-content").unwrap_or_else(|_| {
            Selector::parse("div.content").unwrap()
        })
    });

    let headers: Vec<_> = document.select(&header_sel).collect();
    let bodies: Vec<_> = document.select(&body_sel).collect();

    for (i, header) in headers.iter().enumerate() {
        let header_text = header.text().collect::<Vec<_>>().join(" ");
        let header_text = header_text.split_whitespace().collect::<Vec<_>>().join(" ");

        // Extract author (first word-like chunk before "commented")
        let author = header_text
            .split("commented")
            .next()
            .unwrap_or("Unknown")
            .trim()
            .to_string();

        // Extract date
        let date = if let Some(pos) = header_text.find("on ") {
            header_text[pos + 3..]
                .split_whitespace()
                .take(1)
                .collect::<Vec<_>>()
                .join("")
        } else {
            String::from("Unknown date")
        };

        let body = if let Some(b) = bodies.get(i) {
            b.text()
                .collect::<Vec<_>>()
                .join(" ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            String::new()
        };

        if !author.is_empty() && author != "Unknown" {
            comments.push(AurComment { author, date, body });
        }
    }

    // Return most recent first (AUR shows newest at bottom, we reverse)
    comments.reverse();
    // Cap at 20 comments
    comments.truncate(20);
    comments
}
