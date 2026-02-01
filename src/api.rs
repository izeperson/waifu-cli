// waifu-cli, developed by izeperson + techdude3000
use reqwest::blocking::Client;
use serde::Deserialize;

const API: &str = "https://api.waifu.pics";

#[derive(Debug, Deserialize)]
pub struct Endpoints {
    pub sfw: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImageResp {
    pub url: String,
}

pub fn fetch_endpoints(client: &Client) -> Result<Endpoints, String> {
    client
    .get(format!("{}/endpoints", API))
    .send()
    .map_err(|e| format!("Failed to fetch endpoints: {}", e))
    .and_then(|response| response.json().map_err(|e| format!("Failed to decode endpoints: {}", e)))
}

pub fn fetch_image(client: &Client, category: &str) -> Result<ImageResp, String> {
    client
    .get(format!("{}/sfw/{}", API, category))
    .send()
    .and_then(|resp| resp.json())
    .map_err(|e| format!("Failed to fetch image: {}", e))
}
