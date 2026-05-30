use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Instant;
use crate::api::ImageResp;
use crossterm::terminal::size;
use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode},
    execute,
    terminal,
};
use zeroize::Zeroize;
use api::CATEGORIES;

mod api;
use api::{fetch_endpoints, fetch_image, build_client};

const API: &str = "https://nekos.best/api/v2";

#[derive(Debug, Deserialize)]
struct ManyResp {
    results: Vec<ImageResp>,
}

fn show_stats(client: &Client, categories: &[String]) {
    use std::time::Instant;

    const GREEN: &str = "\x1b[32m";
    const RED: &str = "\x1b[31m";
    const RESET: &str = "\x1b[0m";

    println!("Running Program Functional Test...");
    let start = Instant::now();

    if categories.is_empty() {
        println!("{RED}Program Test: Failed{RESET}");
        println!("No categories found from API.");
        return;
    }
    println!("Endpoints: {} categories found", categories.len());

    let category = &categories[0];

    let batch: ManyResp = match client.get(&format!("{}/{}", API, category))
        .send() 
        .and_then(|r| r.json()) {
        Ok(i) => i,
        Err(_e) => {
            println!("{RED}Program Test: Failed{RESET}");
            println!("Failed to fetch image metadata for '{}'", category);
            return;
        }
    };
    let img = &batch.results[0];
    println!("Single Image Fetch: {}Passed{}", GREEN, RESET);
    println!("Image URL: {}", img.url);

    let bytes = match client.get(&img.url).send().and_then(|r| r.bytes()) {
        Ok(b) => b,
        Err(_e) => {
            println!("{RED}Program Test: Failed{RESET}");
            println!("Failed to download image from '{}'", img.url);
            return;
        }
    };
    println!("Image Bytes: {}OK{}", GREEN, RESET);
    println!("Size: {:.2} KB", bytes.len() as f64 / 1024.0);

    let batch_resp: ManyResp = match client.get(&format!("{}/{}?amount=5", API, category))
        .send() 
        .and_then(|r| r.json()) {
        Ok(b) => b,
        Err(_e) => {
            println!("{RED}Program Test: Failed{RESET}");
            println!("Failed to fetch batch URLs.");
            return;
        }
    };

    if batch_resp.results.len() < 2 {
        println!("{RED}Program Test: Failed{RESET}");
        println!("Batch download returned less than 2 images.");
        return;
    }
    println!("Batch Download ({} images): {}Passed{}", batch_resp.results.len(), GREEN, RESET);

    println!("Program Test: {}Passed{}", GREEN, RESET);
    println!("Time Taken: {:.2?}", start.elapsed());
}

fn main() {
    let client = build_client().unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); });

    let ep_result = fetch_endpoints(&client);
    let ep = match ep_result {
        Ok(endpoints) => endpoints,
        Err(e) => {
            eprintln!("API Error: {}", e);
            std::process::exit(1);
        }
    };

    let mut categories = ep.sfw;
    categories.sort();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        let category = CATEGORIES[rand::random::<usize>() % CATEGORIES.len()];
        fetch_and_display_image(&client, category);
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
        "-t" | "--test" => {
            show_stats(&client, &categories);
        }
        "-c" | "--category" => {
            if args.len() < 3 {
                eprintln!("Error: The '-c' flag requires a category name.");
                eprintln!("Usage: waifu -c <category_name>");
                std::process::exit(1);
            }
            let category_name = &args[2];
            if categories.contains(category_name) {
                if args.len() >= 5 && (args[3] == "-n" || args[3] == "--batch") {
                    let amount_str = &args[4];
                    match amount_str.parse::<usize>() {
                        Ok(amount) => batch_download(&client, category_name, amount),
                        Err(_e) => {
                            eprintln!("Error: Invalid amount '{}'.", amount_str);
                            std::process::exit(1);
                        }
                    }
                } else {
                    fetch_and_display_image(&client, category_name);
                }
            } else {
                eprintln!("Error: Invalid category '{}'.", category_name);
                eprintln!("Run 'waifu -l' to see available categories.");
                std::process::exit(1);
            }
        }
        "-h" | "--help" => {
            print_help();
        }
        _ => {
            eprintln!("Error: Unknown command '{}'.", command);
            eprintln!("Run 'waifu --help' for a list of available commands.");
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("Usage: waifu <command>");
    println!("\nA simple CLI to fetch images from nekos.best.\n");
    println!("Commands:");
    println!("  -c, --category <name>   Fetch an image from a specific category");
    println!("  -n, --batch <amount>    Use '-n <amount>' after category to batch download (e.g. -c waifu -n 50)");
    println!("  -l, --list              List all available categories");
    println!("  -o                      Open the image URL in the default system viewer");
    println!("  -t, --test              Test connectivity");
    println!("  -h, --help              Show this help message");
}

