use crossterm::{cursor, event::{self, Event, KeyCode}, execute, queue, style::{Color, SetForegroundColor, ResetColor}, terminal::{self, Clear, ClearType}};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::io::{stdout, Write};
use std::process::{Command, Stdio};

const API: &str = "https://api.waifu.pics";

#[derive(Debug, Deserialize)]
struct Endpoints { sfw: Vec<String>, }

#[derive(Debug, Deserialize)]
struct ImageResp { url: String, }

fn pick_sfw(list: &[String]) -> String {
    let mut index = 0;
    let mut stdout = stdout();
    let mut handle = stdout.lock();
    terminal::enable_raw_mode().unwrap();
    loop {
        queue!(handle, cursor::MoveTo(0, 0), Clear(ClearType::FromCursorDown)).unwrap();
        for (i, item) in list.iter().enumerate() {
            if i == index {
                queue!(handle, SetForegroundColor(Color::Yellow)).unwrap();
                write!(handle, "> {}", item).unwrap();
                queue!(handle, ResetColor).unwrap();
            } else {
                write!(handle, "  {}", item).unwrap();
            }
            write!(handle, "\r\n").unwrap();
        }
        write!(handle, "\r\n{}/{}", index + 1, list.len()).unwrap();
        handle.flush().unwrap();
        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Up => index = if index == 0 { list.len() - 1 } else { index - 1 },
                KeyCode::Down => index = if index == list.len() - 1 { 0 } else { index + 1 },
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
    let mut stdout = stdout();
    let client = Client::new();
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();
    let ep: Endpoints = client.get(format!("{}/endpoints", API)).send().unwrap().json().unwrap();
    let mut sfw_sorted = ep.sfw.clone();
    sfw_sorted.sort_unstable();
    let args: Vec<String> = std::env::args().collect();
    let choice = if args.len() > 1 { args[1].clone() } else { pick_sfw(&sfw_sorted) };
    let img: ImageResp = client.get(format!("{}/sfw/{}", API, choice)).send().unwrap().json().unwrap();
    let mut child = Command::new("kitty").args(["+kitten", "icat"]).stdin(Stdio::piped()).spawn().expect("failed to run icat");
    let mut resp = client.get(&img.url).send().unwrap();
    {
        let stdin = child.stdin.as_mut().expect("failed to open child stdin");
        resp.copy_to(stdin).unwrap();
    }
    let menu_height = sfw_sorted.len() + 3;
    execute!(stdout, cursor::MoveTo(0, menu_height as u16)).unwrap();
    child.wait().unwrap();
}