use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect, Spacing},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    DefaultTerminal, Frame,
};
use std::io;
use std::rc::Rc;
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

fn draw_layout(frame: &mut Frame) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .spacing(Spacing::Overlap(1))
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Fill(1),
        ])
        .split(frame.area())
}

impl Tui {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true
    }

    fn draw(&mut self, frame: &mut Frame) {
        let sections = draw_layout(frame);

        let title_block = Block::default().borders(Borders::ALL);
        let title = Paragraph::new(Text::styled(
            self.name.clone(),
            Style::default().fg(Color::Green).bold(),
        ))
        .alignment(Alignment::Center)
        .block(title_block);
        frame.render_widget(title, sections[0]);

        let instructions = Line::from(vec![
            Span::styled(" Up", Style::default()),
            Span::styled("<Up>", Style::default().fg(Color::Blue).bold()),
            Span::styled(" Down", Style::default()),
            Span::styled("<Down>", Style::default().fg(Color::Blue).bold()),
            Span::styled(" Start", Style::default()),
            Span::styled("<g>", Style::default().fg(Color::Blue).bold()),
            Span::styled(" End", Style::default()),
            Span::styled("<G>", Style::default().fg(Color::Blue).bold()),
            Span::styled(" Quit", Style::default()),
            Span::styled("<q>", Style::default().fg(Color::Blue).bold()),
            Span::styled(" | ", Style::default().fg(Color::White)),
            Span::styled(" Lines: ", Style::default()),
            Span::styled(
                format!("{}", self.entries.len()),
                Style::default().fg(Color::Blue).bold(),
            ),
        ]);
        let instruction_block = Block::default().borders(Borders::NONE);
        let instruction = Paragraph::new(instructions)
            .block(instruction_block)
            .alignment(Alignment::Center);
        frame.render_widget(instruction, sections[1]);

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
        let list_block = Block::default().borders(Borders::ALL);
        let list = List::new(lines)
            .block(list_block)
            .style(Style::default().white())
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">> ");
        frame.render_stateful_widget(list, sections[2], &mut self.nav_state);
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
