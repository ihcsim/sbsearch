use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
};
use std::error::Error;
use std::io;
use std::path::Path;
use std::rc::Rc;
use textwrap::Options;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use super::sbsearch;

const DEFAULT_MAX_ENTRIES_PER_PAGE: u32 = 100;

#[derive(Debug, Default)]
pub struct Tui {
    current_screen: Screen,
    entries_cache: Vec<sbsearch::Entry>,
    entries_offset: Vec<sbsearch::Entry>,
    exit: bool,
    nav_state: ListState,
    keyword: String,
    search: String,
    search_input: Input,
    search_mode: SearchMode,
    sbpath: String,
    vertical_scroll_state: ScrollbarState,
    vertical_scroll: usize,

    page_final: u32,
    page_goto: u32,
    page_max_entries: u32,
    page_reload: bool,
}

#[derive(Debug, Default, PartialEq)]
enum Screen {
    #[default]
    Main,
    ConfirmExit,
}

#[derive(Debug, Default, PartialEq)]
enum SearchMode {
    #[default]
    Normal,
    Insert,
}

fn split_main_layout(r: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .split(r)
}

fn split_popup_layout(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_area[1])[1]
}

impl Tui {
    pub fn new(support_bundle_path: &str, keyword: &str) -> Self {
        Self {
            current_screen: Screen::Main,
            entries_offset: Vec::new(),
            entries_cache: Vec::new(),
            exit: false,
            nav_state: ListState::default().with_selected(Some(0)),
            keyword: String::from(keyword),
            search: String::new(),
            search_input: Input::default(),
            search_mode: SearchMode::default(),
            sbpath: String::from(support_bundle_path),
            vertical_scroll_state: ScrollbarState::default(),
            vertical_scroll: 0,

            page_final: 1,
            page_goto: 1,
            page_max_entries: DEFAULT_MAX_ENTRIES_PER_PAGE,
            page_reload: true,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
        while !self.exit {
            if self.page_reload {
                self.read_entries_from_sb();
            }

            terminal.draw(|frame| match self.current_screen {
                Screen::ConfirmExit => self.draw_popup(
                    "Confirm Exit",
                    "are you sure you want to exit? (y/n)",
                    30,
                    15,
                    frame,
                ),
                _ => self.draw_main(frame),
            })?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn read_entries_from_sb(&mut self) {
        let root_path = Path::new(self.sbpath.as_str());
        let keyword = self.keyword.as_str();
        let offset = (self.page_goto * self.page_max_entries - self.page_max_entries) as usize;
        let limit = self.page_max_entries as usize;
        let cache = &mut self.entries_cache;

        self.entries_offset = match sbsearch::search(root_path, keyword, offset, limit, cache) {
            Ok(result) => result.entries_offset,
            Err(_) => Vec::new(),
        };
        self.page_final = (self.entries_cache.len() as u32).div_ceil(self.page_max_entries);
        self.page_reload = false;
        self.nav_state = ListState::default().with_selected(Some(0));
    }

    fn exit(&mut self) {
        self.exit = true
    }

    fn draw_main(&mut self, frame: &mut Frame) {
        let sections = split_main_layout(frame.area());
        self.render_title_section(sections[0], frame);
        self.render_meta_section(sections[1], frame);
        self.render_search_section(sections[2], frame);
        self.render_logs_section(sections[3], frame);
    }

    fn draw_popup(
        &mut self,
        title: &str,
        text: &str,
        percent_x: u16,
        percent_y: u16,
        frame: &mut Frame,
    ) {
        let popup_area = split_popup_layout(percent_x, percent_y, frame.area());
        let popup_block = Block::default()
            .title(Line::from(title).centered())
            .borders(Borders::ALL)
            .style(Style::default());
        let popup_para = Paragraph::new(text)
            .block(popup_block)
            .alignment(Alignment::Center);
        frame.render_widget(popup_para, popup_area);
    }

    fn render_title_section(&mut self, area: Rect, frame: &mut Frame) {
        let instructions = Line::from(vec![
            Span::styled(" | (Line)", Style::default().fg(Color::White)),
            Span::styled(" Up", Style::default()),
            Span::styled("<Up>", Style::default().fg(Color::Blue).bold()),
            Span::styled(" Down", Style::default()),
            Span::styled("<Down>", Style::default().fg(Color::Blue).bold()),
            Span::styled(" Start", Style::default()),
            Span::styled("<g>", Style::default().fg(Color::Blue).bold()),
            Span::styled(" End", Style::default()),
            Span::styled("<G>", Style::default().fg(Color::Blue).bold()),
            Span::styled(" | (Page)", Style::default().fg(Color::White)),
            Span::styled(" Previous", Style::default()),
            Span::styled("<Left>", Style::default().fg(Color::Blue).bold()),
            Span::styled(" Next", Style::default()),
            Span::styled("<Right>", Style::default().fg(Color::Blue).bold()),
            Span::styled(" | (Search)", Style::default().fg(Color::White)),
            Span::styled(" Edit", Style::default()),
            Span::styled("<s>", Style::default().fg(Color::Blue).bold()),
            Span::styled(" Clear", Style::default()),
            Span::styled("<c>", Style::default().fg(Color::Blue).bold()),
            Span::styled(" | ", Style::default().fg(Color::White)),
            Span::styled(" Quit", Style::default()),
            Span::styled("<q>", Style::default().fg(Color::Blue).bold()),
            Span::styled(" | ", Style::default().fg(Color::White)),
        ]);
        let title_block = Block::default()
            .borders(Borders::ALL)
            .title_bottom(instructions)
            .title_alignment(Alignment::Center);
        let title_para = Paragraph::new(Text::styled(
            self.sbpath.clone(),
            Style::default().fg(Color::Green).bold(),
        ))
        .alignment(Alignment::Center)
        .block(title_block);
        frame.render_widget(title_para, area);
    }

    fn render_meta_section(&mut self, area: Rect, frame: &mut Frame) {
        let offset = (self.page_goto * self.page_max_entries - self.page_max_entries) as usize;
        let (path, pos) = match self.nav_state.selected() {
            Some(pos) => {
                if self.entries_offset.is_empty() {
                    ("", 0)
                } else {
                    let path_str = self.entries_offset[pos].path.as_str();
                    let name_str = self.sbpath.as_str();
                    if let Some(index) = path_str.find(name_str) {
                        (
                            &path_str[index + name_str.len()..path_str.len()],
                            offset + pos + 1,
                        )
                    } else {
                        ("", 0)
                    }
                }
            }
            None => ("", 0),
        };

        let meta_block = Block::default().borders(Borders::ALL);
        let meta_lines = vec![
            Line::from(vec![
                Span::styled("Keyword: ", Style::default().fg(Color::Green).bold()),
                Span::styled(&self.keyword, Style::default().fg(Color::Green).bold()),
                Span::styled(" | ", Style::default().fg(Color::White)),
                Span::styled("Line: ", Style::default().fg(Color::Green).bold()),
                Span::styled(
                    format!("{}/{}", pos, self.entries_cache.len()),
                    Style::default().fg(Color::Green).bold(),
                ),
                Span::styled(" | ", Style::default().fg(Color::White)),
                Span::styled("Page: ", Style::default().fg(Color::Green).bold()),
                Span::styled(
                    format!("{}/{}", self.page_goto, self.page_final),
                    Style::default().fg(Color::Green).bold(),
                ),
            ]),
            Line::from(vec![
                Span::styled("Filepath: ", Style::default().fg(Color::Green).bold()),
                Span::styled(path, Style::default().fg(Color::Green).bold()),
            ]),
        ];
        let meta_para = Paragraph::new(meta_lines)
            .block(meta_block)
            .alignment(Alignment::Center);
        frame.render_widget(meta_para, area);
    }

    fn render_search_section(&mut self, area: Rect, frame: &mut Frame) {
        let search_block = Block::default().borders(Borders::ALL);
        let width = area.width.max(3) - 3;
        let scroll = self.search_input.visual_scroll(width as usize);
        let search_lines = Line::from(vec![
            Span::styled("Search: ", Style::default().fg(Color::Green).bold()),
            Span::styled(self.search_input.value(), Style::default()),
        ]);
        let input = Paragraph::new(search_lines)
            .style(Style::default())
            .scroll((0, scroll as u16))
            .block(search_block);
        frame.render_widget(input, area);

        // show cursor only in insert mode
        if self.search_mode == SearchMode::Insert {
            let x = self.search_input.visual_cursor().max(scroll) - scroll + 8;
            frame.set_cursor_position((area.x + x as u16, area.y + 1));
        }
    }

    fn render_logs_section(&mut self, area: Rect, frame: &mut Frame) {
        let mut lines: Vec<ListItem> = self
            .entries_offset
            .iter()
            .map(|entry| {
                let width = frame.area().as_size().width as usize;
                let options = Options::new(width);
                let text = format!("{}", entry);
                let wrapped = textwrap::fill(text.as_str(), options);
                let list_item = match entry.level.as_str() {
                    "error" => ListItem::new(wrapped).red(),
                    "warning" => ListItem::new(wrapped).yellow(),
                    _ => ListItem::new(wrapped),
                };
                if !self.search.is_empty()
                    && text
                        .clone()
                        .to_lowercase()
                        .contains(self.search.clone().to_lowercase().as_str())
                {
                    list_item.on_blue()
                } else {
                    list_item
                }
            })
            .collect();
        if lines.is_empty() {
            lines = vec![ListItem::new("No log entries found.".to_string())];
        }

        let lines_count = lines.len();
        let list_block = Block::default().borders(Borders::ALL);
        let list = List::new(lines)
            .block(list_block)
            .style(Style::default())
            .highlight_symbol(">> ")
            .highlight_style(Style::default().bg(Color::Magenta));
        frame.render_stateful_widget(list, area, &mut self.nav_state);

        // render scrollbar
        self.vertical_scroll_state = self.vertical_scroll_state.content_length(lines_count);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            area,
            &mut self.vertical_scroll_state,
        );
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;
        self.handle_key_event(event);
        Ok(())
    }

    fn handle_key_event(&mut self, event: Event) {
        if let Event::Key(key_event) = event {
            if key_event.kind != KeyEventKind::Press {
                return;
            }

            match self.search_mode {
                SearchMode::Normal => match self.current_screen {
                    Screen::Main => match key_event.code {
                        KeyCode::Char('q') => self.current_screen = Screen::ConfirmExit,
                        KeyCode::Char('G') => self.nav_end(),
                        KeyCode::Char('g') => self.nav_first_line(),
                        KeyCode::Char('s') => {
                            self.search_mode = SearchMode::Insert;
                            self.search_input.reset();
                        }
                        KeyCode::Char('c') => self.search = String::new(),
                        KeyCode::Up | KeyCode::Char('k') => self.nav_prev_line(),
                        KeyCode::Down | KeyCode::Char('j') => self.nav_next_line(),
                        KeyCode::Left => self.nav_prev_page(),
                        KeyCode::Right => self.nav_next_page(),
                        _ => {}
                    },
                    Screen::ConfirmExit => match key_event.code {
                        KeyCode::Char('y') => self.exit(),
                        KeyCode::Char('n') => self.current_screen = Screen::Main,
                        _ => {}
                    },
                },
                SearchMode::Insert => match key_event.code {
                    KeyCode::Enter => {
                        self.search = String::from(self.search_input.value());
                        self.search_mode = SearchMode::Normal;
                    }
                    KeyCode::Esc => {
                        self.search = String::new();
                        self.search_input.reset();
                        self.search_mode = SearchMode::Normal;
                    }
                    _ => {
                        self.search_input.handle_event(&event);
                    }
                },
            }
        }
    }

    fn nav_next_line(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_add(1);
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
        let i = match self.nav_state.selected() {
            Some(i) => {
                if i < self.entries_offset.len() - 1 {
                    i + 1
                } else {
                    i
                }
            }
            None => 0,
        };
        self.nav_state.select(Some(i));
    }

    fn nav_prev_line(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
        let i = match self.nav_state.selected() {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else {
                    i
                }
            }
            None => 0,
        };
        self.nav_state.select(Some(i));
    }

    fn nav_first_line(&mut self) {
        self.vertical_scroll_state = self.vertical_scroll_state.position(0);
        self.nav_state.select(Some(0));
    }

    fn nav_end(&mut self) {
        if !self.entries_offset.is_empty() {
            let end = self.entries_offset.len() - 1;
            self.vertical_scroll_state = self.vertical_scroll_state.position(end);
            self.nav_state.select(Some(end));
        }
    }

    fn nav_next_page(&mut self) {
        if self.page_goto < self.page_final {
            self.page_goto = self.page_goto.saturating_add(1);
            self.page_reload = true;
        }
    }

    fn nav_prev_page(&mut self) {
        if self.page_goto > 1 {
            self.page_goto = self.page_goto.saturating_sub(1);
            self.page_reload = true;
        }
    }
}

#[test]
fn new_and_handle_key_event() -> io::Result<()> {
    use crossterm::event::{KeyEvent, KeyModifiers};

    let mut tui = Tui::new("sb_path", "pvc_name");
    tui.entries_offset = vec![
        super::sbsearch::Entry {
            level: String::from("level=info"),
            path: String::from("/path/to/log1"),
            content: String::from("This is an info log entry."),
            timestamp: chrono::Utc::now(),
        },
        super::sbsearch::Entry {
            level: String::from("level=warning"),
            path: String::from("/path/to/log2"),
            content: String::from("This is an warning log entry."),
            timestamp: chrono::Utc::now(),
        },
        super::sbsearch::Entry {
            level: String::from("level=error"),
            path: String::from("/path/to/log3"),
            content: String::from("This is an error log entry."),
            timestamp: chrono::Utc::now(),
        },
    ];

    assert_eq!(tui.sbpath, "sb_path");
    assert_eq!(tui.keyword, "pvc_name");
    assert_eq!(tui.current_screen, Screen::Main);

    // navigation keys
    let key_event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    let event = Event::Key(key_event);
    tui.handle_key_event(event);
    assert_eq!(tui.nav_state.selected(), Some(1));

    let key_event = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
    let event = Event::Key(key_event);
    tui.handle_key_event(event);
    assert_eq!(tui.nav_state.selected(), Some(2));

    let key_event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
    let event = Event::Key(key_event);
    tui.handle_key_event(event);
    assert_eq!(tui.nav_state.selected(), Some(1));

    let key_event = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
    let event = Event::Key(key_event);
    tui.handle_key_event(event);
    assert_eq!(tui.nav_state.selected(), Some(0));

    let key_event = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
    let event = Event::Key(key_event);
    tui.handle_key_event(event);
    assert_eq!(tui.nav_state.selected(), Some(2));

    let key_event = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    let event = Event::Key(key_event);
    tui.handle_key_event(event);
    assert_eq!(tui.nav_state.selected(), Some(0));

    // search mode
    let key_event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
    let event = Event::Key(key_event);
    tui.handle_key_event(event);
    assert_eq!(tui.search_mode, SearchMode::Insert);

    tui.search_input = tui
        .search_input
        .with_value(String::from("test input value"));
    let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let event = Event::Key(key_event);
    tui.handle_key_event(event);
    assert_eq!(tui.search, String::from("test input value"));
    assert_eq!(tui.search_mode, SearchMode::Normal);

    // clear search
    let key_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
    let event = Event::Key(key_event);
    tui.handle_key_event(event);
    assert_eq!(tui.search, String::new());

    // confirm exit popup
    let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
    let event = Event::Key(key_event);
    tui.handle_key_event(event);
    assert_eq!(tui.current_screen, Screen::ConfirmExit);

    let key_event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
    let event = Event::Key(key_event);
    tui.handle_key_event(event);
    assert!(tui.exit);
    tui.current_screen = Screen::Main;

    Ok(())
}
