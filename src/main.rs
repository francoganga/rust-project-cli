use std::time::{Duration, Instant};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::io;
use tui::Terminal;
use tui::backend::CrosstermBackend;
use tui::layout::{Layout, Rect, Direction, Constraint, Alignment};
use tui::widgets::{Widget, Block, Borders, Paragraph, BorderType};
use tui::style::{Color, Style};
use crossterm::terminal::*;
use std::sync::mpsc;
use std::thread;
use crossterm::event::{Event as CEvent, KeyCode};
use crossterm::event;



const DB_PATH: &str = "./data/db.json";

#[derive(Serialize, Deserialize, Clone)]
struct Pet {
    id: usize,
    name: String,
    category: String,
    age: usize,
    created_at: DateTime<Utc>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    Pets,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Pets => 1,
        }
    }
}



pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    enable_raw_mode().expect("can run in raw mode");
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let (sender, receiver) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    thread::spawn(move || {
        let mut last_tick = Instant::now();

        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("Poll works?") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    sender.send(Event::Input(key)).expect("can send events");
                }

            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = sender.send(Event::Tick) {
                    last_tick = Instant::now();
                }

            }


        }
    });

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                    Constraint::Length(3),
                    Constraint::Min(2),
                    Constraint::Length(3)
                    ].as_ref()).split(size);

            let copyright = Paragraph::new("pet-CLI 2020 - all rights reserved")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Copyright")
                        .border_type(BorderType::Plain),
                );

            rect.render_widget(copyright, chunks[2]);

        })?;

        match receiver.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                },
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}
