mod app;
mod styles;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, Screen};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Resolve DB path
    let proj_dirs = directories::ProjectDirs::from("com", "brewdio", "brewdio")
        .expect("Could not determine data directory");
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;
    let db_path = data_dir.join("brewdio.db");

    let conn = persistence::db::init_db(db_path.to_str().unwrap())?;
    let mut app = App::new(conn);

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
                    Screen::RecipeList => handle_list_input(app, key.code),
                    Screen::RecipeEdit { .. } => handle_edit_input(app, key.code),
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_list_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('n') => app.create_recipe(),
        KeyCode::Char('d') => app.delete_selected(),
        KeyCode::Enter => app.open_selected(),
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

    if app.editing_style {
        match key {
            KeyCode::Enter => app.confirm_style(),
            KeyCode::Esc => app.cancel_style(),
            KeyCode::Char('j') | KeyCode::Down => {
                if app.style_index < crate::styles::BEER_STYLES.len() - 1 {
                    app.style_index += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if app.style_index > 0 {
                    app.style_index -= 1;
                }
            }
            _ => {}
        }
        return;
    }

    // Normal edit mode
    match key {
        KeyCode::Esc | KeyCode::Char('q') => app.back_to_list(),
        KeyCode::Char('n') => app.editing_name = true,
        KeyCode::Char('s') => app.editing_style = true,
        KeyCode::Char('1') => app.set_tab(0),
        KeyCode::Char('2') => app.set_tab(1),
        KeyCode::Char('3') => app.set_tab(2),
        KeyCode::Char('4') => app.set_tab(3),
        KeyCode::Char('5') => app.set_tab(4),
        _ => {}
    }
}
