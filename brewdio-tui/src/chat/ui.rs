use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::{ChatRole, ChatState};

/// Draw the chat panel. When `side_panel` is true, the chat occupies the right
/// half of the screen and gets a left border; otherwise it fills the whole screen.
pub fn draw_chat(frame: &mut Frame, chat: &ChatState, has_api_key: bool, side_panel: bool) {
    let area = if side_panel {
        // Right half of the screen
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());
        cols[1]
    } else {
        frame.area()
    };

    frame.render_widget(Clear, area);

    let outer_block = if side_panel {
        Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(Color::DarkGray))
    } else {
        Block::default()
    };
    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header
            Constraint::Min(3),   // Messages
            Constraint::Length(3), // Input
            Constraint::Length(1), // Help bar
        ])
        .split(inner);

    draw_header(frame, chat, chunks[0]);

    if !has_api_key {
        draw_no_api_key(frame, chunks[1]);
    } else {
        draw_messages(frame, chat, chunks[1]);
    }

    draw_input(frame, chat, chunks[2], has_api_key);
    draw_help_bar(frame, chat, chunks[3]);
}

fn draw_header(frame: &mut Frame, chat: &ChatState, area: Rect) {
    let mut title_spans = vec![Span::styled(
        " Brewing Assistant ",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )];

    if let Some((ref _id, ref recipe)) = chat.recipe_context {
        title_spans.push(Span::styled(
            format!("| {}", recipe.name),
            Style::default().fg(Color::Cyan),
        ));
    }

    let status = if chat.is_streaming {
        Span::styled(" streaming... ", Style::default().fg(Color::Green))
    } else {
        Span::raw("")
    };

    let title_line = Line::from(title_spans);
    let status_line = Line::from(vec![status]);

    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));

    let header_area = header_block.inner(area);
    frame.render_widget(header_block, area);

    if header_area.width > 0 && header_area.height > 0 {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Length(15)])
            .split(header_area);

        frame.render_widget(Paragraph::new(title_line), cols[0]);
        frame.render_widget(
            Paragraph::new(status_line).alignment(ratatui::layout::Alignment::Right),
            cols[1],
        );
    }
}

