use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, BatchSizeDialogStep, CultureDialogStep, FermentableDialogStep, HopDialogStep, HomeTab, NotesTarget, Screen, SettingEditState, Tab, VitalDisplay, CULTURE_UNITS, MASS_UNITS, VOLUME_UNITS, USE_TYPES, format_hop_timing, use_type_label};
use brewdio_core::beerjson_types::{CultureAdditionTypeAmount, FermentableAdditionTypeAmount, HopAdditionTypeAmount, UseType};
use brewdio_persistence::settings::{SettingKind, SETTINGS_DESCRIPTORS};


/// Minimum width for the chat panel to appear as a side panel.
/// Below this, chat takes over the full screen.
const CHAT_MIN_WIDTH: u16 = 40;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if app.chat_visible {
    if let Some(ref chat) = app.chat {
        let has_api_key = !app.settings_doc.openai_api_key.is_empty();
        let chat_fits_side = area.width >= CHAT_MIN_WIDTH * 2;

        if chat_fits_side && !chat.fullscreen {
            // Side-by-side: left = normal screen, right = chat panel
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            match &app.screen {
                Screen::Home => draw_home_in(frame, app, cols[0]),
                Screen::RecipeEdit { .. } => draw_recipe_edit_in(frame, app, cols[0]),
                Screen::BatchEdit { .. } => draw_batch_edit_in(frame, app, cols[0]),
            }
            crate::chat::ui::draw_chat(frame, chat, has_api_key, true);
        } else {
            // Full screen chat
            crate::chat::ui::draw_chat(frame, chat, has_api_key, false);
        }
        return;
    }
    }

    match &app.screen {
        Screen::Home => draw_home(frame, app),
        Screen::RecipeEdit { .. } => draw_recipe_edit(frame, app),
        Screen::BatchEdit { .. } => draw_batch_edit(frame, app),
    }
}

fn help_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .title_bottom(Line::from(format!(" {} ", brewdio_core::VERSION)).alignment(Alignment::Right))
}

fn draw_home(frame: &mut Frame, app: &App) {
    draw_home_in(frame, app, frame.area());
}

fn draw_home_in(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Status bar
            Constraint::Min(3),   // Content
            Constraint::Length(3), // Help bar
        ])
        .split(area);

    // Status bar
    draw_status_bar(frame, app, chunks[0]);

    // Tab content
    match app.home_tab {
        HomeTab::Recipes => draw_recipes_tab(frame, app, chunks[1]),
        HomeTab::Batches => draw_batches_tab(frame, app, chunks[1]),
        HomeTab::Settings => draw_settings_tab(frame, app, chunks[1]),
    }

    // Help bar
    let help = match app.home_tab {
        HomeTab::Recipes => {
            let selected_is_deleted = app.recipes.get(app.list_index).map_or(false, |r| r.is_deleted);
            let mut spans = vec![Span::raw(" ")];
            if !app.show_deleted {
                spans.extend_from_slice(&[
                    Span::styled("[n]", Style::default().fg(Color::Cyan)),
                    Span::raw("ew  "),
                ]);
            }
            if selected_is_deleted {
                spans.extend_from_slice(&[
                    Span::styled("[u]", Style::default().fg(Color::Cyan)),
                    Span::raw("ndelete  "),
                ]);
            } else {
                spans.extend_from_slice(&[
                    Span::styled("[d]", Style::default().fg(Color::Cyan)),
                    Span::raw("elete  "),
                    Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
                    Span::raw(" open  "),
                ]);
            }
            spans.extend_from_slice(&[
                Span::styled("[r]", Style::default().fg(Color::Cyan)),
                Span::raw(if app.show_deleted { "ecipes  " } else { "ubbish  " }),
                Span::styled("[c]", Style::default().fg(Color::Cyan)),
                Span::raw("hat  "),
                Span::styled("[Tab]", Style::default().fg(Color::Cyan)),
                Span::raw(" next  "),
                Span::styled("[q]", Style::default().fg(Color::Cyan)),
                Span::raw("uit"),
            ]);
            Paragraph::new(Line::from(spans))
        }
        HomeTab::Batches => Paragraph::new(Line::from(vec![
            Span::styled(" [Enter]", Style::default().fg(Color::Cyan)),
            Span::raw(" open  "),
            Span::styled("[d]", Style::default().fg(Color::Cyan)),
            Span::raw("elete  "),
            Span::styled("[c]", Style::default().fg(Color::Cyan)),
            Span::raw("hat  "),
            Span::styled("[Tab]", Style::default().fg(Color::Cyan)),
            Span::raw(" next  "),
            Span::styled("[q]", Style::default().fg(Color::Cyan)),
            Span::raw("uit"),
        ])),
        HomeTab::Settings => {
            match &app.setting_edit {
                Some(SettingEditState::Selector { .. }) => {
                    let desc = SETTINGS_DESCRIPTORS.get(app.settings_index)
                        .map(|d| d.description)
                        .unwrap_or("");
                    Paragraph::new(Line::from(vec![
                        Span::styled(" [←/→]", Style::default().fg(Color::Cyan)),
                        Span::raw(" change  "),
                        Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
                        Span::raw(" confirm  "),
                        Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
                        Span::raw(" cancel  "),
                        Span::styled(desc, Style::default().fg(Color::DarkGray)),
                    ]))
                }
                Some(SettingEditState::TextInput { .. }) => {
                    let desc = SETTINGS_DESCRIPTORS.get(app.settings_index)
                        .map(|d| d.description)
                        .unwrap_or("");
                    Paragraph::new(Line::from(vec![
                        Span::raw(" Type to edit, "),
                        Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
                        Span::raw(" confirm, "),
                        Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
                        Span::raw(" cancel  "),
                        Span::styled(desc, Style::default().fg(Color::DarkGray)),
                    ]))
                }
                None => {
                    Paragraph::new(Line::from(vec![
                        Span::styled(" [Enter]", Style::default().fg(Color::Cyan)),
                        Span::raw(" edit  "),
                        Span::styled("[c]", Style::default().fg(Color::Cyan)),
                        Span::raw("hat  "),
                        Span::styled("[Tab]", Style::default().fg(Color::Cyan)),
                        Span::raw(" next  "),
                        Span::styled("[q]", Style::default().fg(Color::Cyan)),
                        Span::raw("uit"),
                    ]))
                }
            }
        }
    };
    frame.render_widget(help.block(help_block()), chunks[2]);
}

fn draw_recipes_tab(frame: &mut Frame, app: &App, area: Rect) {
    let title = if app.show_deleted { " Trash " } else { " Recipes " };
    let empty_msg = if app.show_deleted {
        "  No deleted recipes"
    } else {
        "  Press [n] to create a recipe"
    };

    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.recipes.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            empty_msg,
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(empty, inner);
    } else {
        let items: Vec<ListItem> = app
            .recipes
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let prefix = if i == app.list_index { " ► " } else { "   " };
                let item_style = if r.is_deleted {
                    if i == app.list_index {
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Rgb(120, 40, 40))
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(Color::Rgb(180, 80, 80))
                    }
                } else if i == app.list_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let total_width = inner.width as usize;
                // Use display columns (3) not byte length for padding calc
                let used_cols = 3 + r.name.len() + r.style.len();
                let padding = if total_width > used_cols { total_width - used_cols } else { 1 };

                if r.style.is_empty() {
                    ListItem::new(Line::from(vec![
                        Span::styled(prefix, item_style),
                        Span::styled(r.name.clone(), item_style),
                    ]))
                } else {
                    let dim_style = if r.is_deleted || i == app.list_index {
                        item_style
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(prefix, item_style),
                        Span::styled(r.name.clone(), item_style),
                        Span::styled(" ".repeat(padding), Style::default()),
                        Span::styled(r.style.clone(), dim_style),
                    ]))
                }
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, inner);
    }

    // Confirm-delete popup overlay
    if let Some((_, ref name)) = app.confirm_delete {
        draw_confirm_delete(frame, name, area);
    }
}

fn draw_confirm_delete(frame: &mut Frame, recipe_name: &str, area: Rect) {
    let popup_width = 44u16.min(area.width.saturating_sub(4));
    let popup_height = 5u16.min(area.height.saturating_sub(2));
    let popup_area = centered_rect(popup_width, popup_height, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Delete Recipe ");
    let inner = block.inner(popup_area);
    frame.render_widget(
        Paragraph::new("").style(Style::default().bg(Color::Black)),
        popup_area,
    );
    frame.render_widget(block, popup_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    // Truncate name if needed
    let max_name_len = (inner.width as usize).saturating_sub(11); // " Delete '" + "'?"
    let display_name = if recipe_name.len() > max_name_len {
        format!("{}...", &recipe_name[..max_name_len.saturating_sub(3)])
    } else {
        recipe_name.to_string()
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Delete '", Style::default().fg(Color::White)),
            Span::styled(display_name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("'?", Style::default().fg(Color::White)),
        ])),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" [y]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("es  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[n]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("o", Style::default().fg(Color::DarkGray)),
        ])),
        rows[1],
    );
}

