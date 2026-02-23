mod app;
mod search_selector;
mod styles;
mod ui;

use std::io;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, BatchSizeDialogStep, CultureDialogStep, FermentableDialogStep, HopDialogStep, HomeTab, Screen, Tab, CULTURE_UNITS, MASS_UNITS, VOLUME_UNITS, USE_TYPES};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Resolve DB path
    let proj_dirs = directories::ProjectDirs::from("com", "brewdio", "brewdio")
        .expect("Could not determine data directory");
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;
    let db_path = data_dir.join("brewdio.db");

    let conn = brewdio_persistence::connection_native::open(db_path.to_str().unwrap())?;
    let conn = Arc::new(Mutex::new(conn));

    // Start background sync worker if BREWDIO_SERVER_URL is set
    let sync_connected = Arc::new(AtomicBool::new(false));
    let _runtime = if let Ok(url) = std::env::var("BREWDIO_SERVER_URL") {
        let rt = tokio::runtime::Runtime::new()?;
        let conn_clone = conn.clone();
        let connected = sync_connected.clone();
        rt.spawn(async move {
            let _ = brewdio_persistence::sync_worker::spawn_sync(conn_clone, url, connected).await;
        });
        Some(rt)
    } else {
        None
    };

    let mut app = App::new(conn);
    if _runtime.is_some() {
        app.sync_connected = Some(sync_connected);
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    let result = run_loop(&mut terminal, &mut app);

    // Teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match &app.screen {
                    Screen::Home => handle_home_input(app, key.code),
                    Screen::RecipeEdit { .. } => handle_edit_input(app, key.code),
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_home_input(app: &mut App, key: KeyCode) {
    // Tab switching (all tabs)
    match key {
        KeyCode::Tab => {
            app.home_tab = app.home_tab.next();
            return;
        }
        KeyCode::BackTab => {
            app.home_tab = app.home_tab.prev();
            return;
        }
        KeyCode::Char('1') => {
            app.home_tab = HomeTab::Recipes;
            return;
        }
        KeyCode::Char('2') => {
            app.home_tab = HomeTab::Batches;
            return;
        }
        KeyCode::Char('3') => {
            app.home_tab = HomeTab::Settings;
            return;
        }
        KeyCode::Char('q') if !app.editing_setting => {
            app.should_quit = true;
            return;
        }
        _ => {}
    }

    match app.home_tab {
        HomeTab::Recipes => handle_recipes_tab_input(app, key),
        HomeTab::Batches => handle_batches_tab_input(app, key),
        HomeTab::Settings => handle_settings_tab_input(app, key),
    }
}

fn handle_recipes_tab_input(app: &mut App, key: KeyCode) {
    // Handle confirm-delete popup first
    if app.confirm_delete.is_some() {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_delete_yes(),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.confirm_delete_no(),
            _ => {}
        }
        return;
    }

    let selected_is_deleted = app.recipes.get(app.list_index).map_or(false, |r| r.is_deleted);

    match key {
        KeyCode::Char('n') if !app.show_deleted => app.create_recipe(),
        KeyCode::Char('d') if !selected_is_deleted => app.prompt_delete_selected(),
        KeyCode::Char('u') if selected_is_deleted => app.undelete_selected(),
        KeyCode::Char('r') => app.toggle_show_deleted(),
        KeyCode::Enter if !selected_is_deleted => app.open_selected(),
        KeyCode::Char('j') | KeyCode::Down => {
            if !app.recipes.is_empty() && app.list_index < app.recipes.len() - 1 {
                app.list_index += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.list_index > 0 {
                app.list_index -= 1;
            }
        }
        _ => {}
    }
}

fn handle_batches_tab_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('d') => app.delete_selected_batch(),
        KeyCode::Char('j') | KeyCode::Down => {
            if !app.batches.is_empty() && app.batch_list_index < app.batches.len() - 1 {
                app.batch_list_index += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.batch_list_index > 0 {
                app.batch_list_index -= 1;
            }
        }
        _ => {}
    }
}