fn draw_no_api_key(frame: &mut Frame, area: Rect) {
    let msg = vec![
        Line::from(""),
        Line::from(Span::styled(
            " No AI API key configured",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(" To use the AI assistant:"),
        Line::from(""),
        Line::from("   1. Press Esc to close this panel"),
        Line::from("   2. Go to the Settings tab (press 3)"),
        Line::from("   3. Set your API key in \"AI API Key\""),
        Line::from("   4. Press c to open the assistant again"),
        Line::from(""),
        Line::from(Span::styled(
            " Works with OpenAI, Gemini, Groq, and other",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            " OpenAI-compatible providers. Set base URL",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            " and model in Settings to switch providers.",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(msg).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_messages(frame: &mut Frame, chat: &ChatState, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    for msg in &chat.messages {
        match msg.role {
            ChatRole::Assistant => {
                // Tool call indicators
                for tc in &msg.tool_calls {
                    let status_str = if tc.error.is_some() {
                        Span::styled(" error", Style::default().fg(Color::Red))
                    } else if tc.result.is_some() {
                        let summary = tc
                            .result
                            .as_ref()
                            .and_then(|r| {
                                if let Some(arr) = r.as_array() {
                                    Some(format!("{} results", arr.len()))
                                } else if r.get("success").and_then(|s| s.as_bool()) == Some(true) {
                                    Some("ok".to_string())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| "ok".to_string());
                        Span::styled(
                            format!(" {}", summary),
                            Style::default().fg(Color::Green),
                        )
                    } else {
                        Span::styled(" running...", Style::default().fg(Color::Yellow))
                    };

                    lines.push(Line::from(vec![
                        Span::styled("  [", Style::default().fg(Color::DarkGray)),
                        Span::styled(&tc.name, Style::default().fg(Color::Magenta)),
                        Span::styled("]", Style::default().fg(Color::DarkGray)),
                        Span::styled(" \u{2192}", Style::default().fg(Color::DarkGray)),
                        status_str,
                    ]));
                }

                // Text content
                if !msg.content.is_empty() {
                    if !msg.tool_calls.is_empty() {
                        lines.push(Line::from(""));
                    }

                    let is_error = msg.content.starts_with("Error:");
                    let content_lines: Vec<&str> = msg.content.split('\n').collect();
                    for (i, line) in content_lines.iter().enumerate() {
                        let text_span = if is_error {
                            Span::styled(line.to_string(), Style::default().fg(Color::Red))
                        } else {
                            render_text(line)
                        };
                        if i == 0 {
                            let prefix_color = if is_error { Color::Red } else { Color::Green };
                            lines.push(Line::from(vec![
                                Span::styled(
                                    "AI: ",
                                    Style::default()
                                        .fg(prefix_color)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                text_span,
                            ]));
                        } else {
                            lines.push(Line::from(vec![
                                Span::raw("    "),
                                text_span,
                            ]));
                        }
                    }
                }
            }
            ChatRole::User => {
                lines.push(Line::from(""));
                let content_lines: Vec<&str> = msg.content.split('\n').collect();
                for (i, line) in content_lines.iter().enumerate() {
                    if i == 0 {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "You: ",
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(*line),
                        ]));
                    } else {
                        lines.push(Line::from(vec![Span::raw("     "), Span::raw(*line)]));
                    }
                }
            }
            _ => {}
        }

        lines.push(Line::from(""));
    }

    // Streaming indicator when waiting for first token
    if chat.is_streaming {
        let last_is_empty_assistant = chat
            .messages
            .last()
            .map_or(true, |m| m.role != ChatRole::Assistant || m.content.is_empty());
        if last_is_empty_assistant {
            lines.push(Line::from(vec![
                Span::styled(
                    "AI: ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("\u{258c}", Style::default().fg(Color::Green)),
            ]));
        }
    }

    // Calculate scroll: auto-scroll to bottom unless user has scrolled up
    let visible_height = area.height as usize;
    let total_lines = lines.len();
    let bottom = total_lines.saturating_sub(visible_height);
    let scroll = if chat.user_scrolled {
        let clamped = chat.scroll_offset.min(bottom);
        if clamped >= bottom {
            // User scrolled back to the bottom — re-engage auto-scroll
            bottom
        } else {
            clamped
        }
    } else {
        bottom
    };

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
    frame.render_widget(paragraph, area);
}

fn draw_input(frame: &mut Frame, chat: &ChatState, area: Rect, has_api_key: bool) {
    let block = Block::default()
        .borders(Borders::TOP | Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Prompt on the left, editor on the right
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(inner);

    let prompt = Span::styled("> ", Style::default().fg(Color::Cyan));
    frame.render_widget(Paragraph::new(Line::from(prompt)), cols[0]);

    if !has_api_key {
        let placeholder = Paragraph::new(Line::from(Span::styled(
            "Set API key in Settings first",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(placeholder, cols[1]);
    } else if chat.is_streaming {
        let text = chat.input_text();
        let dimmed = Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(dimmed, cols[1]);
    } else {
        frame.render_widget(&chat.editor, cols[1]);
    }
}

fn draw_help_bar(frame: &mut Frame, chat: &ChatState, area: Rect) {
    let spans = if chat.is_streaming {
        vec![
            Span::raw(" "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" cancel  "),
            Span::styled("\u{2191}\u{2193}", Style::default().fg(Color::Cyan)),
            Span::raw(" scroll  "),
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" resize"),
        ]
    } else {
        vec![
            Span::raw(" "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(" send  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" close  "),
            Span::styled("\u{2191}\u{2193}", Style::default().fg(Color::Cyan)),
            Span::raw(" scroll  "),
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" resize"),
        ]
    };

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, area);
}

/// Basic inline formatting — returns plain text for now.
fn render_text(text: &str) -> Span<'static> {
    Span::raw(text.to_string())
}