fn draw_confirm_equipment(frame: &mut Frame, new_efficiency: f64, area: Rect) {
    let popup_width = 48u16.min(area.width.saturating_sub(4));
    let popup_height = 5u16.min(area.height.saturating_sub(2));
    let popup_area = centered_rect(popup_width, popup_height, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Change Equipment ");
    let inner = block.inner(popup_area);
    frame.render_widget(
        Paragraph::new("").style(Style::default().bg(Color::Black)),
        popup_area,
    );
    frame.render_widget(block, popup_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Overwrite efficiency to ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:.0}%", new_efficiency),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled("?", Style::default().fg(Color::White)),
        ])),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" [y]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("es  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[n]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("o", Style::default().fg(Color::DarkGray)),
        ])),
        rows[1],
    );
}

fn draw_batches_tab(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Batches ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.batches.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "  No batches yet",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(empty, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .batches
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let prefix = if i == app.batch_list_index {
                " ► "
            } else {
                "   "
            };
            let style = if i == app.batch_list_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let dim_style = if i == app.batch_list_index {
                style
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let recipe_part = format!("({})", b.recipe_name);
            let total_width = inner.width as usize;
            // prefix(3) + name + 2 spaces + recipe_part + brew_date
            let used = 3 + b.name.len() + 2 + recipe_part.len() + b.brew_date.len();
            let padding = if total_width > used { total_width - used } else { 1 };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(b.name.clone(), style),
                Span::styled("  ", Style::default()),
                Span::styled(recipe_part, dim_style),
                Span::styled(" ".repeat(padding), Style::default()),
                Span::styled(b.brew_date.clone(), dim_style),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn draw_settings_tab(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = SETTINGS_DESCRIPTORS
        .iter()
        .enumerate()
        .map(|(i, desc)| {
            let prefix = if i == app.settings_index {
                " ► "
            } else {
                "   "
            };
            let style = if i == app.settings_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let value_display = if i == app.settings_index {
                match &app.setting_edit {
                    Some(SettingEditState::Selector { options, index }) => {
                        format!("◄ {} ►", options[*index])
                    }
                    Some(SettingEditState::TextInput { input }) => {
                        format!("{}▏", input)
                    }
                    None => setting_display_value(&app.settings_doc, desc),
                }
            } else {
                setting_display_value(&app.settings_doc, desc)
            };

            ListItem::new(Line::from(Span::styled(
                format!("{}{:<24}{}", prefix, desc.key, value_display),
                style,
            )))
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(" Settings "));
    frame.render_widget(list, area);
}

fn setting_display_value(doc: &brewdio_persistence::settings::SettingsDocument, desc: &brewdio_persistence::settings::SettingDescriptor) -> String {
    let value = doc.get_value(desc.key);
    match desc.kind {
        SettingKind::Secret => {
            if value.is_empty() {
                "(not set)".to_string()
            } else {
                "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}".to_string()
            }
        }
        _ => value,
    }
}

fn draw_recipe_edit(frame: &mut Frame, app: &App) {
    draw_recipe_edit_in(frame, app, frame.area());
}

fn draw_recipe_edit_in(frame: &mut Frame, app: &App, area: Rect) {
    // Header has 4 rows (Name, Style, Batch, Notes) + 2 border = 6
    // Vitals: 5 vitals × 2 lines + 2 border = 12
    // Top row height is driven by vitals
    let top_height = 12;

    // Split vertically: status bar, top row (header beside vitals), tab content, help bar
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),          // Status bar
            Constraint::Length(top_height), // Header + Vitals side-by-side
            Constraint::Min(3),            // Tab content (full width)
            Constraint::Length(3),          // Help bar
        ])
        .split(area);

    // Status bar
    draw_status_bar(frame, app, outer[0]);

    // Split top row horizontally: left (header) | right (vitals)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(34)])
        .split(outer[1]);

    // Left side: header (Name + Style + Batch)
    draw_header(frame, app, cols[0]);

    // Right side: vitals panel
    draw_vitals_panel(frame, app, cols[1]);

    // Tab content (full width)
    draw_tab_content(frame, app, outer[2]);

    // Help bar
    let help_text = if app.confirm_equipment_idx.is_some() {
        " [y]es  [n]o"
    } else if app.editing_name {
        " Type to edit, [Enter] confirm, [Esc] cancel"
    } else if app.style_selector.is_some() || app.equipment_selector.is_some() {
        " Type to search, [↑/↓] navigate, [Enter] confirm, [Esc] cancel"
    } else if app.fermentable_dialog.is_some() {
        match app.fermentable_dialog.as_ref().unwrap().step {
            FermentableDialogStep::SelectFermentable => {
                " Type to search, [↑/↓] navigate, [Enter] confirm, [Esc] cancel"
            }
            FermentableDialogStep::EnterAmount => " Type amount, [Enter] confirm, [Esc] cancel",
            FermentableDialogStep::SelectUnit => {
                " [←/→] change unit, [Enter] confirm, [Esc] cancel"
            }
        }
    } else if app.batch_size_dialog.is_some() {
        match app.batch_size_dialog.as_ref().unwrap().step {
            BatchSizeDialogStep::EnterValue => " Type volume, [Enter] confirm, [Esc] cancel",
            BatchSizeDialogStep::SelectUnit => {
                " [←/→] change unit, [Enter] confirm, [Esc] cancel"
            }
        }
    } else if app.hop_dialog.is_some() {
        match app.hop_dialog.as_ref().unwrap().step {
            HopDialogStep::SelectHop => {
                " Type to search, [↑/↓] navigate, [Enter] confirm, [Esc] cancel"
            }
            HopDialogStep::EnterAmount => " Type amount, [Enter] confirm, [Esc] cancel",
            HopDialogStep::SelectUnit => {
                " [←/→] change unit, [Enter] confirm, [Esc] cancel"
            }
            HopDialogStep::SelectUse => {
                " [←/→] change use, [Enter] confirm, [Esc] cancel"
            }
            HopDialogStep::EnterTime => " Type time, [Enter] confirm, [Esc] cancel",
        }
    } else if app.culture_dialog.is_some() {
        match app.culture_dialog.as_ref().unwrap().step {
            CultureDialogStep::SelectCulture => {
                " Type to search, [↑/↓] navigate, [Enter] confirm, [Esc] cancel"
            }
            CultureDialogStep::EnterAmount => " Type amount, [Enter] confirm, [Esc] cancel",
            CultureDialogStep::SelectUnit => {
                " [←/→] change unit, [Enter] confirm, [Esc] cancel"
            }
        }
    } else if app.notes_editor.is_some() {
        " [F2] save  [Esc] cancel"
    } else if app.active_tab == Tab::History {
        " [j/k] navigate  [c]hat  [n]ame  [s]tyle  [e]quip  [v]ol  [b]rew  [o]notes  [1-7] tabs  [Esc] back"
    } else if app.active_tab == Tab::Batches {
        " [Enter] open  [j/k] navigate  [c]hat  [n]ame  [s]tyle  [e]quip  [v]ol  [b]rew  [o]notes  [1-7] tabs  [Esc] back"
    } else if app.active_tab == Tab::Fermentables || app.active_tab == Tab::Hops || app.active_tab == Tab::Cultures {
        " [a]dd  [Enter] edit  [d]elete  [j/k] navigate  [c]hat  [n]ame  [s]tyle  [e]quip  [v]ol  [b]rew  [o]notes  [1-7] tabs  [Esc] back"
    } else {
        " [c]hat  [n]ame  [s]tyle  [e]quip  [v]ol  [b]rew  [o]notes  [1-7] tabs  [Esc] back"
    };
    let help = Paragraph::new(Line::from(Span::raw(help_text)))
        .block(help_block());
    frame.render_widget(help, outer[3]);

    // Confirm equipment change popup overlay
    if let Some(idx) = app.confirm_equipment_idx {
        let all = brewdio_core::data::equipment();
        let new_eff = all[idx].efficiency.brewhouse.value;
        draw_confirm_equipment(frame, new_eff, area);
    }

    // Notes popup overlay
    if app.notes_editor.is_some() {
        draw_notes_popup(frame, app);
    }
}

fn draw_batch_edit(frame: &mut Frame, app: &App) {
    draw_batch_edit_in(frame, app, frame.area());
}

fn draw_batch_edit_in(frame: &mut Frame, app: &App, area: Rect) {
    let top_height = 12;

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(top_height),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(area);

    draw_status_bar(frame, app, outer[0]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(34)])
        .split(outer[1]);

    draw_header(frame, app, cols[0]);
    draw_vitals_panel(frame, app, cols[1]);
    draw_tab_content(frame, app, outer[2]);

    let help_text = if app.confirm_equipment_idx.is_some() {
        " [y]es  [n]o"
    } else if app.editing_name {
        " Type to edit, [Enter] confirm, [Esc] cancel"
    } else if app.editing_brew_date {
        " Type date (YYYY-MM-DD), [Enter] confirm, [Esc] cancel"
    } else if app.style_selector.is_some() || app.equipment_selector.is_some() {
        " Type to search, [↑/↓] navigate, [Enter] confirm, [Esc] cancel"
    } else if app.fermentable_dialog.is_some() {
        match app.fermentable_dialog.as_ref().unwrap().step {
            FermentableDialogStep::SelectFermentable => {
                " Type to search, [↑/↓] navigate, [Enter] confirm, [Esc] cancel"
            }
            FermentableDialogStep::EnterAmount => " Type amount, [Enter] confirm, [Esc] cancel",
            FermentableDialogStep::SelectUnit => {
                " [←/→] change unit, [Enter] confirm, [Esc] cancel"
            }
        }
    } else if app.batch_size_dialog.is_some() {
        match app.batch_size_dialog.as_ref().unwrap().step {
            BatchSizeDialogStep::EnterValue => " Type volume, [Enter] confirm, [Esc] cancel",
            BatchSizeDialogStep::SelectUnit => {
                " [←/→] change unit, [Enter] confirm, [Esc] cancel"
            }
        }
    } else if app.hop_dialog.is_some() {
        match app.hop_dialog.as_ref().unwrap().step {
            HopDialogStep::SelectHop => {
                " Type to search, [↑/↓] navigate, [Enter] confirm, [Esc] cancel"
            }
            HopDialogStep::EnterAmount => " Type amount, [Enter] confirm, [Esc] cancel",
            HopDialogStep::SelectUnit => {
                " [←/→] change unit, [Enter] confirm, [Esc] cancel"
            }
            HopDialogStep::SelectUse => {
                " [←/→] change use, [Enter] confirm, [Esc] cancel"
            }
            HopDialogStep::EnterTime => " Type time, [Enter] confirm, [Esc] cancel",
        }
    } else if app.culture_dialog.is_some() {
        match app.culture_dialog.as_ref().unwrap().step {
            CultureDialogStep::SelectCulture => {
                " Type to search, [↑/↓] navigate, [Enter] confirm, [Esc] cancel"
            }
            CultureDialogStep::EnterAmount => " Type amount, [Enter] confirm, [Esc] cancel",
            CultureDialogStep::SelectUnit => {
                " [←/→] change unit, [Enter] confirm, [Esc] cancel"
            }
        }
    } else if app.notes_editor.is_some() {
        " [F2] save  [Esc] cancel"
    } else if app.active_tab == Tab::History {
        " [j/k] navigate  [c]hat  [n]ame  [b]rew date  [e]quip  [r]ecipe  [o]notes  [1-6] tabs  [Esc] back"
    } else if app.active_tab == Tab::Fermentables || app.active_tab == Tab::Hops || app.active_tab == Tab::Cultures {
        " [a]dd  [Enter] edit  [d]elete  [j/k] navigate  [c]hat  [n]ame  [b]rew date  [e]quip  [r]ecipe  [o]notes  [1-6] tabs  [Esc] back"
    } else {
        " [c]hat  [n]ame  [b]rew date  [e]quip  [v]ol  [r]ecipe  [o]notes  [1-6] tabs  [Esc] back"
    };
    let help = Paragraph::new(Line::from(Span::raw(help_text)))
        .block(help_block());
    frame.render_widget(help, outer[3]);

    // Confirm equipment change popup overlay
    if let Some(idx) = app.confirm_equipment_idx {
        let all = brewdio_core::data::equipment();
        let new_eff = all[idx].efficiency.brewhouse.value;
        draw_confirm_equipment(frame, new_eff, area);
    }

    // Notes popup overlay
    if app.notes_editor.is_some() {
        draw_notes_popup(frame, app);
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let is_batch = matches!(app.screen, Screen::BatchEdit { .. });
    let title = if is_batch { " Batch " } else { " Recipe " };
    let block = Block::default().borders(Borders::ALL).title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // All rows are Length(1) except the last (notes) which can expand up to 5 lines.
    let fixed_rows = if is_batch { 6 } else { 5 };
    let mut constraints: Vec<Constraint> = (0..fixed_rows).map(|_| Constraint::Length(1)).collect();
    constraints.push(Constraint::Max(5)); // notes row
    constraints.push(Constraint::Min(0));

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let mut row_idx = 0;

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
    frame.render_widget(Paragraph::new(name_line), rows[row_idx]);
    row_idx += 1;

    // Style row
    if let Some(ref selector) = app.style_selector {
        let display = selector
            .selected_label()
            .unwrap_or("(no match)");
        let style_line = Line::from(vec![
            Span::styled(" Style: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("< {} >", display),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(Paragraph::new(style_line), rows[row_idx]);
    } else {
        let style_line = Line::from(vec![
            Span::styled(" Style: ", Style::default().fg(Color::DarkGray)),
            Span::raw(app.style_name()),
        ]);
        frame.render_widget(Paragraph::new(style_line), rows[row_idx]);
    }
    row_idx += 1;

    // Equipment row
    if let Some(ref selector) = app.equipment_selector {
        let display = selector
            .selected_label()
            .unwrap_or("(no match)");
        let equip_line = Line::from(vec![
            Span::styled(" Equip: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("< {} >", display),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(Paragraph::new(equip_line), rows[row_idx]);
    } else {
        let equip_line = Line::from(vec![
            Span::styled(" Equip: ", Style::default().fg(Color::DarkGray)),
            Span::raw(app.equipment_name()),
        ]);
        frame.render_widget(Paragraph::new(equip_line), rows[row_idx]);
    }
    row_idx += 1;

    // Efficiency row
    let eff_display = if let Some(ref doc) = app.current_doc {
        format!("{:.0}%", doc.recipe.efficiency.brewhouse.value)
    } else {
        "(none)".to_string()
    };
    let eff_line = Line::from(vec![
        Span::styled(" Effic: ", Style::default().fg(Color::DarkGray)),
        Span::raw(eff_display),
    ]);
    frame.render_widget(Paragraph::new(eff_line), rows[row_idx]);
    row_idx += 1;

    // Batch size row
    let mut batch_spans = vec![Span::styled(" Batch: ", Style::default().fg(Color::DarkGray))];
    if let Some(doc) = app.current_doc.as_ref() {
        let bs = &doc.recipe.batch_size;
        let unit_str = format!("{:?}", bs.unit).to_lowercase();
        batch_spans.extend(qty_amt(bs.value, &unit_str, Color::Reset));
    } else {
        batch_spans.push(Span::raw("(none)"));
    }
    frame.render_widget(Paragraph::new(Line::from(batch_spans)), rows[row_idx]);
    row_idx += 1;

    // Brew date row (batch edit only)
    if is_batch {
        let date_style = if app.editing_brew_date {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let date_display = if app.editing_brew_date {
            format!("{}▏", app.brew_date_input)
        } else {
            app.brew_date_input.clone()
        };
        let date_line = Line::from(vec![
            Span::styled(" Brew:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(date_display, date_style),
        ]);
        frame.render_widget(Paragraph::new(date_line), rows[row_idx]);
        row_idx += 1;
    }

    // Notes row (wraps up to 5 lines)
    let notes_raw = if is_batch {
        app.batch_notes_text.as_str()
    } else {
        app.current_doc
            .as_ref()
            .and_then(|d| d.recipe.notes.as_deref())
            .unwrap_or("")
    };
    let notes_area = rows[row_idx];
    if notes_raw.is_empty() {
        let notes_line = Line::from(vec![
            Span::styled(" Notes: ", Style::default().fg(Color::DarkGray)),
            Span::styled("(none)", Style::default().fg(Color::DarkGray)),
        ]);
        frame.render_widget(Paragraph::new(notes_line), notes_area);
    } else {
        let text = format!(" Notes: {}", notes_raw.replace('\n', " "));
        let paragraph = Paragraph::new(Span::styled(text, Style::default().fg(Color::DarkGray)))
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, notes_area);
    }

    // Overlay batch size dialog if active
    if app.batch_size_dialog.is_some() {
        draw_batch_size_dialog(frame, app, area);
    }
}

fn draw_batch_size_dialog(frame: &mut Frame, app: &App, area: Rect) {
    let dialog = match app.batch_size_dialog.as_ref() {
        Some(d) => d,
        None => return,
    };

    let popup_width = 44u16.min(area.width.saturating_sub(4));
    let popup_height = match dialog.step {
        BatchSizeDialogStep::EnterValue => 5,
        BatchSizeDialogStep::SelectUnit => 7,
    };
    let popup_height = popup_height.min(area.height.saturating_sub(2));
    let popup_area = centered_rect(popup_width, popup_height, area);

    let title = match dialog.step {
        BatchSizeDialogStep::EnterValue => " Batch Size ",
        BatchSizeDialogStep::SelectUnit => " Batch Unit ",
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(popup_area);
    frame.render_widget(
        Paragraph::new("").style(Style::default().bg(Color::Black)),
        popup_area,
    );
    frame.render_widget(block, popup_area);

    match dialog.step {
        BatchSizeDialogStep::EnterValue => {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(inner);

            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Volume: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{}▏", dialog.value_input),
                        Style::default().fg(Color::Yellow),
                    ),
                ])),
                rows[0],
            );
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    " [Enter] next  [Esc] cancel",
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[1],
            );
        }
        BatchSizeDialogStep::SelectUnit => {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(inner);

            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Volume: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(&dialog.value_input),
                ])),
                rows[0],
            );
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "─".repeat(inner.width as usize),
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[1],
            );

            let unit_labels: Vec<String> = VOLUME_UNITS
                .iter()
                .enumerate()
                .map(|(i, u)| {
                    let label = format!("{:?}", u).to_lowercase();
                    if i == dialog.unit_index {
                        format!("[{}]", label)
                    } else {
                        format!(" {} ", label)
                    }
                })
                .collect();

            let mut spans = vec![Span::styled(" Unit: ", Style::default().fg(Color::DarkGray))];
            spans.push(Span::styled("◄ ", Style::default().fg(Color::Cyan)));
            for (i, label) in unit_labels.iter().enumerate() {
                let style = if i == dialog.unit_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                spans.push(Span::styled(label.clone(), style));
                if i < unit_labels.len() - 1 {
                    spans.push(Span::raw("  "));
                }
            }
            spans.push(Span::styled(" ►", Style::default().fg(Color::Cyan)));

            frame.render_widget(Paragraph::new(Line::from(spans)), rows[2]);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    " [Enter] confirm  [Esc] cancel  [←/→] change",
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[3],
            );
        }
    }
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let bg = Color::Rgb(200, 200, 210);
    let bar_style = Style::default().bg(bg);
    let inactive_fg = Color::Rgb(80, 80, 95);
    let active_bg = Color::Rgb(50, 50, 60);
    let active_fg = Color::Rgb(230, 230, 240);
    let view_bg = Color::Rgb(120, 120, 140);
    let view_fg = Color::Rgb(240, 240, 250);

    let mut spans: Vec<Span> = vec![
        Span::styled(
            " brewdio ",
            Style::default()
                .bg(Color::Rgb(60, 60, 70))
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default()),
        ];

    match &app.screen {
        Screen::Home => {
            spans.push(Span::styled(
                " Home ",
                Style::default().bg(view_bg).fg(view_fg).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(" ", Style::default()));

            let home_tabs = [HomeTab::Recipes, HomeTab::Batches, HomeTab::Settings];
            for (i, tab) in home_tabs.iter().enumerate() {
                let is_active = *tab == app.home_tab;
                let label = format!(" [{}]{} ", i + 1, tab.label());
                if is_active {
                    spans.push(Span::styled(
                        label,
                        Style::default()
                            .bg(active_bg)
                            .fg(active_fg)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::styled(
                        label,
                        Style::default().bg(bg).fg(inactive_fg),
                    ));
                }
                spans.push(Span::styled(" ", bar_style));
            }
        }
        Screen::RecipeEdit { .. } => {
            spans.push(Span::styled(
                " Recipe ",
                Style::default().bg(view_bg).fg(view_fg).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(" ", Style::default()));

            let recipe_tabs = [
                Tab::Fermentables,
                Tab::Hops,
                Tab::Cultures,
                Tab::Water,
                Tab::Mash,
                Tab::Batches,
                Tab::History,
            ];
            for (i, tab) in recipe_tabs.iter().enumerate() {
                let is_active = *tab == app.active_tab;
                let label = format!(" [{}]{} ", i + 1, tab.label());
                if is_active {
                    spans.push(Span::styled(
                        label,
                        Style::default()
                            .bg(active_bg)
                            .fg(active_fg)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::styled(
                        label,
                        Style::default().bg(bg).fg(inactive_fg),
                    ));
                }
                spans.push(Span::styled(" ", bar_style));
            }
        }
        Screen::BatchEdit { .. } => {
            spans.push(Span::styled(
                " Batch ",
                Style::default().bg(view_bg).fg(view_fg).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(" ", Style::default()));

            let batch_tabs = [
                Tab::Fermentables,
                Tab::Hops,
                Tab::Cultures,
                Tab::Water,
                Tab::Mash,
                Tab::History,
            ];
            for (i, tab) in batch_tabs.iter().enumerate() {
                let is_active = *tab == app.active_tab;
                let label = format!(" [{}]{} ", i + 1, tab.label());
                if is_active {
                    spans.push(Span::styled(
                        label,
                        Style::default()
                            .bg(active_bg)
                            .fg(active_fg)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::styled(
                        label,
                        Style::default().bg(bg).fg(inactive_fg),
                    ));
                }
                spans.push(Span::styled(" ", bar_style));
            }
        }
    }

    // Sync status indicator (right-aligned)
    let sync_text = match &app.sync_connected {
        Some(flag) => {
            use brewdio_persistence::sync_worker::{SYNC_STATUS_CONNECTED, SYNC_STATUS_CLIENT_OUTDATED, SYNC_STATUS_SERVER_OUTDATED};
            match flag.load(std::sync::atomic::Ordering::Relaxed) {
                SYNC_STATUS_CONNECTED => Some((" Connected ", Color::Rgb(80, 180, 80), "\u{25CF} ")),
                SYNC_STATUS_CLIENT_OUTDATED => Some((" Incompatible - Update App ", Color::Rgb(220, 160, 40), "\u{26A0} ")),
                SYNC_STATUS_SERVER_OUTDATED => Some((" Incompatible - Update Server ", Color::Rgb(220, 160, 40), "\u{26A0} ")),
                _ => Some((" Disconnected ", Color::Rgb(180, 80, 80), "\u{25CB} ")),
            }
        }
        None => None,
    };

    let sync_width = sync_text.as_ref().map_or(0, |(t, _, icon)| t.len() + icon.len());
    let used: usize = spans.iter().map(|s| s.content.len()).sum();
    let remaining = (area.width as usize).saturating_sub(used + sync_width);
    if remaining > 0 {
        spans.push(Span::styled(" ".repeat(remaining), bar_style));
    }

    if let Some((label, color, icon)) = sync_text {
        spans.push(Span::styled(
            icon,
            Style::default().bg(bg).fg(color),
        ));
        spans.push(Span::styled(
            label,
            Style::default().bg(bg).fg(color).add_modifier(Modifier::BOLD),
        ));
    }

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_tab_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref selector) = app.style_selector {
        selector.draw(frame, area);
        return;
    }

    if let Some(ref selector) = app.equipment_selector {
        selector.draw(frame, area);
        return;
    }

    match app.active_tab {
        Tab::Fermentables => draw_fermentables_tab(frame, app, area),
        Tab::Hops => draw_hops_tab(frame, app, area),
        Tab::Cultures => draw_cultures_tab(frame, app, area),
        Tab::Water => draw_water_tab(frame, app, area),
        Tab::Batches => draw_recipe_batches_tab(frame, app, area),
        Tab::History => draw_history_tab(frame, app, area),
        _ => {
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
    }
}

fn draw_water_tab(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Water ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let result = match app.compute_water() {
        Some(r) => r,
        None => {
            let empty = Paragraph::new(Line::from(Span::styled(
                "  No recipe loaded",
                Style::default().fg(Color::DarkGray),
            )));
            frame.render_widget(empty, inner);
            return;
        }
    };

    // Compute beer color from SRM for bar styling
    let doc = app.current_doc.as_ref().unwrap();
    let recipe = &doc.recipe;
    let srm = brewdio_core::color::calculate_color(
        &recipe.ingredients.fermentable_additions,
        &recipe.batch_size,
    );
    let [r, g, b] = brewdio_core::olfarve::srm_to_srgb(srm, None);
    let beer_color = Color::Rgb(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    );
    let strike_color = Color::Rgb(100, 149, 237); // cornflower blue

    let vol_unit = &result.total_water_needed.unit;
    let vol_label = format!("{:?}", vol_unit).to_lowercase();

    // Layout: strike info (top) | bar chart (middle) | stage breakdown (bottom)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Strike/sparge info
            Constraint::Min(8),   // Bar chart
            Constraint::Length(7), // Stage breakdown
        ])
        .split(inner);

    // ── Strike & Sparge Info ──
    let strike = &result.strike_water;
    let temp_label = format!("{:?}", strike.temperature.unit);
    let temp_unit_str = format!("°{}", temp_label);

    let mut total_spans = vec![Span::styled(" Total Water  ", Style::default().fg(Color::DarkGray))];
    total_spans.extend(qty_f2(result.total_water_needed.value, &vol_label, strike_color));

    let mut strike_spans = vec![Span::styled(" Strike       ", Style::default().fg(Color::DarkGray))];
    strike_spans.extend(qty_f2(strike.volume.value, &vol_label, strike_color));
    strike_spans.push(Span::styled("  heat to ", Style::default().fg(Color::DarkGray)));
    strike_spans.extend(qty_spans(&format!("{:.1}", strike.temperature.value), &temp_unit_str, Color::White));

    let mut info_lines: Vec<Line> = vec![
        Line::from(total_spans),
        Line::from(strike_spans),
    ];

    if let Some(ref sparge) = result.sparge_water {
        let mut sparge_spans = vec![Span::styled(" Sparge       ", Style::default().fg(Color::DarkGray))];
        sparge_spans.extend(qty_f2(sparge.volume.value, &vol_label, strike_color));
        sparge_spans.push(Span::styled("  heat to ", Style::default().fg(Color::DarkGray)));
        sparge_spans.extend(qty_spans(&format!("{:.1}", sparge.temperature.value), &temp_unit_str, Color::White));
        info_lines.push(Line::from(sparge_spans));
    }

    let boil_time = recipe
        .boil
        .as_ref()
        .map(|b| b.boil_time.value as f64)
        .unwrap_or(60.0);

    let mut batch_spans = vec![Span::styled(" ", Style::default())];
    batch_spans.extend(qty_amt(recipe.batch_size.value, &format!("{:?}", recipe.batch_size.unit).to_lowercase(), Color::DarkGray));
    batch_spans.push(Span::styled(
        format!(" batch, {:.0} min boil", boil_time),
        Style::default().fg(Color::DarkGray),
    ));
    info_lines.push(Line::from(batch_spans));

    frame.render_widget(Paragraph::new(info_lines), chunks[0]);

    // ── Bar Chart: volumes at each stage ──
    let stages = &result.calculated_stages;
    let scale = 100u64;
    let num_bars = stages.len().max(1);

    let bars: Vec<Bar> = stages
        .iter()
        .map(|stage| {
            let vol = stage.volume_in;
            let label = match stage.id.as_str() {
                "source" => "Strike",
                "mash" => "Mash",
                "kettle" => "Kettle",
                "fermenter" => "Ferm",
                "packaging" => "Pkg",
                _ => &stage.label,
            };
            let color = if stage.is_source {
                strike_color
            } else {
                beer_color
            };
            Bar::default()
                .value((vol * scale as f64) as u64)
                .label(Line::from(label.to_string()))
                .text_value(format!("{:.2} {}", vol, vol_label))
                .style(Style::default().fg(color))
                .value_style(Style::default().fg(Color::White).bg(color))
        })
        .collect();

    // Calculate bar width to fill available space, with 1-char gaps
    let available = chunks[1].width as usize;
    let total_gaps = num_bars.saturating_sub(1);
    let bar_w = available
        .saturating_sub(total_gaps)
        .checked_div(num_bars)
        .unwrap_or(5)
        .max(3) as u16;

    // Center the chart horizontally
    let chart_width = (bar_w as usize * num_bars + total_gaps).min(available);
    let h_pad = (available.saturating_sub(chart_width)) / 2;
    let chart_area = Rect {
        x: chunks[1].x + h_pad as u16,
        y: chunks[1].y,
        width: chart_width as u16,
        height: chunks[1].height,
    };

    let chart = BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .bar_width(bar_w)
        .bar_gap(1);

    frame.render_widget(chart, chart_area);

    // ── Stage Breakdown ──
    let non_source_stages: Vec<_> = stages.iter().filter(|s| !s.is_source).collect();

    let stage_constraints: Vec<Constraint> = non_source_stages
        .iter()
        .map(|_| Constraint::Ratio(1, non_source_stages.len() as u32))
        .collect();

    let stage_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(stage_constraints)
        .split(chunks[2]);

    for (i, stage) in non_source_stages.iter().enumerate() {
        let mut in_spans = vec![Span::styled("  in  ", Style::default().fg(Color::DarkGray))];
        in_spans.extend(qty_f2(stage.volume_in, &vol_label, Color::White));

        let mut out_spans = vec![Span::styled("  out ", Style::default().fg(Color::DarkGray))];
        out_spans.extend(qty_f2(stage.volume_out, &vol_label, Color::White));

        let mut lines: Vec<Line> = vec![
            Line::from(Span::styled(
                format!(" {}", stage.label),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )),
            Line::from(in_spans),
            Line::from(out_spans),
        ];

        // Show individual losses
        // Losses are in gal internally; total_loss is already in output units.
        // Compute each loss's share of the converted total.
        let raw_total: f64 = stage.losses.iter().map(|l| match l.loss_type {
            brewdio_core::water::WaterLossType::Rate => l.value * (boil_time / 60.0),
            brewdio_core::water::WaterLossType::Flat => l.value,
        }).sum();
        for loss in &stage.losses {
            let raw_val = match loss.loss_type {
                brewdio_core::water::WaterLossType::Rate => loss.value * (boil_time / 60.0),
                brewdio_core::water::WaterLossType::Flat => loss.value,
            };
            let loss_display = if raw_total > 0.0 {
                (raw_val / raw_total) * stage.total_loss
            } else {
                0.0
            };
            let short_label = match loss.id.as_str() {
                "grainAbs" => "grain",
                "tunDead" => "tun",
                "boilOff" => "boil",
                "trub" => "trub",
                "trub_ferm" => "yeast",
                "lines" => "xfer",
                _ => &loss.label,
            };
            let mut loss_spans = vec![Span::styled("  ", Style::default())];
            loss_spans.extend(qty_spans(&format!("-{:.2}", loss_display), &vol_label, Color::Red));
            loss_spans.push(Span::styled(format!(" {}", short_label), Style::default().fg(Color::DarkGray)));
            lines.push(Line::from(loss_spans));
        }

        frame.render_widget(Paragraph::new(lines), stage_cols[i]);
    }
}

fn draw_recipe_batches_tab(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Batches ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.recipe_batches.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "  No batches yet. Press [b] to brew.",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(empty, inner);
    } else {
        let items: Vec<ListItem> = app
            .recipe_batches
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let prefix = if i == app.recipe_batch_list_index {
                    " ► "
                } else {
                    "   "
                };
                let style = if i == app.recipe_batch_list_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let dim_style = if i == app.recipe_batch_list_index {
                    style
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let total_width = inner.width as usize;
                // prefix(3) + name + brew_date
                let used = 3 + b.name.len() + b.brew_date.len();
                let padding = if total_width > used { total_width - used } else { 1 };

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(b.name.clone(), style),
                    Span::styled(" ".repeat(padding), Style::default()),
                    Span::styled(b.brew_date.clone(), dim_style),
                ]))
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, inner);
    }
}