fn handle_settings_tab_input(app: &mut App, key: KeyCode) {
    if app.editing_setting {
        match key {
            KeyCode::Enter => {
                // Confirm edit
                if app.settings_index < app.settings_entries.len() {
                    app.settings_entries[app.settings_index].value = app.setting_input.clone();
                    app.save_setting_value();
                }
                app.editing_setting = false;
            }
            KeyCode::Esc => {
                app.editing_setting = false;
            }
            KeyCode::Backspace => {
                app.setting_input.pop();
            }
            KeyCode::Char(c) => {
                app.setting_input.push(c);
            }
            _ => {}
        }
        return;
    }

    match key {
        KeyCode::Enter => {
            if app.settings_index < app.settings_entries.len() {
                app.setting_input = app.settings_entries[app.settings_index].value.clone();
                app.editing_setting = true;
            }
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if !app.settings_entries.is_empty()
                && app.settings_index < app.settings_entries.len() - 1
            {
                app.settings_index += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.settings_index > 0 {
                app.settings_index -= 1;
            }
        }
        _ => {}
    }
}

fn handle_edit_input(app: &mut App, key: KeyCode) {
    if app.editing_name {
        match key {
            KeyCode::Enter => app.confirm_name(),
            KeyCode::Esc => app.cancel_name(),
            KeyCode::Backspace => {
                app.name_input.pop();
            }
            KeyCode::Char(c) => {
                app.name_input.push(c);
            }
            _ => {}
        }
        return;
    }

    if let Some(ref mut selector) = app.style_selector {
        match selector.handle_key(key) {
            search_selector::SearchAction::Confirm(idx) => app.confirm_style(idx),
            search_selector::SearchAction::Cancel => app.cancel_style(),
            search_selector::SearchAction::Nothing => {}
        }
        return;
    }

    // Fermentable dialog
    if app.fermentable_dialog.is_some() {
        handle_fermentable_dialog(app, key);
        return;
    }

    // Hop dialog
    if app.hop_dialog.is_some() {
        handle_hop_dialog(app, key);
        return;
    }

    // Culture dialog
    if app.culture_dialog.is_some() {
        handle_culture_dialog(app, key);
        return;
    }

    // Batch size dialog
    if app.batch_size_dialog.is_some() {
        handle_batch_size_dialog(app, key);
        return;
    }

    // Normal edit mode — global keys first
    match key {
        KeyCode::Esc | KeyCode::Char('q') => { app.back_to_list(); return; }
        KeyCode::Char('n') => { app.editing_name = true; return; }
        KeyCode::Char('s') => { app.open_style_selector(); return; }
        KeyCode::Char('v') => { app.open_batch_size_dialog(); return; }
        KeyCode::Char('b') => { app.create_batch_from_current(); return; }
        KeyCode::Char('1') => { app.set_tab(0); return; }
        KeyCode::Char('2') => { app.set_tab(1); return; }
        KeyCode::Char('3') => { app.set_tab(2); return; }
        KeyCode::Char('4') => { app.set_tab(3); return; }
        KeyCode::Char('5') => { app.set_tab(4); return; }
        _ => {}
    }

    // Tab-specific keys
    if app.active_tab == Tab::Fermentables {
        match key {
            KeyCode::Char('a') => app.open_add_fermentable(),
            KeyCode::Enter => app.open_edit_fermentable(),
            KeyCode::Char('d') => app.delete_selected_fermentable(),
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(ref doc) = app.current_doc {
                    let len = doc.recipe.ingredients.fermentable_additions.len();
                    if len > 0 && app.fermentable_list_index < len - 1 {
                        app.fermentable_list_index += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if app.fermentable_list_index > 0 {
                    app.fermentable_list_index -= 1;
                }
            }
            _ => {}
        }
    } else if app.active_tab == Tab::Hops {
        match key {
            KeyCode::Char('a') => app.open_add_hop(),
            KeyCode::Enter => app.open_edit_hop(),
            KeyCode::Char('d') => app.delete_selected_hop(),
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(ref doc) = app.current_doc {
                    let len = doc.recipe.ingredients.hop_additions.len();
                    if len > 0 && app.hop_list_index < len - 1 {
                        app.hop_list_index += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if app.hop_list_index > 0 {
                    app.hop_list_index -= 1;
                }
            }
            _ => {}
        }
    } else if app.active_tab == Tab::Cultures {
        match key {
            KeyCode::Char('a') => app.open_add_culture(),
            KeyCode::Enter => app.open_edit_culture(),
            KeyCode::Char('d') => app.delete_selected_culture(),
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(ref doc) = app.current_doc {
                    let len = doc.recipe.ingredients.culture_additions.len();
                    if len > 0 && app.culture_list_index < len - 1 {
                        app.culture_list_index += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if app.culture_list_index > 0 {
                    app.culture_list_index -= 1;
                }
            }
            _ => {}
        }
    }
}

fn handle_fermentable_dialog(app: &mut App, key: KeyCode) {
    let step = app
        .fermentable_dialog
        .as_ref()
        .map(|d| d.step.clone())
        .unwrap();

    match step {
        FermentableDialogStep::SelectFermentable => {
            let action = app
                .fermentable_dialog
                .as_mut()
                .unwrap()
                .selector
                .handle_key(key);
            match action {
                search_selector::SearchAction::Confirm(idx) => {
                    let dialog = app.fermentable_dialog.as_mut().unwrap();
                    dialog.selected_fermentable_index = idx;
                    dialog.step = FermentableDialogStep::EnterAmount;
                }
                search_selector::SearchAction::Cancel => {
                    app.fermentable_dialog = None;
                }
                search_selector::SearchAction::Nothing => {}
            }
        }
        FermentableDialogStep::EnterAmount => match key {
            KeyCode::Enter => {
                let dialog = app.fermentable_dialog.as_mut().unwrap();
                // Validate it parses as a number
                if dialog.amount_input.parse::<f64>().is_ok() {
                    dialog.step = FermentableDialogStep::SelectUnit;
                }
            }
            KeyCode::Esc => {
                app.fermentable_dialog = None;
            }
            KeyCode::Backspace => {
                app.fermentable_dialog.as_mut().unwrap().amount_input.pop();
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                app.fermentable_dialog
                    .as_mut()
                    .unwrap()
                    .amount_input
                    .push(c);
            }
            _ => {}
        },
        FermentableDialogStep::SelectUnit => match key {
            KeyCode::Enter => {
                app.confirm_fermentable_dialog();
            }
            KeyCode::Esc => {
                app.fermentable_dialog = None;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                let dialog = app.fermentable_dialog.as_mut().unwrap();
                if dialog.unit_index > 0 {
                    dialog.unit_index -= 1;
                } else {
                    dialog.unit_index = MASS_UNITS.len() - 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                let dialog = app.fermentable_dialog.as_mut().unwrap();
                dialog.unit_index = (dialog.unit_index + 1) % MASS_UNITS.len();
            }
            _ => {}
        },
    }
}

fn handle_hop_dialog(app: &mut App, key: KeyCode) {
    let step = app
        .hop_dialog
        .as_ref()
        .map(|d| d.step.clone())
        .unwrap();

    match step {
        HopDialogStep::SelectHop => {
            let action = app
                .hop_dialog
                .as_mut()
                .unwrap()
                .selector
                .handle_key(key);
            match action {
                search_selector::SearchAction::Confirm(idx) => {
                    let dialog = app.hop_dialog.as_mut().unwrap();
                    dialog.selected_hop_index = idx;
                    dialog.step = HopDialogStep::EnterAmount;
                }
                search_selector::SearchAction::Cancel => {
                    app.hop_dialog = None;
                }
                search_selector::SearchAction::Nothing => {}
            }
        }
        HopDialogStep::EnterAmount => match key {
            KeyCode::Enter => {
                let dialog = app.hop_dialog.as_mut().unwrap();
                if dialog.amount_input.parse::<f64>().is_ok() {
                    dialog.step = HopDialogStep::SelectUnit;
                }
            }
            KeyCode::Esc => {
                app.hop_dialog = None;
            }
            KeyCode::Backspace => {
                app.hop_dialog.as_mut().unwrap().amount_input.pop();
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                app.hop_dialog.as_mut().unwrap().amount_input.push(c);
            }
            _ => {}
        },
        HopDialogStep::SelectUnit => match key {
            KeyCode::Enter => {
                let dialog = app.hop_dialog.as_mut().unwrap();
                dialog.step = HopDialogStep::SelectUse;
            }
            KeyCode::Esc => {
                app.hop_dialog = None;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                let dialog = app.hop_dialog.as_mut().unwrap();
                if dialog.unit_index > 0 {
                    dialog.unit_index -= 1;
                } else {
                    dialog.unit_index = MASS_UNITS.len() - 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                let dialog = app.hop_dialog.as_mut().unwrap();
                dialog.unit_index = (dialog.unit_index + 1) % MASS_UNITS.len();
            }
            _ => {}
        },
        HopDialogStep::SelectUse => match key {
            KeyCode::Enter => {
                let dialog = app.hop_dialog.as_mut().unwrap();
                dialog.step = HopDialogStep::EnterTime;
            }
            KeyCode::Esc => {
                app.hop_dialog = None;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                let dialog = app.hop_dialog.as_mut().unwrap();
                if dialog.use_index > 0 {
                    dialog.use_index -= 1;
                } else {
                    dialog.use_index = USE_TYPES.len() - 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                let dialog = app.hop_dialog.as_mut().unwrap();
                dialog.use_index = (dialog.use_index + 1) % USE_TYPES.len();
            }
            _ => {}
        },
        HopDialogStep::EnterTime => match key {
            KeyCode::Enter => {
                let dialog = app.hop_dialog.as_ref().unwrap();
                if dialog.time_input.parse::<i64>().is_ok() {
                    app.confirm_hop_dialog();
                }
            }
            KeyCode::Esc => {
                app.hop_dialog = None;
            }
            KeyCode::Backspace => {
                app.hop_dialog.as_mut().unwrap().time_input.pop();
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                app.hop_dialog.as_mut().unwrap().time_input.push(c);
            }
            _ => {}
        },
    }
}

fn handle_culture_dialog(app: &mut App, key: KeyCode) {
    let step = app
        .culture_dialog
        .as_ref()
        .map(|d| d.step.clone())
        .unwrap();

    match step {
        CultureDialogStep::SelectCulture => {
            let action = app
                .culture_dialog
                .as_mut()
                .unwrap()
                .selector
                .handle_key(key);
            match action {
                search_selector::SearchAction::Confirm(idx) => {
                    let dialog = app.culture_dialog.as_mut().unwrap();
                    dialog.selected_culture_index = idx;
                    dialog.step = CultureDialogStep::EnterAmount;
                }
                search_selector::SearchAction::Cancel => {
                    app.culture_dialog = None;
                }
                search_selector::SearchAction::Nothing => {}
            }
        }
        CultureDialogStep::EnterAmount => match key {
            KeyCode::Enter => {
                let dialog = app.culture_dialog.as_mut().unwrap();
                if dialog.amount_input.parse::<f64>().is_ok() {
                    dialog.step = CultureDialogStep::SelectUnit;
                }
            }
            KeyCode::Esc => {
                app.culture_dialog = None;
            }
            KeyCode::Backspace => {
                app.culture_dialog.as_mut().unwrap().amount_input.pop();
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                app.culture_dialog.as_mut().unwrap().amount_input.push(c);
            }
            _ => {}
        },
        CultureDialogStep::SelectUnit => match key {
            KeyCode::Enter => {
                app.confirm_culture_dialog();
            }
            KeyCode::Esc => {
                app.culture_dialog = None;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                let dialog = app.culture_dialog.as_mut().unwrap();
                if dialog.unit_index > 0 {
                    dialog.unit_index -= 1;
                } else {
                    dialog.unit_index = CULTURE_UNITS.len() - 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                let dialog = app.culture_dialog.as_mut().unwrap();
                dialog.unit_index = (dialog.unit_index + 1) % CULTURE_UNITS.len();
            }
            _ => {}
        },
    }
}

fn handle_batch_size_dialog(app: &mut App, key: KeyCode) {
    let step = app
        .batch_size_dialog
        .as_ref()
        .map(|d| d.step.clone())
        .unwrap();

    match step {
        BatchSizeDialogStep::EnterValue => match key {
            KeyCode::Enter => {
                let dialog = app.batch_size_dialog.as_mut().unwrap();
                if dialog.value_input.parse::<f64>().is_ok() {
                    dialog.step = BatchSizeDialogStep::SelectUnit;
                }
            }
            KeyCode::Esc => app.cancel_batch_size_dialog(),
            KeyCode::Backspace => {
                app.batch_size_dialog.as_mut().unwrap().value_input.pop();
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                app.batch_size_dialog
                    .as_mut()
                    .unwrap()
                    .value_input
                    .push(c);
            }
            _ => {}
        },
        BatchSizeDialogStep::SelectUnit => match key {
            KeyCode::Enter => app.confirm_batch_size_dialog(),
            KeyCode::Esc => app.cancel_batch_size_dialog(),
            KeyCode::Left | KeyCode::Char('h') => {
                let dialog = app.batch_size_dialog.as_mut().unwrap();
                if dialog.unit_index > 0 {
                    dialog.unit_index -= 1;
                } else {
                    dialog.unit_index = VOLUME_UNITS.len() - 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                let dialog = app.batch_size_dialog.as_mut().unwrap();
                dialog.unit_index = (dialog.unit_index + 1) % VOLUME_UNITS.len();
            }
            _ => {}
        },
    }
}
