//! TUI module for the CTO Platform installer
//!
//! Provides a beautiful terminal user interface using ratatui for
//! guiding users through the CTO platform installation process.

mod app;
mod event;
mod theme;

pub mod screens;
pub mod widgets;

use app::App;
use event::EventHandler;

use anyhow::{bail, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    tty::IsTty,
};
use ratatui::prelude::*;
use std::io::{self, Stdout};

/// Terminal type alias for convenience
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Check if we're running in an interactive terminal
pub fn is_interactive() -> bool {
    io::stdout().is_tty()
}

/// Initialize the terminal for TUI mode
pub fn init() -> Result<Tui> {
    if !is_interactive() {
        bail!("TUI requires an interactive terminal. Use --no-tui for non-interactive mode.");
    }

    enable_raw_mode()?;
    
    // If any subsequent operation fails, we must restore raw mode
    let result = (|| {
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        Terminal::new(backend).map_err(Into::into)
    })();
    
    // If initialization failed, restore terminal state before returning error
    if result.is_err() {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }
    
    result
}

/// Restore the terminal to its original state
pub fn restore() -> Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

/// Run the TUI application
pub async fn run(demo_mode: bool) -> Result<()> {
    // Initialize terminal
    let mut terminal = init()?;

    // Create app state
    let mut app = App::new(demo_mode);

    // Create event handler
    let event_handler = EventHandler::new(250); // 250ms tick rate

    // Main loop
    let result = app.run(&mut terminal, event_handler).await;

    // Restore terminal
    restore()?;

    result
}