fn fetch_and_display_image(client: &Client, category: &str) {
    loop { // image fetch/display section
        let img_result = fetch_image(client, category);

        let img = match img_result {
            Ok(i) => i,
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        };

        let mut bytes = match client.get(&img.url).send().and_then(|resp| resp.bytes()) {
            Ok(b) => b.to_vec(),
            Err(_e) => {
                eprintln!("Error: Failed to download image from {}", img.url);
                break;
            }
        };

        let (cols, _) = size().unwrap_or((80, 24));
        let title = "--waifu-cli--";
        let title_padding = cols.saturating_sub(title.len() as u16) / 2;

        let _ = execute!(io::stdout(), MoveToColumn(title_padding));
        println!("{}", title);
        println!();

        let _ = execute!(io::stdout(), MoveToColumn(title_padding));
let mut displayed = false;
        let viewers = [
            ("kitty", vec!["+kitten", "icat"]),
            ("wezterm", vec!["imgcat"]),
            ("viu", vec!["-"]),
            ("chafa", vec!["-"]),
        ];

        for (cmd, args) in viewers {
            let mut command = Command::new(cmd);
            command.args(&args).stdin(Stdio::piped());

            if cmd != "kitty" {
                command.stdout(Stdio::piped());
            }

            if let Ok(mut child) = command.spawn() {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(&bytes);
                }

                if cmd != "kitty" {
                    let mut output_bytes = Vec::new();
                    if let Some(mut child_stdout) = child.stdout.take() {
                        use std::io::Read;
                        let _ = child_stdout.read_to_end(&mut output_bytes);
                    }
                    let _ = child.wait();

                    if !output_bytes.is_empty() {
                        let _ = execute!(io::stdout(), MoveToColumn(title_padding));
                        let _ = io::stdout().write_all(&output_bytes);
                        let _ = io::stdout().flush();
                        displayed = true;
                        break;
                    }
                } else {
                    let _ = execute!(io::stdout(), MoveToColumn(title_padding));
                    let _ = child.wait();
                    displayed = true;
                    break;
                }
            }
        }

        if !displayed {
            println!("No terminal image viewer found (kitty, wezterm, viu, chafa).");
            println!("Image URL: {}", img.url);
        }

        let mut continue_fetching = false;
        terminal::enable_raw_mode().unwrap();

        'input: loop {
            let print_prompt = || {
                let prompt = "[s]ave | [o]pen | [n]ext | [q]uit:";
                let (c, _) = size().unwrap_or((80, 24));
                let p_padding = c.saturating_sub(prompt.len() as u16) / 2;
                let _ = execute!(io::stdout(), MoveToColumn(p_padding));
                print!("{}", prompt);
                io::stdout().flush().unwrap();
            };

            print_prompt();

            if let Ok(Event::Key(key_event)) = event::read() {
                match key_event.code {
                    KeyCode::Char('s') => {
                        terminal::disable_raw_mode().unwrap();
                        let filename = img.url.split('/').last().unwrap_or("waifu.png");
                        if std::fs::write(filename, &bytes).is_ok() {
                            println!("Image saved as {}", filename);
                            println!();
                        } else {
                            eprintln!("Error: Failed to save image.");
                        }
                        terminal::enable_raw_mode().unwrap();
                        continue 'input;
                    }
                    KeyCode::Char('o') => {
                        let _ = if cfg!(target_os = "windows") {
                            Command::new("cmd").args(["/C", "start", &img.url]).spawn()
                        } else if cfg!(target_os = "macos") {
                            Command::new("open").arg(&img.url).spawn()
                        } else {
                            Command::new("xdg-open").arg(&img.url).spawn()
                        };
                    }
                    KeyCode::Char('n') => {
                        continue_fetching = true;
                        break 'input;
                    }
                    KeyCode::Enter | KeyCode::Char('q') => break 'input,
                    _ => {}
                }
            }
        }

        terminal::disable_raw_mode().unwrap();
        println!();

        if !continue_fetching {
            break;
        }

        bytes.zeroize();
    }
}

