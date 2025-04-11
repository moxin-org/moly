use reqwest::header::{HeaderValue, USER_AGENT};
use scraper::Selector;

/// Perform a GET request and return the raw bytes.
pub(crate) async fn fetch_bytes(url: &str) -> Result<Vec<u8>, ()> {
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        // Trick the server into thinking we're a browser
        .header(USER_AGENT, HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"
        ))
        .send()
        .await
        .map_err(|_| ())?;

    if !response.status().is_success() {
        return Err(());
    }

    let text = response.bytes().await.map_err(|_| ())?;
    Ok(text.to_vec())
}

/// Perform a GET request and parse the response as text.
pub(crate) async fn fetch_text(url: &str) -> Result<String, ()> {
    let bytes = fetch_bytes(url).await.map_err(|_| ())?;
    let text = String::from_utf8(bytes).map_err(|_| ())?;
    Ok(text)
}

/// Type representing an HTML document.
///
/// This just holds a string that is parsed each time by extraction functions, but
/// the extra type wrapper prevents feeding any string into them.
///
/// The html type from the `scraper` crate would be better but it has some
/// non-Send data, so holding a parsed tree for efficiency is not trivial inside
/// multi-threaded code, at least with that representation.
pub(crate) struct Html(String);

impl Html {
    /// Convert to a parsed HTML document.
    pub(crate) fn to_scraper(&self) -> scraper::Html {
        scraper::Html::parse_document(&self.0)
    }
}

/// Perform a GET request and try to parse it as an HTML document.
pub(crate) async fn fetch_html(url: &str) -> Result<Html, ()> {
    let text = fetch_text(url).await?;
    Ok(Html(text))
}

/// Extract the title from a standard HTML document.
pub(crate) fn extract_title(document: &Html) -> Option<String> {
    let document = document.to_scraper();
    let title_selector = Selector::parse("title").unwrap();
    document
        .select(&title_selector)
        .next()
        .and_then(|element| element.text().next())
        .map(|text| text.trim().to_string())
}
