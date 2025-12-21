use chrono::{self, DateTime, Utc};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use regex::Regex;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::Path;

fn main() {
    let args = Args::parse();
    let root_dir = args.root_dir.as_str();
    let key = args.key;

    let root_path = Path::new(root_dir);
    let mut entries: Vec<Entry> = Vec::new();
    search_tree(root_path, &key, &mut entries, &search).unwrap();

    entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    for entry in &entries {
        println!("{}", entry);
    }

    let mut terminal = ratatui::init();
    let result = Tui::default().run(&mut terminal);
    ratatui::restore();
    match result {
        Ok(_) => println!("done"),
        Err(e) => panic!("{}", e),
    };
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
                let identifier: Vec<&str> = path.to_str().unwrap().split('/').collect();
                let timestamp = regex_dt.find(content.as_str()).unwrap();
                let timestamp_fixed_offset = DateTime::parse_from_rfc3339(timestamp.as_str()).unwrap();
                let level = match regex_lv.find(content.as_str()) {
                    None => "unknown",
                    Some(r) => r.as_str(),
                };

                let entry = Entry {
                    component: Component {
                        name: String::from(identifier[5]),
                        namespace: String::from(identifier[4]),
                    },
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
    component: Component,
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

#[derive(Debug)]
struct Component {
    name: String,
    namespace: String,
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.namespace, self.name)
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

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Widget for &Tui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Log Finder ".bold());
        let instructions = Line::from(vec![
            " Up".into(),
            "<Up>".blue().bold(),
            " Down".into(),
            "<Down>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = "hello";
        Paragraph::new(counter_text).block(block).render(area, buf);
    }
}