fn batch_download(client: &Client, category: &str, count: usize) {
    println!(
        "Starting batch download of {} images from category '{}'...",
        count, category
    );
    let start_time = Instant::now();

    let mut seen_urls: HashSet<String> = HashSet::new();
    let mut urls: Vec<String> = Vec::new();

    while urls.len() < count {
        let request_amount = std::cmp::min(count - urls.len(), 20);
        let resp_result: Result<ManyResp, _> = client
            .get(format!("{}/{}?amount={}", API, category, request_amount))
            .send()
            .and_then(|resp| resp.json());

        if let Ok(mut data) = resp_result {
            if data.results.is_empty() {
                break;
            }

            data.results
                .retain(|img| seen_urls.insert(img.url.clone()));

            urls.extend(data.results.into_iter().map(|img| img.url));
        } else {
            break;
        }

        if urls.len() >= count {
            break;
        }
    }

    urls.truncate(count);
    let count = urls.len();

    if count == 0 {
        eprintln!("Failed to fetch image URLs.");
        return;
    }

    let workers = std::cmp::min(count, 32);
    let (job_tx, job_rx) = mpsc::channel::<String>();
    let job_rx = Arc::new(Mutex::new(job_rx));
    let (res_tx, res_rx) = mpsc::channel();

    for url in urls {
        job_tx.send(url).unwrap();
    }
    drop(job_tx);

    let mut handles = Vec::new();

    for _ in 0..workers {
        let job_rx = Arc::clone(&job_rx);
        let res_tx = res_tx.clone();

        handles.push(thread::spawn(move || {
            let client = build_client().unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); });
            loop {
                let url = {
                    let lock = job_rx.lock().unwrap();
                    match lock.recv() {
                        Ok(u) => u,
                        Err(_) => break,
                    }
                };

                let start = Instant::now();
                if let Ok(bytes) = client.get(&url).send().and_then(|r| r.bytes()) {
                    let filename = url.split('/').last().unwrap_or("waifu.png");
                    let _ = std::fs::write(filename, &bytes);
                    let elapsed = start.elapsed().as_secs_f64();
                    let speed = bytes.len() as f64 / 1024.0 / elapsed;
                    let _ = res_tx.send(speed);
                } else {
                    let _ = res_tx.send(0.0);
                }
            }
        }));
    }

    for i in 0..count {
        let speed = res_rx.recv().unwrap_or(0.0) / 1024.0;
        let percentage = ((i + 1) * 100) / count;
        let bar_len = 30;
        let filled = bar_len * (i + 1) / count;
        let bar: String = "=".repeat(filled) + &" ".repeat(bar_len - filled);

        print!("\r[{}] {}% | Downloaded {}/{} | Speed: {:.2} MB/s",
            bar, percentage, i + 1, count, speed);
        io::stdout().flush().unwrap();
    }

    for h in handles {
        let _ = h.join();
    }

    println!(
        "\nCompleted {} images in {:.2?}",
        count,
        start_time.elapsed()
    );
}
