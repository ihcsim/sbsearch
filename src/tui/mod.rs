use ratatui::{
    DefaultTerminal, Frame,
    widgets::{ListState, ScrollbarState},
};
use std::error::Error;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use tui_input::Input;

use super::sbsearch;

mod event;
mod render;

const DEFAULT_MAX_ENTRIES_PER_PAGE: usize = 100;

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

    page_final: usize,
    page_goto: usize,
    page_max_entries: usize,
    page_reload: bool,

    last_saved_filename: String,
}

#[derive(Debug, Default, PartialEq)]
enum Screen {
    #[default]
    Main,
    ConfirmExit,
    ConfirmSave,
}

#[derive(Debug, Default, PartialEq, Clone)]
enum SearchMode {
    #[default]
    Normal,
    Insert,
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

            last_saved_filename: String::new(),
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
                Screen::ConfirmSave => {
                    let filename =
                        format!("sbsearch_{}.log", chrono::Utc::now().format("%Y%m%d%H%M%S"));
                    let text = format!("save search result to ./{}? (y/n)", filename);
                    self.last_saved_filename = filename;
                    self.draw_popup("Confirm Save", text.as_str(), 40, 15, frame);
                }
                _ => self.draw_main(frame),
            })?;
            event::handle(self)?;
        }
        Ok(())
    }

    fn read_entries_from_sb(&mut self) {
        let root_path = Path::new(self.sbpath.as_str());
        let keyword = self.keyword.as_str();
        let offset = self.page_goto * self.page_max_entries - self.page_max_entries;
        let limit = self.page_max_entries;
        let cache = &mut self.entries_cache;

        self.entries_offset = match sbsearch::search(root_path, keyword, offset, limit, cache) {
            Ok(result) => result.entries_offset,
            Err(_) => Vec::new(),
        };
        self.page_final = self.entries_cache.len().div_ceil(self.page_max_entries);
        self.page_reload = false;
        self.nav_state = ListState::default().with_selected(Some(0));
    }

    fn save_to_file(&mut self) -> io::Result<()> {
        if let Ok(file) = std::fs::File::create(&self.last_saved_filename) {
            let mut writer = BufWriter::new(&file);
            for entry in &self.entries_cache {
                write!(writer, "{}", entry)?;
            }
        }
        self.current_screen = Screen::Main;
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true
    }

    fn draw_main(&mut self, frame: &mut Frame) {
        let sections = render::split_main_layout(frame.area());
        let offset = self.page_goto * self.page_max_entries - self.page_max_entries;
        let (filepath, selected) = match self.nav_state.selected() {
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
        let scroll_width = sections[2].width.max(3) - 3;
        let search_scroll = self.search_input.visual_scroll(scroll_width as usize);
        let search_cursor_pos =
            self.search_input.visual_cursor().max(search_scroll) - search_scroll + 8;
        let search_cursor_show = self.search_mode == SearchMode::Insert;

        let mut r = render::Renderer::new(
            String::from(filepath),
            self.keyword.clone(),
            self.page_final,
            self.page_goto,
            self.entries_cache.len(),
            selected,
            self.sbpath.clone(),
            search_cursor_pos as u16,
            search_cursor_show,
            search_scroll as u16,
            self.search_input.value().to_string(),
            &self.entries_offset,
            &mut self.nav_state,
            self.vertical_scroll_state,
        );
        r.render_title_section(sections[0], frame);
        r.render_meta_section(sections[1], frame);
        r.render_search_section(sections[2], frame);
        r.render_logs_section(sections[3], frame);
    }

    fn draw_popup(&self, title: &str, text: &str, width: u16, height: u16, frame: &mut Frame) {
        render::draw_popup(title, text, width, height, frame);
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
