use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, Screen, Tab};
use crate::styles::BEER_STYLES;

pub fn draw(frame: &mut Frame, app: &App) {
    match &app.screen {
        Screen::RecipeList => draw_recipe_list(frame, app),
        Screen::RecipeEdit { .. } => draw_recipe_edit(frame, app),
    }
}

fn draw_recipe_list(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(frame.area());

    // Recipe list
    let items: Vec<ListItem> = app
        .recipes
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let prefix = if i == app.list_index { " ► " } else { "   " };
            let style = if i == app.list_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(
                format!("{}{}", prefix, r.name),
                style,
            )))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Brewdio "),
    );
    frame.render_widget(list, chunks[0]);

    // Help bar
    let help = Paragraph::new(Line::from(vec![
        Span::styled(" [n]", Style::default().fg(Color::Cyan)),
        Span::raw("ew  "),
        Span::styled("[d]", Style::default().fg(Color::Cyan)),
        Span::raw("elete  "),
        Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
        Span::raw(" open  "),
        Span::styled("[q]", Style::default().fg(Color::Cyan)),
        Span::raw("uit"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[1]);
}

fn draw_recipe_edit(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Name + Style
            Constraint::Length(3), // Tab bar
            Constraint::Min(3),   // Tab content
            Constraint::Length(3), // Help bar
        ])
        .split(frame.area());

    // Header: Name + Style
    draw_header(frame, app, chunks[0]);

    // Tab bar
    draw_tabs(frame, app, chunks[1]);

    // Tab content
    draw_tab_content(frame, app, chunks[2]);

    // Help bar
    let help_text = if app.editing_name {
        " Type to edit, [Enter] confirm, [Esc] cancel"
    } else if app.editing_style {
        " [↑/↓] select, [Enter] confirm, [Esc] cancel"
    } else {
        " [n]ame  [s]tyle  [1-5] tabs  [Esc] back"
    };
    let help = Paragraph::new(Line::from(Span::raw(help_text)))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[3]);
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Recipe ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    // Name row
    let name_style = if app.editing_name {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let name_display = if app.editing_name {
        format!("{}▏", app.name_input)
    } else {
        app.name_input.clone()
    };
    let name_line = Line::from(vec![
        Span::styled(" Name:  ", Style::default().fg(Color::DarkGray)),
        Span::styled(name_display, name_style),
    ]);
    frame.render_widget(Paragraph::new(name_line), rows[0]);

    // Style row
    if app.editing_style {
        let style_name = &BEER_STYLES[app.style_index].name;
        let style_line = Line::from(vec![
            Span::styled(" Style: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("< {} >", style_name),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(Paragraph::new(style_line), rows[1]);
    } else {
        let style_line = Line::from(vec![
            Span::styled(" Style: ", Style::default().fg(Color::DarkGray)),
            Span::raw(app.style_name()),
        ]);
        frame.render_widget(Paragraph::new(style_line), rows[1]);
    }
}

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = [
        Tab::Fermentables,
        Tab::Hops,
        Tab::Cultures,
        Tab::Water,
        Tab::Mash,
    ]
    .iter()
    .enumerate()
    .map(|(i, t)| Line::from(format!("[{}]{}", i + 1, t.label())))
    .collect();

    let tabs = Tabs::new(titles)
        .select(app.active_tab.index())
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::raw(" "));

    frame.render_widget(tabs, area);
}

fn draw_tab_content(frame: &mut Frame, app: &App, area: Rect) {
    if app.editing_style {
        draw_style_selector(frame, app, area);
        return;
    }

    let content = Paragraph::new(Line::from(Span::styled(
        "  (coming soon)",
        Style::default().fg(Color::DarkGray),
    )))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", app.active_tab.label())),
    );
    frame.render_widget(content, area);
}

fn draw_style_selector(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = BEER_STYLES
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let prefix = if i == app.style_index { " ► " } else { "   " };
            let style = if i == app.style_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(
                format!("{}{} ({})", prefix, s.name, s.category),
                style,
            )))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Select Style "),
    );
    frame.render_widget(list, area);
}
