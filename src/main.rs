// waifu-cli, developed by izeperson + techdude3000
use reqwest::blocking::Client;
use serde::Deserialize;
use std::io::{self, Write};
use crossterm::terminal::size;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};
use std::time::Instant;
use zeroize::Zeroize;
use std::process::{Command, Stdio};

const API: &str = "https://api.waifu.pics";

#[derive(Debug, Deserialize)]
struct Endpoints {
    nsfw: Vec<String>, // there will be NO nsfw stuff in this program
}

#[derive(Debug, Deserialize)]
struct ImageResp {
    url: String,
}

fn main() {
    let client = Client::new();
    let ep_result: Result<Endpoints, String> = client
        .get(format!("{}/endpoints", API))
        .send()
        .map_err(|e| format!("Failed to fetch endpoints: {}", e))
        .and_then(|response| response.json().map_err(|e| format!("Failed to decode endpoints: {}", e)));

    let ep = match ep_result {
        Ok(endpoints) => endpoints,
        Err(e) => {
            eprintln!("API Error: {}", e);
            std::process::exit(1);
        }
    };

    let mut categories = ep.nsfw;
    categories.sort();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: waifu <command>");
        println!("Run 'waifu --help' for a list of commands.");
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "-l" | "--list" => {
            println!("Available categories:");
            for category in categories {
                println!("  {}", category);
            }
        }
        "-s" | "--stats" => {
            println!("Running performance statistics...");
            let total_start = Instant::now();

            let meta_start = Instant::now();
            let test_category = "waifu";
            let img_resp = client
                .get(format!("{}/nsfw/{}", API, test_category))
                .send()
                .and_then(|resp| resp.json::<ImageResp>());

            if let Ok(img) = img_resp {
                let meta_fetch_time = meta_start.elapsed();
                println!("[1/2] Fetched image metadata for '{}' in: {:?}", test_category, meta_fetch_time);

                let download_start = Instant::now();
                let download_resp = client.get(&img.url).send().and_then(|resp| resp.bytes());

                if let Ok(bytes) = download_resp {
                    let download_time = download_start.elapsed();
                    println!("[2/2] Downloaded image ({:.2} KB) in: {:?}", bytes.len() as f64 / 1024.0, download_time);
                    println!("\nTotal time elapsed: {:?}", total_start.elapsed());
                } else {
                    eprintln!("Error: Failed to download image bytes from {}", img.url);
                }
            } else {
                eprintln!("Error: Failed to fetch image metadata for category '{}'.", test_category);
            }
        }
        "-t" | "--test" => {
            println!("Testing API connectivity...");
            if categories.is_empty() {
                eprintln!("Error: No categories found. API may be down or returned an empty list.");
                std::process::exit(1);
            }
            let test_category = &categories[0];
            match client.get(format!("{}/nsfw/{}", API, test_category)).send() {
                Ok(response) if response.status().is_success() => {
                    println!("Test successful! Fetched an image from '{}'.", test_category);
                }
                _ => {
                    eprintln!("Error: Failed to fetch an image from '{}'. API might be down or returned an error.", test_category);
                    std::process::exit(1);
                }
            }
        }
        "-c" | "--category" => {
            if args.len() < 3 {
                eprintln!("Error: The '-c' flag requires a category name.");
                eprintln!("Usage: waifu -c <category_name>");
                std::process::exit(1);
            }
            let category_name = &args[2];
            if categories.contains(category_name) {
                fetch_and_display_image(&client, category_name);
            } else {
                eprintln!("Error: Invalid category '{}'.", category_name);
                eprintln!("Run 'waifu -l' to see available categories.");
                std::process::exit(1);
            }
        }
        "-h" | "--help" => {
            println!("Usage: waifu <command>");
            println!("\nA simple CLI to fetch images from waifu.pics.\n");
            println!("Commands:");
            println!("  -c, --category <name>   Fetch an image from a specific category");
            println!("  -l, --list              List all available categories");
            println!("  -s, --stats             Show request performance statistics");
            println!("  -t, --test              Test API connectivity");
            println!("  -h, --help              Show this help message");
        }
        _ => {
            eprintln!("Error: Unknown command '{}'.", command);
            eprintln!("Run 'waifu --help' for a list of available commands.");
            std::process::exit(1);
        }
    }
}



fn fetch_and_display_image(client: &Client, category: &str) {
    loop { // image fetch/display section
        let img_result: Result<ImageResp, _> = client
            .get(format!("{}/nsfw/{}", API, category))
            .send()
            .and_then(|resp| resp.json());

        let img = match img_result {
            Ok(i) => i,
            Err(_) => {
                eprintln!("Error: Failed to fetch image metadata for category '{}'.", category);
                break;
            }
        };

        let mut bytes = match client.get(&img.url).send().and_then(|resp| resp.bytes()) {
            Ok(b) => b.to_vec(),
            Err(_) => {
                eprintln!("Error: Failed to download image from {}", img.url);
                break;
            }
        };

        if let Ok((cols, _)) = size() { // spacing
            let title = "--waifu-cli--";
            let title_len = title.len() as u16;
            let padding = (cols.saturating_sub(title_len)) / 2;
            let spaces = " ".repeat(padding as usize);
            println!("{}{}", spaces, title);
            println!();
        } else {
            println!("--waifu-cli--");
            println!();
        }

        if let Ok(mut child) = Command::new("kitty").args(["+kitten", "icat"]).stdin(Stdio::piped()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(&bytes);
            }
            let _ = child.wait();

            print!("[s]ave | [n]ext | [enter/q] quit: ");
            io::stdout().flush().unwrap();

            terminal::enable_raw_mode().unwrap();
            let mut continue_fetching = false;
            'input: loop { // basic while loop, you would find this in python as while true:. 
                if let Ok(Event::Key(key_event)) = event::read() {
                    match key_event.code {
                        KeyCode::Char('s') => {
                            terminal::disable_raw_mode().unwrap();
                            println!();
                            let filename = img.url.split('/').last().unwrap_or("waifu.png");
                            if std::fs::write(filename, &bytes).is_ok() {
                                println!("Image saved as {}", filename);
                            } else {
                                eprintln!("Error: Failed to save image.");
                            }
                            print!("[s]ave | [n]ext | [enter/q] quit: ");
                            io::stdout().flush().unwrap();
                            terminal::enable_raw_mode().unwrap();
                            continue 'input;
                        }
                        KeyCode::Char('n') => {
                            continue_fetching = true;
                            break 'input;
                        }
                        KeyCode::Enter | KeyCode::Char('q') | _ => {
                            break 'input;
                        }
                    }
                }
            }
            terminal::disable_raw_mode().unwrap();
            println!();

            if !continue_fetching {
                break;
            }
        } else {
            println!("Failed to display image. Is kitty terminal installed?");
            println!("Image URL: {}", img.url);
            break;
        }

        bytes.zeroize();
    }
}
