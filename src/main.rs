use chrono::{self, DateTime, Utc};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    style::{Modifier, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, ListItem, ListState},
    DefaultTerminal, Frame,
};
use regex::Regex;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::Path;

fn main() -> io::Result<()> {
    let args = Args::parse();
    let root_dir = args.root_dir.as_str();
    let key = args.key;

    let root_path = Path::new(root_dir);
    let mut entries: Vec<Entry> = Vec::new();
    search_tree(root_path, &key, &mut entries, &search).unwrap();
    entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    let mut terminal = ratatui::init();
    let app_out = Tui::new(entries).run(&mut terminal);
    ratatui::restore();
    app_out
}

fn search_tree(
    dir: &Path,
    key: &str,
    v: &mut Vec<Entry>,
    callback: &dyn Fn(&Path, &mut Vec<Entry>, &str),
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            search_tree(&path, key, v, callback)?;
            continue;
        }

        if path.is_file() {
            callback(&path, v, key);
            continue;
        }

        println!("skipping {}", path.display())
    }
    Ok(())
}

fn search(path: &Path, v: &mut Vec<Entry>, s: &str) {
    let regex_dt = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z").unwrap();
    let regex_lv = Regex::new(r"level=([^\s]+)").unwrap();

    if let Ok(file) = File::open(path) {
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            if let Ok(content) = line
                && content.contains(s)
            {
                let timestamp = regex_dt.find(content.as_str()).unwrap();
                let timestamp_fixed_offset = DateTime::parse_from_rfc3339(timestamp.as_str()).unwrap();
                let level = match regex_lv.find(content.as_str()) {
                    None => "unknown",
                    Some(r) => r.as_str(),
                };

                let entry = Entry {
                    content: content.clone(),
                    level: String::from(level),
                    path: String::from(path.to_str().unwrap()),
                    timestamp: timestamp_fixed_offset.with_timezone(&Utc),
                };

                v.push(entry);
            }
        }
    } else {
        println!("could no open file: {}", path.display());
    }
}

#[derive(Debug)]
struct Entry {
    content: String,
    level: String,
    path: String,
    timestamp: DateTime<Utc>,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = self.content.clone();
        let msg = match self.level.as_str() {
            "error" => out.red(),
            "warn" => out.yellow(),
            "info" => out.green(),
            "debug" => out.cyan(),
            _ => out.blue(),
        };
        write!(f, "{}", msg)
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    root_dir: String,

    #[arg(short, long)]
    key: String,
}

#[derive(Debug, Default)]
struct Tui {
    entries: Vec<Entry>,
    nav_state: ListState,
    exit: bool,
}

impl Tui {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn new(entries: Vec<Entry>) -> Self {
        Tui {
            entries,
            exit: false,
            nav_state: ListState::default().with_selected(Some(0)),
        }
    }
    fn nav_next(&mut self) {
        let i = match self.nav_state.selected() {
            Some(i) => {
                if i >= self.entries.len() - 1 {
                    0 // Wrap around to the start
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.nav_state.select(Some(i));
    }

    fn nav_prev(&mut self) {
        let i = match self.nav_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.entries.len() - 1 // Wrap around to the end
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.nav_state.select(Some(i));
    }

    fn draw(&mut self, frame: &mut Frame) {
        let title = Line::from(" Support Bundle Log Finder ".bold());
        let instructions = Line::from(vec![
            " Up".into(),
            "<Up>".blue().bold(),
            " Down".into(),
            "<Down>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
            " Lines: ".into(),
            format!("{}", self.entries.len()).blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);
        let lines: Vec<ListItem> = self
            .entries
            .iter()
            .map(|i| ListItem::new(format!("{}", i)))
            .collect();
        let list = List::new(lines)
            .block(block)
            .style(Style::default().white())
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">> ");
        frame.render_stateful_widget(list, frame.area(), &mut self.nav_state);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Up => self.nav_prev(),
            KeyCode::Down => self.nav_next(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true
    }
}

#[test]
fn handle_key_event() -> io::Result<()> {
    let entries: Vec<Entry> = Vec::new();
    let mut tui = Tui::new(entries);

    tui.handle_key_event(KeyCode::Char('q').into());
    assert!(tui.exit);

    Ok(())
}