fn draw_history_tab(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" History ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.history_loading {
        let loading = Paragraph::new(Line::from(Span::styled(
            "  Loading history...",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(loading, inner);
        return;
    }

    if app.history_entries.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "  No history available",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(empty, inner);
        return;
    }

    let visible_height = inner.height as usize;
    // Show most recent first
    let reversed: Vec<_> = app.history_entries.iter().rev().collect();
    let total = reversed.len();

    // Ensure scroll doesn't exceed bounds
    let scroll = app.history_scroll.min(total.saturating_sub(1));

    // Viewport: show entries starting at scroll position
    let end = (scroll + visible_height).min(total);
    let visible = &reversed[scroll..end];

    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i + scroll == app.history_scroll;

            let prefix = if is_selected { " \u{25ba} " } else { "   " };

            // Format timestamp: YYYY-MM-DD HH:MM:SS from epoch millis
            let ts = format_timestamp(entry.change.timestamp);

            // Actor ID: first 8 hex chars
            let actor = if entry.change.actor_id.len() >= 8 {
                &entry.change.actor_id[..8]
            } else {
                &entry.change.actor_id
            };

            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let dim_style = if is_selected {
                style
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(ts, dim_style),
                Span::styled("  ", Style::default()),
                Span::styled(actor.to_string(), Style::default().fg(Color::DarkGray)),
                Span::styled("  ", Style::default()),
                Span::styled(entry.summary.clone(), style),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn format_timestamp(epoch_millis: i64) -> String {
    if epoch_millis == 0 {
        return "                   ".to_string();
    }
    let total_secs = epoch_millis / 1000;
    let secs_of_day = ((total_secs % 86400) + 86400) % 86400;
    let hours = secs_of_day / 3600;
    let minutes = (secs_of_day % 3600) / 60;
    let seconds = secs_of_day % 60;

    let days_since_epoch = total_secs / 86400;
    let mut days = days_since_epoch;
    let mut year = 1970i64;
    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut month = 12;
    for (i, &md) in month_days.iter().enumerate() {
        if days < md {
            month = i + 1;
            break;
        }
        days -= md;
    }
    let day = days + 1;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

fn draw_fermentables_tab(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Fermentables ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let additions = match app.current_doc.as_ref() {
        Some(doc) => &doc.recipe.ingredients.fermentable_additions,
        None => return,
    };

    if additions.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "  Press [a] to add a fermentable",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(empty, inner);
    } else {
        let items: Vec<ListItem> = additions
            .iter()
            .enumerate()
            .map(|(i, addition)| {
                let is_selected = i == app.fermentable_list_index;

                // Color swatch
                let srm = brewdio_core::units::color_to_srm(&addition.color);
                let [r, g, b] = brewdio_core::olfarve::srm_to_srgb(srm, None);
                let swatch_color = Color::Rgb(
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8,
                );

                let prefix = if is_selected { " ► " } else { "   " };
                let (amount_val, amount_unit) = match &addition.amount {
                    FermentableAdditionTypeAmount::MassType(m) => {
                        (m.value, format!("{:?}", m.unit).to_lowercase())
                    }
                    FermentableAdditionTypeAmount::VolumeType(v) => {
                        (v.value, format!("{:?}", v.unit).to_lowercase())
                    }
                };
                let amount_val_str = format_amount(amount_val);
                let amount_str = format!("{} {}", amount_val_str, amount_unit);
                let name = &addition.name;

                // Calculate padding: total width minus prefix(3) - swatch(3) - name - amount
                let total_width = inner.width as usize;
                let used = 3 + 3 + name.len() + amount_str.len();
                let padding = if total_width > used {
                    total_width - used
                } else {
                    1
                };

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let val_color = if is_selected { Color::Yellow } else { Color::Reset };

                let mut spans = vec![
                    Span::styled(prefix, style),
                    Span::styled("██ ", Style::default().fg(swatch_color)),
                    Span::styled(name.clone(), style),
                    Span::styled(" ".repeat(padding), Style::default()),
                ];
                spans.extend(qty_amt(amount_val, &amount_unit, val_color));
                ListItem::new(Line::from(spans))
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, inner);
    }

    // Overlay dialog if active
    if app.fermentable_dialog.is_some() {
        draw_fermentable_dialog(frame, app, area);
    }
}

fn format_amount(value: f64) -> String {
    if value == value.floor() {
        format!("{:.0}", value)
    } else {
        // Trim trailing zeros
        let s = format!("{:.2}", value);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Render a quantity with unit as two spans: value bold, unit dim.
/// `value_str` is the formatted number, `unit_str` is the unit label.
/// Returns spans styled with `color` for the value and dimmed for the unit.
fn qty_spans(value_str: &str, unit_str: &str, color: Color) -> Vec<Span<'static>> {
    vec![
        Span::styled(
            value_str.to_string(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}", unit_str),
            Style::default().fg(Color::DarkGray),
        ),
    ]
}

/// Convenience: format a f64 value (2 decimal places) + unit as styled spans.
fn qty_f2(value: f64, unit_str: &str, color: Color) -> Vec<Span<'static>> {
    qty_spans(&format!("{:.2}", value), unit_str, color)
}

/// Format an amount value (smart decimal trimming) + unit as styled spans.
fn qty_amt(value: f64, unit_str: &str, color: Color) -> Vec<Span<'static>> {
    qty_spans(&format_amount(value), unit_str, color)
}

fn draw_fermentable_dialog(frame: &mut Frame, app: &App, area: Rect) {
    let dialog = match app.fermentable_dialog.as_ref() {
        Some(d) => d,
        None => return,
    };

    match dialog.step {
        FermentableDialogStep::SelectFermentable => {
            dialog.selector.draw(frame, area);
        }
        FermentableDialogStep::EnterAmount => {
            let all = brewdio_core::data::fermentables();
            let ferm_name = &all[dialog.selected_fermentable_index].name;

            let popup_width = 50u16.min(area.width.saturating_sub(4));
            let popup_height = 7u16.min(area.height.saturating_sub(2));
            let popup_area = centered_rect(popup_width, popup_height, area);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Enter Amount ");
            let inner = block.inner(popup_area);
            // Clear background
            frame.render_widget(
                Paragraph::new("").style(Style::default().bg(Color::Black)),
                popup_area,
            );
            frame.render_widget(block, popup_area);

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(inner);

            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Grain: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(ferm_name.clone()),
                ])),
                rows[0],
            );
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "─".repeat(inner.width as usize),
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[1],
            );
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Amount: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{}▏", dialog.amount_input),
                        Style::default().fg(Color::Yellow),
                    ),
                ])),
                rows[2],
            );
        }
        FermentableDialogStep::SelectUnit => {
            let all = brewdio_core::data::fermentables();
            let ferm_name = &all[dialog.selected_fermentable_index].name;

            let popup_width = 50u16.min(area.width.saturating_sub(4));
            let popup_height = 9u16.min(area.height.saturating_sub(2));
            let popup_area = centered_rect(popup_width, popup_height, area);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Select Unit ");
            let inner = block.inner(popup_area);
            frame.render_widget(
                Paragraph::new("").style(Style::default().bg(Color::Black)),
                popup_area,
            );
            frame.render_widget(block, popup_area);

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(inner);

            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Grain: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(ferm_name.clone()),
                ])),
                rows[0],
            );
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Amount: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(&dialog.amount_input),
                ])),
                rows[1],
            );
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "─".repeat(inner.width as usize),
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[2],
            );

            // Unit selector: < lb >
            let unit_labels: Vec<String> = MASS_UNITS
                .iter()
                .enumerate()
                .map(|(i, u)| {
                    let label = format!("{:?}", u).to_lowercase();
                    if i == dialog.unit_index {
                        format!("[{}]", label)
                    } else {
                        format!(" {} ", label)
                    }
                })
                .collect();

            let mut spans = vec![Span::styled(" Unit: ", Style::default().fg(Color::DarkGray))];
            spans.push(Span::styled("◄ ", Style::default().fg(Color::Cyan)));
            for (i, label) in unit_labels.iter().enumerate() {
                let style = if i == dialog.unit_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                spans.push(Span::styled(label.clone(), style));
                if i < unit_labels.len() - 1 {
                    spans.push(Span::raw("  "));
                }
            }
            spans.push(Span::styled(" ►", Style::default().fg(Color::Cyan)));

            frame.render_widget(Paragraph::new(Line::from(spans)), rows[3]);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    " [Enter] confirm  [Esc] cancel  [←/→] change unit",
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[4],
            );
        }
    }
}

