use scraper::{Html, Selector};

#[allow(dead_code)]
pub struct Extracted {
    pub title: String,
    pub body_html: String,
}

#[allow(dead_code)]
pub fn extract_article(html: &str) -> Extracted {
    let document = Html::parse_document(html);
    // Try to extract main content
    let body_sel = Selector::parse(
        "#mw-content-text .mw-parser-output, #bodyContent, main, article, .content",
    )
    .unwrap_or_else(|_| Selector::parse("body").unwrap());
    if let Some(el) = document.select(&body_sel).next() {
        let body_html = el.html();
        Extracted {
            title: String::new(),
            body_html,
        }
    } else {
        Extracted {
            title: String::new(),
            body_html: html.to_string(),
        }
    }
}
