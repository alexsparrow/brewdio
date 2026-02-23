use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub struct SearchItem {
    pub label: String,
    pub detail: String,
    pub index: usize,
}

pub enum SearchAction {
    Nothing,
    Cancel,
    Confirm(usize),
}

pub struct SearchSelector {
    query: String,
    items: Vec<SearchItem>,
    filtered: Vec<usize>,
    cursor: usize,
    title: String,
}

fn fuzzy_match(query: &str, haystack: &str) -> bool {
    let mut haystack_chars = haystack.chars().flat_map(|c| c.to_lowercase());
    for qc in query.chars().flat_map(|c| c.to_lowercase()) {
        if haystack_chars.find(|&hc| hc == qc).is_none() {
            return false;
        }
    }
    true
}

impl SearchSelector {
    pub fn new(title: impl Into<String>, items: Vec<SearchItem>) -> Self {
        let filtered: Vec<usize> = (0..items.len()).collect();
        SearchSelector {
            query: String::new(),
            items,
            filtered,
            cursor: 0,
            title: title.into(),
        }
    }

    pub fn set_cursor_to_index(&mut self, original_index: usize) {
        if let Some(pos) = self.filtered.iter().position(|&i| self.items[i].index == original_index) {
            self.cursor = pos;
        }
    }

    fn refilter(&mut self) {
        if self.query.is_empty() {
            self.filtered = (0..self.items.len()).collect();
        } else {
            self.filtered = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    fuzzy_match(&self.query, &item.label)
                        || fuzzy_match(&self.query, &item.detail)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.cursor = 0;
    }

    pub fn handle_key(&mut self, key: KeyCode) -> SearchAction {
        match key {
            KeyCode::Esc => SearchAction::Cancel,
            KeyCode::Enter => {
                if let Some(&item_idx) = self.filtered.get(self.cursor) {
                    SearchAction::Confirm(self.items[item_idx].index)
                } else {
                    SearchAction::Nothing
                }
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.refilter();
                SearchAction::Nothing
            }
            KeyCode::Down => {
                if !self.filtered.is_empty() && self.cursor < self.filtered.len() - 1 {
                    self.cursor += 1;
                }
                SearchAction::Nothing
            }
            KeyCode::Up => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                SearchAction::Nothing
            }
            KeyCode::Char('j') if self.query.is_empty() => {
                if !self.filtered.is_empty() && self.cursor < self.filtered.len() - 1 {
                    self.cursor += 1;
                }
                SearchAction::Nothing
            }
            KeyCode::Char('k') if self.query.is_empty() => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                SearchAction::Nothing
            }
            KeyCode::Char(c) => {
                self.query.push(c);
                self.refilter();
                SearchAction::Nothing
            }
            _ => SearchAction::Nothing,
        }
    }

    pub fn selected_label(&self) -> Option<&str> {
        self.filtered
            .get(self.cursor)
            .map(|&i| self.items[i].label.as_str())
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.title));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(1)])
            .split(inner);

        // Search input
        let search_line = Line::from(vec![
            Span::styled(" > ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{}▏", self.query),
                Style::default().fg(Color::Yellow),
            ),
        ]);
        frame.render_widget(Paragraph::new(search_line), chunks[0]);

        // Separator
        let sep = Paragraph::new(Line::from(Span::styled(
            "─".repeat(inner.width as usize),
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(sep, chunks[1]);

        // Visible height for the list
        let list_height = chunks[2].height as usize;

        // Compute scroll offset to keep cursor visible
        let scroll_offset = if list_height == 0 {
            0
        } else if self.cursor >= list_height {
            self.cursor - list_height + 1
        } else {
            0
        };

        // Filtered items list
        let items: Vec<ListItem> = self
            .filtered
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(list_height)
            .map(|(fi, &item_idx)| {
                let item = &self.items[item_idx];
                let is_selected = fi == self.cursor;
                let prefix = if is_selected { " ► " } else { "   " };
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(Span::styled(
                    format!("{}{} ({})", prefix, item.label, item.detail),
                    style,
                )))
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, chunks[2]);
    }
}