fn draw_hops_tab(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Hops ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let additions = match app.current_doc.as_ref() {
        Some(doc) => &doc.recipe.ingredients.hop_additions,
        None => return,
    };

    if additions.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "  Press [a] to add a hop",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(empty, inner);
    } else {
        // Compute column widths for table-like alignment
        let max_name_len = additions.iter().map(|a| a.name.len()).max().unwrap_or(0);
        let max_timing_len = additions
            .iter()
            .map(|a| format_hop_timing(&a.timing).len())
            .max()
            .unwrap_or(0);

        let items: Vec<ListItem> = additions
            .iter()
            .enumerate()
            .map(|(i, addition)| {
                let is_selected = i == app.hop_list_index;

                let prefix = if is_selected { " ► " } else { "   " };
                let (amount_val, amount_unit) = match &addition.amount {
                    HopAdditionTypeAmount::MassType(m) => {
                        (m.value, format!("{:?}", m.unit).to_lowercase())
                    }
                    HopAdditionTypeAmount::VolumeType(v) => {
                        (v.value, format!("{:?}", v.unit).to_lowercase())
                    }
                };
                let amount_str = format!("{} {}", format_amount(amount_val), amount_unit);
                let timing_str = format_hop_timing(&addition.timing);
                let name = &addition.name;

                // Pad name and timing to fixed column widths
                let name_pad = max_name_len - name.len() + 2;
                let timing_pad = max_timing_len - timing_str.len();

                // Remaining space goes between timing and amount (right-aligned)
                let fixed = 3 + max_name_len + 2 + max_timing_len + amount_str.len();
                let total_width = inner.width as usize;
                let trailing = total_width.saturating_sub(fixed).max(1);

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let val_color = if is_selected { Color::Yellow } else { Color::Reset };

                let mut spans = vec![
                    Span::styled(prefix, style),
                    Span::styled(name.clone(), style),
                    Span::styled(" ".repeat(name_pad), Style::default()),
                    Span::styled(
                        timing_str,
                        if is_selected {
                            style
                        } else {
                            Style::default().fg(Color::DarkGray)
                        },
                    ),
                    Span::styled(" ".repeat(timing_pad + trailing), Style::default()),
                ];
                spans.extend(qty_amt(amount_val, &amount_unit, val_color));
                ListItem::new(Line::from(spans))
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, inner);
    }

    // Overlay dialog if active
    if app.hop_dialog.is_some() {
        draw_hop_dialog(frame, app, area);
    }
}

