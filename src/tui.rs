use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    style::{Modifier, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, ListItem, ListState},
    DefaultTerminal, Frame,
};
use std::io;
use textwrap::Options;

#[derive(Debug, Default)]
pub struct Tui {
    name: String,
    entries: Vec<super::sbfind::Entry>,
    exit: bool,
    nav_state: ListState,
}

pub fn new(support_bundle_name: String, entries: Vec<super::sbfind::Entry>) -> Tui {
    Tui {
        name: support_bundle_name,
        entries,
        exit: false,
        nav_state: ListState::default().with_selected(Some(0)),
    }
}

impl Tui {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
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

    fn nav_start(&mut self) {
        self.nav_state.select(Some(0));
    }

    fn nav_end(&mut self) {
        let i = self.entries.len();
        self.nav_state.select(Some(i));
    }

    fn draw(&mut self, frame: &mut Frame) {
        let title = Line::from(self.name.as_str().bold());
        let instructions = Line::from(vec![
            " Up".into(),
            "<Up>".blue().bold(),
            " Down".into(),
            "<Down>".blue().bold(),
            " Start".into(),
            "<g> ".blue().bold(),
            " End".into(),
            "<G> ".blue().bold(),
            " Quit ".into(),
            "<q> ".blue().bold(),
            " Lines: ".into(),
            format!("{}", self.entries.len()).blue().bold(),
        ]);
        let lines: Vec<ListItem> = self
            .entries
            .iter()
            .map(|entry| {
                let width = frame.area().as_size().width as usize;
                let options = Options::new(width);
                let text = format!("{}", entry);
                let wrapped = textwrap::fill(text.as_str(), options);

                match entry.level.as_str() {
                    "level=error" => ListItem::new(wrapped).red(),
                    "level=info" => ListItem::new(wrapped),
                    "level=warning" => ListItem::new(wrapped).yellow(),
                    _ => ListItem::new(wrapped),
                }
            })
            .collect();
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);
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
            KeyCode::Char('G') => self.nav_end(),
            KeyCode::Char('g') => self.nav_start(),
            KeyCode::Up | KeyCode::Char('k') => self.nav_prev(),
            KeyCode::Down | KeyCode::Char('j') => self.nav_next(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true
    }
}

#[test]
fn handle_key_event() -> io::Result<()> {
    let entries: Vec<super::sbfind::Entry> = Vec::new();
    let mut tui = new(String::new(), entries);

    tui.handle_key_event(KeyCode::Char('g').into());
    assert_eq!(tui.nav_state.selected(), Some(0));

    tui.handle_key_event(KeyCode::Char('G').into());
    assert_eq!(tui.nav_state.selected(), Some(0));

    tui.handle_key_event(KeyCode::Char('q').into());
    assert!(tui.exit);

    Ok(())
}
