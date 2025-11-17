use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::{Color, SetForegroundColor, ResetColor},
    terminal::{self, Clear, ClearType},
};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::io::{stdout, Write};
use std::process::{Command, Stdio};

const API: &str = "https://api.waifu.pics";

#[derive(Debug, Deserialize)]
struct Endpoints {
    sfw: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ImageResp {
    url: String,
}

// selection menu
fn pick_sfw(list: &[String]) -> String {
    let mut index = 0;
    let mut stdout = stdout();

    // arrow key fix
    terminal::enable_raw_mode().unwrap();

    loop {
        // move cursor to top left and clear text below
        execute!(stdout, cursor::MoveTo(0,0), Clear(ClearType::FromCursorDown)).unwrap();

        for (i, item) in list.iter().enumerate() {
            if i == index {
                execute!(stdout, SetForegroundColor(Color::Yellow)).unwrap();
                print!("> {}", item);
                execute!(stdout, ResetColor).unwrap();
            } else {
                print!("  {}", item);
            }
            print!("\r\n");
        }

        print!("\r\n{}/{}", index + 1, list.len());
        stdout.flush().unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Up => if index > 0 { index -= 1 },
                KeyCode::Down => if index + 1 < list.len() { index += 1 },
                KeyCode::Enter => break,
                KeyCode::Esc => {
                    terminal::disable_raw_mode().unwrap();
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    }

    terminal::disable_raw_mode().unwrap();
    list[index].clone()
}

fn main() {
    let client = Client::new();

    // get categories
    let ep: Endpoints = client
        .get(format!("{}/endpoints", API))
        .send()
        .unwrap()
        .json()
        .unwrap();

    // allow tags if the user wants to specify content
    let args: Vec<String> = std::env::args().collect();
    let choice = if args.len() > 1 {
        args[1].clone()
    } else {
        pick_sfw(&ep.sfw)
    };

    // grab image URL (to be sent to terminal)
    let img: ImageResp = client
        .get(format!("{}/sfw/{}", API, choice))
        .send()
        .unwrap()
        .json()
        .unwrap();

    // cache images into memory
    let bytes = client.get(&img.url).send().unwrap().bytes().unwrap();

    // show image in terminal (kitty only)
    let mut child = Command::new("kitty")
        .args(["+kitten", "icat"])
        .stdin(Stdio::piped())
        .spawn()
        .expect("failed to run icat");

    child.stdin.as_mut().unwrap().write_all(&bytes).unwrap();
    child.wait().unwrap();
}
