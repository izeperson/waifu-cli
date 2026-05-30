use reqwest::blocking::Client;
use serde::Deserialize;

pub const API: &str = "https://nekos.best/api/v2";

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
    artist_href: Option<String>,
    artist_name: Option<String>,
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NekosResp {
    results: Vec<NekosImage>,
}

#[derive(Debug, Deserialize)]
pub struct ImageResp {
    pub url: String,
    pub artist_name: Option<String>,
    pub artist_href: Option<String>,
    pub source_url: Option<String>,
}

pub fn build_client() -> Result<Client, String> {
    Client::builder()
        .user_agent("waifu-cli/0.1.6")
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))
}

pub fn fetch_endpoints(_client: &Client) -> Result<Endpoints, String> {
    Ok(Endpoints {
        sfw: CATEGORIES.iter().map(|s| s.to_string()).collect(),
    })
}

pub fn fetch_image(client: &Client, category: &str) -> Result<ImageResp, String> {
    let url = format!("{}/{}", API, category);
    let response = client.get(&url).send().map_err(|e| format!("Request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("API returned error status: {}", response.status()));
    }

    let resp: NekosResp = response
        .error_for_status()
        .map_err(|e| format!("API returned an error: {}", e))?
        .json()
        .map_err(|e| format!("Failed to decode image response: {}", e))?;

    resp.results
        .into_iter()
        .next()
        .map(|img| ImageResp { 
            url: img.url,
            artist_name: img.artist_name,
            artist_href: img.artist_href,
            source_url: img.source_url,
        })
        .ok_or_else(|| "No images returned".to_string())
}
