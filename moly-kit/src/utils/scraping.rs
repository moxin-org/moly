use reqwest::{
    header::{HeaderValue, USER_AGENT},
    Client,
};
use scraper::{Html, Selector};

/// Perform a GET request and parse the response as text.
pub(crate) async fn fetch_text(url: &str) -> Result<String, ()> {
    let client = Client::new();

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

    let text = response.text().await.map_err(|_| ())?;
    Ok(text)
}

/// Perform a GET request and try to parse it as an HTML document.
pub(crate) async fn fetch_html(url: &str) -> Result<Html, ()> {
    let text = fetch_text(url).await?;
    let document = scraper::Html::parse_document(&text);
    Ok(document)
}

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

/// Extract the title from a standard HTML document.
pub(crate) fn extract_title(document: &Html) -> Option<String> {
    let title_selector = Selector::parse("title").unwrap();
    document
        .select(&title_selector)
        .next()
        .and_then(|element| element.text().next())
        .map(|text| text.trim().to_string())
}

/// Extract the favicon URL from a standard HTML document.
pub(crate) fn extract_favicon(document: &Html) -> Option<String> {
    let favicon_selector = Selector::parse("link[rel=\"icon\"]").unwrap();
    document
        .select(&favicon_selector)
        .next()
        .and_then(|element| element.value().attr("href"))
        .map(|href| href.to_string())
}
