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
    cursor::{Hide, MoveTo, MoveToColumn, Show},
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use zeroize::Zeroize;
use api::CATEGORIES;

mod api;
use api::{fetch_endpoints, fetch_image, build_client};

const API: &str = "https://nekos.best/api/v2";
const VERSION: &str = "0.1.6";

#[derive(Debug, Deserialize)]
struct ManyResp {
    results: Vec<ImageResp>,
}

#[derive(Default, Clone, Copy)]
struct DownloadFilters {
    min_size_kb: Option<f64>,
    min_width: Option<u32>,
    min_height: Option<u32>,
}

fn get_image_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 { return None; }
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        let w = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
        let h = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
        return Some((w, h));
    } else if bytes.starts_with(&[0xFF, 0xD8]) {
        let mut i = 2;
        while i + 9 < bytes.len() {
            if bytes[i] != 0xFF { i += 1; continue; }
            if bytes[i+1] == 0xC0 || bytes[i+1] == 0xC2 {
                let h = u16::from_be_bytes(bytes[i+5..i+7].try_into().ok()?) as u32;
                let w = u16::from_be_bytes(bytes[i+7..i+9].try_into().ok()?) as u32;
                return Some((w, h));
            }
            i += 2 + u16::from_be_bytes(bytes[i+2..i+4].try_into().ok()?) as usize;
        }
    }
    None
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

    let mut upscale = true;
    let mut args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|x| x == "--no-upscale") {
        upscale = false;
        args.remove(pos);
    }

    if args.len() < 2 {
        let category = CATEGORIES[rand::random::<usize>() % CATEGORIES.len()];
        fetch_and_display_image(&client, category, upscale);
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "-v" | "--version" => {
            println!("waifu-cli version {}", VERSION);
        }
        "-r" | "--random" => {
            let category = &categories[rand::random::<usize>() % categories.len()];
            fetch_and_display_image(&client, category, upscale);
        }
        "-o" | "--open" => {
            let category = &categories[rand::random::<usize>() % categories.len()];
            if let Ok(img) = fetch_image(&client, category) {
                println!("Opening random image: {}", img.url);
                let _ = if cfg!(target_os = "windows") {
                    Command::new("cmd").args(["/C", "start", &img.url]).spawn()
                } else if cfg!(target_os = "macos") {
                    Command::new("open").arg(&img.url).spawn()
                } else {
                    Command::new("xdg-open").arg(&img.url).spawn()
                };
            } else {
                eprintln!("Error: Failed to fetch image for opening.");
            }
        }
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
                        Ok(amount) => {
                            let mut filters = DownloadFilters::default();
                            let mut i = 5;
                            while i < args.len() {
                                match args[i].as_str() {
                                    "--min-size" => {
                                        if let Some(s) = args.get(i+1).and_then(|v| v.parse().ok()) { filters.min_size_kb = Some(s); }
                                        i += 2;
                                    }
                                    "--min-width" => {
                                        if let Some(w) = args.get(i+1).and_then(|v| v.parse().ok()) { filters.min_width = Some(w); }
                                        i += 2;
                                    }
                                    "--min-height" => {
                                        if let Some(h) = args.get(i+1).and_then(|v| v.parse().ok()) { filters.min_height = Some(h); }
                                        i += 2;
                                    }
                                    _ => i += 1,
                                }
                            }
                            batch_download(&client, category_name, amount, filters)
                        },
                        Err(_e) => {
                            eprintln!("Error: Invalid amount '{}'.", amount_str);
                            std::process::exit(1);
                        }
                    }
                } else {
                    fetch_and_display_image(&client, category_name, upscale);
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
    println!("  -r, --random            Fetch a random image from a random category");
    println!("  -v, --version           Show version information");
    println!("  -o                      Open the image URL in the default system viewer");
    println!("  -t, --test              Test connectivity");
    println!("  --min-size <KB>         Filter batch downloads by minimum file size");
    println!("  --min-width <pixels>    Filter batch downloads by minimum width");
    println!("  --min-height <pixels>   Filter batch downloads by minimum height");
    println!("  --no-upscale            Don't upscale small images to fit the terminal");
    println!("  --check-links           Perform a deep check of category endpoints");
    println!("  -h, --help              Show this help message");
}