fn draw_hop_dialog(frame: &mut Frame, app: &App, area: Rect) {
    let dialog = match app.hop_dialog.as_ref() {
        Some(d) => d,
        None => return,
    };

    match dialog.step {
        HopDialogStep::SelectHop => {
            dialog.selector.draw(frame, area);
        }
        HopDialogStep::EnterAmount => {
            let all = brewdio_core::data::hops();
            let hop_name = &all[dialog.selected_hop_index].name;

            let popup_width = 50u16.min(area.width.saturating_sub(4));
            let popup_height = 7u16.min(area.height.saturating_sub(2));
            let popup_area = centered_rect(popup_width, popup_height, area);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Enter Amount ");
            let inner = block.inner(popup_area);
            frame.render_widget(
                Paragraph::new("").style(Style::default().bg(Color::Black)),
                popup_area,
            );
            frame.render_widget(block, popup_area);

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(inner);

            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Hop: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(hop_name.clone()),
                ])),
                rows[0],
            );
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "─".repeat(inner.width as usize),
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[1],
            );
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Amount: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{}▏", dialog.amount_input),
                        Style::default().fg(Color::Yellow),
                    ),
                ])),
                rows[2],
            );
        }
        HopDialogStep::SelectUnit => {
            let all = brewdio_core::data::hops();
            let hop_name = &all[dialog.selected_hop_index].name;

            let popup_width = 50u16.min(area.width.saturating_sub(4));
            let popup_height = 9u16.min(area.height.saturating_sub(2));
            let popup_area = centered_rect(popup_width, popup_height, area);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Select Unit ");
            let inner = block.inner(popup_area);
            frame.render_widget(
                Paragraph::new("").style(Style::default().bg(Color::Black)),
                popup_area,
            );
            frame.render_widget(block, popup_area);

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(inner);

            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Hop: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(hop_name.clone()),
                ])),
                rows[0],
            );
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Amount: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(&dialog.amount_input),
                ])),
                rows[1],
            );
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "─".repeat(inner.width as usize),
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[2],
            );

            let unit_labels: Vec<String> = MASS_UNITS
                .iter()
                .enumerate()
                .map(|(i, u)| {
                    let label = format!("{:?}", u).to_lowercase();
                    if i == dialog.unit_index {
                        format!("[{}]", label)
                    } else {
                        format!(" {} ", label)
                    }
                })
                .collect();

            let mut spans = vec![Span::styled(" Unit: ", Style::default().fg(Color::DarkGray))];
            spans.push(Span::styled("◄ ", Style::default().fg(Color::Cyan)));
            for (i, label) in unit_labels.iter().enumerate() {
                let style = if i == dialog.unit_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                spans.push(Span::styled(label.clone(), style));
                if i < unit_labels.len() - 1 {
                    spans.push(Span::raw("  "));
                }
            }
            spans.push(Span::styled(" ►", Style::default().fg(Color::Cyan)));

            frame.render_widget(Paragraph::new(Line::from(spans)), rows[3]);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    " [Enter] next  [Esc] cancel  [←/→] change unit",
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[4],
            );
        }
        HopDialogStep::SelectUse => {
            let all = brewdio_core::data::hops();
            let hop_name = &all[dialog.selected_hop_index].name;
            let unit_str = format!("{:?}", MASS_UNITS[dialog.unit_index]).to_lowercase();

            let popup_width = 50u16.min(area.width.saturating_sub(4));
            let popup_height = 11u16.min(area.height.saturating_sub(2));
            let popup_area = centered_rect(popup_width, popup_height, area);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Select Use ");
            let inner = block.inner(popup_area);
            frame.render_widget(
                Paragraph::new("").style(Style::default().bg(Color::Black)),
                popup_area,
            );
            frame.render_widget(block, popup_area);

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(inner);

            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Hop: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(hop_name.clone()),
                ])),
                rows[0],
            );
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Amount: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("{} {}", &dialog.amount_input, unit_str)),
                ])),
                rows[1],
            );
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "─".repeat(inner.width as usize),
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[2],
            );

            let use_labels: Vec<String> = USE_TYPES
                .iter()
                .enumerate()
                .map(|(i, u)| {
                    let label = use_type_label(u);
                    if i == dialog.use_index {
                        format!("[{}]", label)
                    } else {
                        format!(" {} ", label)
                    }
                })
                .collect();

            let mut spans = vec![Span::styled(" Use: ", Style::default().fg(Color::DarkGray))];
            spans.push(Span::styled("◄ ", Style::default().fg(Color::Cyan)));
            for (i, label) in use_labels.iter().enumerate() {
                let style = if i == dialog.use_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                spans.push(Span::styled(label.clone(), style));
                if i < use_labels.len() - 1 {
                    spans.push(Span::raw("  "));
                }
            }
            spans.push(Span::styled(" ►", Style::default().fg(Color::Cyan)));

            frame.render_widget(Paragraph::new(Line::from(spans)), rows[3]);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    " [Enter] next  [Esc] cancel  [←/→] change use",
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[4],
            );
        }
        HopDialogStep::EnterTime => {
            let all = brewdio_core::data::hops();
            let hop_name = &all[dialog.selected_hop_index].name;
            let unit_str = format!("{:?}", MASS_UNITS[dialog.unit_index]).to_lowercase();
            let use_label = use_type_label(&USE_TYPES[dialog.use_index]);
            let time_unit = match USE_TYPES[dialog.use_index] {
                UseType::AddToFermentation => "days",
                _ => "min",
            };

            let popup_width = 50u16.min(area.width.saturating_sub(4));
            let popup_height = 11u16.min(area.height.saturating_sub(2));
            let popup_area = centered_rect(popup_width, popup_height, area);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Enter Time ");
            let inner = block.inner(popup_area);
            frame.render_widget(
                Paragraph::new("").style(Style::default().bg(Color::Black)),
                popup_area,
            );
            frame.render_widget(block, popup_area);

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(inner);

            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Hop: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(hop_name.clone()),
                ])),
                rows[0],
            );
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Amount: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(format!("{} {}", &dialog.amount_input, unit_str)),
                ])),
                rows[1],
            );
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Use: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(use_label),
                ])),
                rows[2],
            );
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "─".repeat(inner.width as usize),
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[3],
            );
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(
                        format!(" Time ({}): ", time_unit),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("{}▏", dialog.time_input),
                        Style::default().fg(Color::Yellow),
                    ),
                ])),
                rows[4],
            );
        }
    }
}

