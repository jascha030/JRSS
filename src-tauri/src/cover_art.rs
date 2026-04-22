use color_thief::{get_palette, ColorFormat};
use image::imageops::FilterType;
use reqwest::blocking::Client;
use std::collections::HashSet;
use std::time::Duration;
use url::Url;

fn normalize_image_url(image_url: &str) -> Result<String, String> {
    let parsed_url =
        Url::parse(image_url).map_err(|_| "Image URL is not a valid URL.".to_string())?;

    match parsed_url.scheme() {
        "http" | "https" => Ok(parsed_url.to_string()),
        _ => Err("Image URL must use http or https.".to_string())
    }
}

fn build_http_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("JRSS/0.0.1 Cover")
        .build()
        .map_err(|error| format!("Failed to create HTTP client: {error}"))
}

fn color_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

pub fn extract_cover_palette(image_url: &str) -> Result<Vec<String>, String> {
    let normalized_url = normalize_image_url(image_url)?;
    let client = build_http_client()?;

    let response = client
        .get(&normalized_url)
        .send()
        .map_err(|error| format!("Failed to fetch cover image: {error}"))?;

    let status = response.status();

    if !status.is_success() {
        return Err(format!("Cover image request failed with status {status}."));
    }

    let bytes = response
        .bytes()
        .map_err(|error| format!("Failed to read cover image response body: {error}"))?;

    let decoded =
        image::load_from_memory(&bytes).map_err(|error| format!("Failed to decode cover image: {error}"))?;

    let resized = if decoded.width() > 512 || decoded.height() > 512 {
        decoded.resize(512, 512, FilterType::Triangle)
    } else {
        decoded
    };

    let rgba = resized.to_rgba8();

    let palette = get_palette(rgba.as_raw(), ColorFormat::Rgba, 5, 6)
        .map_err(|error| format!("Failed to extract cover palette: {error}"))?;

    let mut seen = HashSet::new();

    let colors = palette
        .into_iter()
        .map(|color| color_to_hex(color.r, color.g, color.b))
        .filter(|hex| seen.insert(hex.clone()))
        .collect::<Vec<_>>();

    if colors.is_empty() {
        return Err("Cover image palette was empty.".to_string());
    }

    Ok(colors)
}