fn render_image(bytes: &[u8], cols: u16, rows: u16, is_interactive: bool, upscale: bool) -> bool {
    let title = "--waifu-cli--";
    let title_padding = cols.saturating_sub(title.len() as u16) / 2;
    let _ = execute!(io::stdout(), MoveToColumn(title_padding));
    println!("{}", title);
    println!();

    let h_val = rows.saturating_sub(4).to_string();
    let w_val = cols.to_string();
    let place_val = format!("{}x{}@0x1", w_val, h_val);
    let chafa_size = format!("{}x{}", w_val, h_val);

    let mut kitty_args = vec!["+kitten", "icat", "--stdin", "yes"];
    if upscale {
        kitty_args.push("--scale-up");
    }
    if is_interactive {
        kitty_args.push("--place");
        kitty_args.push(&place_val);
    }

    let viewers: [(&str, Vec<&str>); 4] = [
        ("kitty", kitty_args.clone()),
        ("wezterm", vec!["imgcat", "--width", &w_val, "--height", &h_val]),
        ("viu", vec!["-w", &w_val, "-h", &h_val, "-"]),
        ("chafa", vec!["--size", &chafa_size, "-"]),
    ];

    for (cmd, args) in viewers {
        let mut command = Command::new(cmd);
        command.args(&args).stdin(Stdio::piped());

        if cmd != "kitty" {
            command.stdout(Stdio::piped());
        }

        if let Ok(mut child) = command.spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(bytes);
            }

            if cmd != "kitty" {
                let mut output_bytes = Vec::new();
                if let Some(mut child_stdout) = child.stdout.take() {
                    use std::io::Read;
                    let _ = child_stdout.read_to_end(&mut output_bytes);
                }
                let status = child.wait().ok();

                if status.map_or(false, |s| s.success()) && !output_bytes.is_empty() {
                    if is_interactive {
                        let _ = execute!(io::stdout(), MoveTo(0, 2));
                    }
                    let _ = io::stdout().write_all(&output_bytes);
                    let _ = io::stdout().flush();
                    return true;
                }
            } else {
                if let Ok(status) = child.wait() {
                    if status.success() {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn fetch_and_display_image(client: &Client, category: &str, upscale: bool) {
    let _ = execute!(io::stdout(), EnterAlternateScreen, Hide);
    let mut last_bytes: Vec<u8> = Vec::new();

    loop { // image fetch/display section
        let _ = execute!(io::stdout(), terminal::Clear(terminal::ClearType::All), MoveTo(0, 0));
        let img_result = fetch_image(client, category);

        let img = match img_result {
            Ok(i) => i,
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        };

        let response = client.get(&img.url).send();
        let mut bytes = match response {
            Ok(resp) => {
                let content_type = resp.headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("unknown");
                
                if !content_type.starts_with("image/") && !content_type.contains("octet-stream") {
                    eprintln!("Warning: URL may not be an image (Content-Type: {})", content_type);
                }

                resp.bytes().map(|b| b.to_vec()).unwrap_or_default()
            }
            Err(_) => Vec::new(),
        };

        if bytes.is_empty() {
            eprintln!("Error: Failed to download image from {}", img.url);
            break;
        };

        let (cols, rows) = size().unwrap_or((80, 24));
        
        if !render_image(&bytes, cols, rows, true, upscale) {
            println!("No terminal image viewer found (kitty, wezterm, viu, chafa).");
            println!("Image URL: {}", img.url);
        }

        let mut continue_fetching = false;
        terminal::enable_raw_mode().unwrap();

        'input: loop {
            let print_prompt = || {
                let prompt = "[s]ave | [u]rl | [a]rtist | [o]pen | [n]ext | [q]uit:";
                let (c, r) = size().unwrap_or((80, 24));
                let p_padding = c.saturating_sub(prompt.len() as u16) / 2;
                let _ = execute!(
                    io::stdout(),
                    MoveTo(p_padding, r.saturating_sub(1)),
                    terminal::Clear(terminal::ClearType::CurrentLine)
                );
                print!("{}", prompt);
                io::stdout().flush().unwrap();
            };

            print_prompt();

            if let Ok(ev) = event::read() {
                match ev {
                    Event::Key(key_event) => match key_event.code {
                    KeyCode::Char('s') => {
                        terminal::disable_raw_mode().unwrap();
                        let _ = execute!(io::stdout(), MoveToColumn(0), terminal::Clear(terminal::ClearType::CurrentLine));
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
                    KeyCode::Char('u') => {
                        terminal::disable_raw_mode().unwrap();
                        let _ = execute!(io::stdout(), MoveToColumn(0), terminal::Clear(terminal::ClearType::CurrentLine));
                        println!("\nImage URL: {}", img.url);
                        terminal::enable_raw_mode().unwrap();
                        print_prompt();
                        continue 'input;
                    }
                    KeyCode::Char('a') => {
                        terminal::disable_raw_mode().unwrap();
                        let _ = execute!(io::stdout(), MoveToColumn(0), terminal::Clear(terminal::ClearType::CurrentLine));
                        println!("--- Metadata ---");
                        println!("Artist: {}", img.artist_name.as_deref().unwrap_or("Unknown"));
                        if let Some(ref href) = img.artist_href { println!("Artist Link: {}", href); }
                        if let Some(ref src) = img.source_url { println!("Source: {}", src); }
                        println!("----------------");
                        println!("(Press any key to return)");
                        let _ = event::read();
                        terminal::enable_raw_mode().unwrap();
                        let _ = execute!(io::stdout(), terminal::Clear(terminal::ClearType::FromCursorUp));
                        print_prompt();
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
                    },
                    Event::Resize(new_cols, new_rows) => {
                        // Re-render the image with new dimensions
                        let _ = execute!(io::stdout(), terminal::Clear(terminal::ClearType::All), MoveTo(0, 0));
                        if !render_image(&bytes, new_cols, new_rows, true, upscale) {
                            println!("No terminal image viewer found (kitty, wezterm, viu, chafa).");
                            println!("Image URL: {}", img.url);
                        }
                    }
                    _ => {}
                }
            }
        }

        terminal::disable_raw_mode().unwrap();
        println!();

        last_bytes = bytes.clone();
        if !continue_fetching {
            break;
        }

        bytes.zeroize();
    }

    let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);

    // Print the last viewed image to the main terminal buffer on exit
    if !last_bytes.is_empty() {
        let (cols, rows) = size().unwrap_or((80, 24));
        render_image(&last_bytes, cols, rows, false, upscale);
        last_bytes.zeroize();
    }
}

fn batch_download(client: &Client, category: &str, count: usize, filters: DownloadFilters) {
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
                let mut success = false;
                for _ in 0..3 { // Retry up to 3 times
                    if let Ok(bytes) = client.get(&url).send().and_then(|r| r.bytes()) {
                        let size_kb = bytes.len() as f64 / 1024.0;
                        let dims = get_image_dimensions(&bytes);
                        
                        let size_ok = filters.min_size_kb.map_or(true, |s| size_kb >= s);
                        let width_ok = filters.min_width.map_or(true, |w| dims.map_or(false, |(dw, _)| dw >= w));
                        let height_ok = filters.min_height.map_or(true, |h| dims.map_or(false, |(_, dh)| dh >= h));

                        if size_ok && width_ok && height_ok {
                            let filename = url.split('/').last().unwrap_or("waifu.png");
                            let _ = std::fs::write(filename, &bytes);
                            let elapsed = start.elapsed().as_secs_f64();
                            let speed = bytes.len() as f64 / 1024.0 / elapsed;
                            let _ = res_tx.send(speed);
                            success = true;
                            break;
                        } else {
                            break; // Filter failed, don't retry this URL
                        }
                    }
                    thread::sleep(std::time::Duration::from_millis(500));
                }
                
                if !success {
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