fn draw_cultures_tab(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Cultures ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let additions = match app.current_doc.as_ref() {
        Some(doc) => &doc.recipe.ingredients.culture_additions,
        None => return,
    };

    if additions.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "  Press [a] to add a culture",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(empty, inner);
    } else {
        let items: Vec<ListItem> = additions
            .iter()
            .enumerate()
            .map(|(i, addition)| {
                let is_selected = i == app.culture_list_index;

                let prefix = if is_selected { " ► " } else { "   " };
                let (amount_val, amount_unit_str) = match &addition.amount {
                    CultureAdditionTypeAmount::UnitType(u) => {
                        (u.value, format!("{:?}", u.unit).to_lowercase())
                    }
                    CultureAdditionTypeAmount::MassType(m) => {
                        (m.value, format!("{:?}", m.unit).to_lowercase())
                    }
                    CultureAdditionTypeAmount::VolumeType(v) => {
                        (v.value, format!("{:?}", v.unit).to_lowercase())
                    }
                };
                let amount_str = format!("{} {}", format_amount(amount_val), amount_unit_str);
                let name = &addition.name;

                let total_width = inner.width as usize;
                let used = 3 + name.len() + amount_str.len();
                let padding = if total_width > used {
                    total_width - used
                } else {
                    1
                };

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let val_color = if is_selected { Color::Yellow } else { Color::Reset };

                let mut spans = vec![
                    Span::styled(prefix, style),
                    Span::styled(name.clone(), style),
                    Span::styled(" ".repeat(padding), Style::default()),
                ];
                spans.extend(qty_amt(amount_val, &amount_unit_str, val_color));
                ListItem::new(Line::from(spans))
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, inner);
    }

    // Overlay dialog if active
    if app.culture_dialog.is_some() {
        draw_culture_dialog(frame, app, area);
    }
}

