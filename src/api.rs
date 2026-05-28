// waifu-cli, developed by izeperson + techdude3000
use reqwest::blocking::Client;
use serde::Deserialize;

const API: &str = "https://nekos.best/api/v2";

pub const CATEGORIES: &[&str] = &[
    "neko", "husbando", "kitsune", "waifu",
    "blush", "clap", "confused", "cry", "dance",
    "feed", "happy", "highfive", "hug", "kiss",
    "laugh", "lurk", "pat", "peck", "poke",
    "punch", "shoot", "shrug", "sip", "sleep",
    "smile", "smug", "stare", "think", "tickle",
    "wave", "wink", "yeet",
];

#[derive(Debug, Deserialize)]
pub struct Endpoints {
    pub sfw: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct NekosImage {
    url: String,
}

#[derive(Debug, Deserialize)]
struct NekosResp {
    results: Vec<NekosImage>,
}

#[derive(Debug, Deserialize)]
pub struct ImageResp {
    pub url: String,
}

pub fn build_client() -> Result<Client, String> {
    Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:124.0) Gecko/20100101 Firefox/124.0")
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))
}

pub fn fetch_endpoints(_client: &Client) -> Result<Endpoints, String> {
    Ok(Endpoints {
        sfw: CATEGORIES.iter().map(|s| s.to_string()).collect(),
    })
}

pub fn fetch_image(client: &Client, category: &str) -> Result<ImageResp, String> {
    let resp: NekosResp = client
        .get(format!("{}/{}", API, category))
        .send()
        .map_err(|e| format!("Failed to fetch image: {}", e))?
        .json()
        .map_err(|e| format!("Failed to decode image response: {}", e))?;

    resp.results
        .into_iter()
        .next()
        .map(|img| ImageResp { url: img.url })
        .ok_or_else(|| "No images returned".to_string())
}