fn draw_culture_dialog(frame: &mut Frame, app: &App, area: Rect) {
    let dialog = match app.culture_dialog.as_ref() {
        Some(d) => d,
        None => return,
    };

    match dialog.step {
        CultureDialogStep::SelectCulture => {
            dialog.selector.draw(frame, area);
        }
        CultureDialogStep::EnterAmount => {
            let all = brewdio_core::data::cultures();
            let culture_name = &all[dialog.selected_culture_index].name;

            let popup_width = 50u16.min(area.width.saturating_sub(4));
            let popup_height = 7u16.min(area.height.saturating_sub(2));
            let popup_area = centered_rect(popup_width, popup_height, area);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Enter Amount ");
            let inner = block.inner(popup_area);
            frame.render_widget(
                Paragraph::new("").style(Style::default().bg(Color::Black)),
                popup_area,
            );
            frame.render_widget(block, popup_area);

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(inner);

            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Culture: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(culture_name.clone()),
                ])),
                rows[0],
            );
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "─".repeat(inner.width as usize),
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[1],
            );
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Amount: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{}▏", dialog.amount_input),
                        Style::default().fg(Color::Yellow),
                    ),
                ])),
                rows[2],
            );
        }
        CultureDialogStep::SelectUnit => {
            let all = brewdio_core::data::cultures();
            let culture_name = &all[dialog.selected_culture_index].name;

            let popup_width = 50u16.min(area.width.saturating_sub(4));
            let popup_height = 9u16.min(area.height.saturating_sub(2));
            let popup_area = centered_rect(popup_width, popup_height, area);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Select Unit ");
            let inner = block.inner(popup_area);
            frame.render_widget(
                Paragraph::new("").style(Style::default().bg(Color::Black)),
                popup_area,
            );
            frame.render_widget(block, popup_area);

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(inner);

            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Culture: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(culture_name.clone()),
                ])),
                rows[0],
            );
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(" Amount: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(&dialog.amount_input),
                ])),
                rows[1],
            );
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "─".repeat(inner.width as usize),
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[2],
            );

            let unit_labels: Vec<String> = CULTURE_UNITS
                .iter()
                .enumerate()
                .map(|(i, u)| {
                    let label = format!("{:?}", u).to_lowercase();
                    if i == dialog.unit_index {
                        format!("[{}]", label)
                    } else {
                        format!(" {} ", label)
                    }
                })
                .collect();

            let mut spans = vec![Span::styled(" Unit: ", Style::default().fg(Color::DarkGray))];
            spans.push(Span::styled("◄ ", Style::default().fg(Color::Cyan)));
            for (i, label) in unit_labels.iter().enumerate() {
                let style = if i == dialog.unit_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                spans.push(Span::styled(label.clone(), style));
                if i < unit_labels.len() - 1 {
                    spans.push(Span::raw("  "));
                }
            }
            spans.push(Span::styled(" ►", Style::default().fg(Color::Cyan)));

            frame.render_widget(Paragraph::new(Line::from(spans)), rows[3]);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    " [Enter] confirm  [Esc] cancel  [←/→] change unit",
                    Style::default().fg(Color::DarkGray),
                ))),
                rows[4],
            );
        }
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

fn draw_vitals_panel(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Vitals ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let vitals = app.compute_vitals();
    if vitals.is_empty() {
        return;
    }

    // Each vital uses exactly 2 lines (label + bar), no spacing between them
    let mut constraints: Vec<Constraint> = vitals
        .iter()
        .flat_map(|_| [Constraint::Length(1), Constraint::Length(1)])
        .collect();
    constraints.push(Constraint::Min(0));

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, vital) in vitals.iter().enumerate() {
        let row_base = i * 2;

        // Line 1: centered label with dashes filling full width
        let label_text = format!(" {} ", vital.label);
        let width = inner.width as usize;
        let pad_total = width.saturating_sub(label_text.len());
        let pad_left = pad_total / 2;
        let pad_right = pad_total - pad_left;
        let label = format!(
            "{}{}{}",
            "─".repeat(pad_left),
            label_text,
            "─".repeat(pad_right),
        );
        let label_line = Line::from(Span::styled(
            label,
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(Paragraph::new(label_line), rows[row_base]);

        // Line 2: value + range bar
        let bar_width = inner.width.saturating_sub(7); // reserve space for value + space
        let bar_line = draw_range_bar(vital, bar_width);
        frame.render_widget(Paragraph::new(bar_line), rows[row_base + 1]);
    }
}

fn draw_notes_popup(frame: &mut Frame, app: &App) {
    let editor = match app.notes_editor.as_ref() {
        Some(e) => e,
        None => return,
    };

    let area = frame.area();
    let popup_width = (area.width * 3 / 4).max(40).min(area.width.saturating_sub(4));
    let popup_height = (area.height * 3 / 4).max(10).min(area.height.saturating_sub(2));
    let popup_area = centered_rect(popup_width, popup_height, area);

    // Clear the area behind the popup so it's fully opaque
    frame.render_widget(Clear, popup_area);

    let title = match app.notes_target {
        NotesTarget::Recipe => " Recipe Notes ",
        NotesTarget::Batch => " Batch Notes ",
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::Black));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Split inner: editor area + hint bar
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    frame.render_widget(editor, rows[0]);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" [F2]", Style::default().fg(Color::Cyan)),
            Span::raw(" save  "),
            Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
            Span::raw(" cancel"),
        ])),
        rows[1],
    );
}

fn draw_range_bar(vital: &VitalDisplay, bar_width: u16) -> Line<'static> {
    let bar_width = bar_width as usize;
    if bar_width == 0 {
        return Line::from(Span::raw(vital.formatted.clone()));
    }

    // Determine value text color
    let value_color = match &vital.style_range {
        Some(range) => {
            if vital.value >= range.min && vital.value <= range.max {
                Color::Green
            } else {
                Color::Red
            }
        }
        None => Color::Reset,
    };

    // Pad formatted value to 6 chars
    let value_text = format!("{:<6}", vital.formatted);
    let mut spans: Vec<Span> = vec![Span::styled(value_text, Style::default().fg(value_color))];

    // Build bar characters
    let range = vital.normal_max - vital.normal_min;
    if range <= 0.0 {
        return Line::from(spans);
    }

    // Find the column closest to the current value
    let value_frac = (vital.value - vital.normal_min) / range;
    let value_col = (value_frac * (bar_width as f64 - 1.0))
        .round()
        .clamp(0.0, bar_width as f64 - 1.0) as usize;

    let is_srm = vital.label == "SRM";

    for col in 0..bar_width {
        let col_value = vital.normal_min + (col as f64 / (bar_width as f64 - 1.0).max(1.0)) * range;

        if col == value_col {
            spans.push(Span::styled(
                "│",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
        } else if let Some(ref sr) = vital.style_range {
            if col_value >= sr.min && col_value <= sr.max {
                if is_srm {
                    let [r, g, b] = brewdio_core::olfarve::srm_to_srgb(col_value, None);
                    let srm_color = Color::Rgb(
                        (r * 255.0) as u8,
                        (g * 255.0) as u8,
                        (b * 255.0) as u8,
                    );
                    spans.push(Span::styled("▓", Style::default().fg(srm_color)));
                } else {
                    spans.push(Span::styled("▓", Style::default().fg(Color::Cyan)));
                }
            } else {
                spans.push(Span::styled("░", Style::default().fg(Color::DarkGray)));
            }
        } else {
            spans.push(Span::styled("░", Style::default().fg(Color::DarkGray)));
        }
    }

    Line::from(spans)
}
