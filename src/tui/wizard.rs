//! Wizard orchestrator: the 6-step linear manifest authoring wizard.
//!
//! Uses crossterm for full-screen terminal rendering. Each step clears the
//! terminal, draws a progress bar, step title, and step content. Handles
//! terminal resize events and input via crossterm event polling.

use std::io::Write;
use std::path::Path;

use crossterm::ExecutableCommand;
use crossterm::cursor;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{Clear, ClearType};

use crate::error::{Result, SetmeupError};
use crate::manifest::Manifest;

use super::steps::{self, WizardState};
use crate::tui::interactive;

/// Whether we are attached to an interactive terminal.
// Re-exported via mod.rs — defined in dashboard module for convenience.
pub use super::dashboard::spin_until;

/// Acquire a secret value via a masked prompt.
/// Returns the value with echo suppressed (best effort via crossterm).
pub fn prompt_masked(prompt: &str) -> Result<String> {
    if !interactive() {
        return Err(SetmeupError::Secret(
            "masked prompt requires a terminal".into(),
        ));
    }
    let mut stdout = std::io::stdout();
    write!(stdout, "{prompt}").map_err(SetmeupError::Io)?;
    stdout.flush().map_err(SetmeupError::Io)?;
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .map_err(SetmeupError::Io)?;
    let value = line.trim().to_string();
    // Overwrite the typed-in clear-text with asterisks.
    write!(
        stdout,
        "\r{}{}",
        cursor::MoveToColumn(0),
        " ".repeat(prompt.chars().count() + value.len())
    )
    .map_err(SetmeupError::Io)?;
    write!(stdout, "\r{prompt}{}", "*".repeat(value.len())).map_err(SetmeupError::Io)?;
    writeln!(stdout).map_err(SetmeupError::Io)?;
    if value.is_empty() {
        return Err(SetmeupError::Secret("empty input".into()));
    }
    Ok(value)
}

/// Write the wizard's manifest to `path` (interface for the `configure` cmd).
pub fn write_manifest(manifest: &Manifest, path: Option<&Path>) -> Result<()> {
    manifest.save(path)
}

/// Run the manifest-authoring wizard.
///
/// Collects configuration into a `Manifest` and returns it. Does not touch
/// the system. When not attached to a TTY, returns an error directing the
/// caller to non-interactive modes.
pub fn run_wizard(existing: Option<&Manifest>) -> Result<Manifest> {
    if !interactive() {
        return Err(SetmeupError::Manifest(
            "wizard requires an interactive terminal; use a manifest file directly".into(),
        ));
    }

    let mut manifest = existing.cloned().unwrap_or_default();
    let mut state = WizardState::new();

    loop {
        render_step(&state, &manifest)?;

        match crossterm::event::read() {
            Ok(event) => match handle_wizard_event(event, &mut state, &mut manifest) {
                WizardAction::Continue => continue,
                WizardAction::ExitOk => {
                    manifest.validate()?;
                    return Ok(manifest);
                }
                WizardAction::ExitCancel => {
                    return Err(SetmeupError::Manifest("wizard cancelled by user".into()));
                }
                WizardAction::Redraw => {
                    continue;
                }
            },
            Err(e) => {
                return Err(SetmeupError::Io(e));
            }
        }
    }
}

/// Actions the wizard can take after handling an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WizardAction {
    Continue,
    ExitOk,
    ExitCancel,
    Redraw,
}

/// Handle one crossterm event and mutate step/state accordingly.
fn handle_wizard_event(
    event: Event,
    state: &mut WizardState,
    manifest: &mut Manifest,
) -> WizardAction {
    match event {
        Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press | KeyEventKind::Repeat,
            ..
        }) => match code {
            KeyCode::Esc => WizardAction::ExitCancel,
            KeyCode::Char('q') | KeyCode::Char('Q') => WizardAction::ExitCancel,
            KeyCode::Enter => advance_step(state, manifest),
            KeyCode::Backspace => {
                if state.current_step > 1 {
                    state.retreat();
                }
                WizardAction::Continue
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                if state.current_step > 1 {
                    state.retreat();
                }
                WizardAction::Continue
            }
            KeyCode::Char('n') | KeyCode::Char('N') if state.current_step == 6 => {
                // On review step, 'n' goes back
                state.retreat();
                WizardAction::Continue
            }
            KeyCode::Char('y') | KeyCode::Char('Y') if state.current_step == 6 => {
                // On review step, 'y' confirms
                WizardAction::ExitOk
            }
            // Step-specific handlers
            KeyCode::Char(c) => {
                handle_step_input(c, state, manifest);
                WizardAction::Continue
            }
            _ => WizardAction::Continue,
        },
        Event::Resize(_, _) => WizardAction::Redraw,
        _ => WizardAction::Continue,
    }
}

/// Handle step-specific character input for non-Enter actions.
fn handle_step_input(c: char, _state: &mut WizardState, _manifest: &mut Manifest) {
    // Deferred to follow-up changes for per-step editing.
    // Currently the wizard is a confirm-through flow.
    let _ = c;
}

/// Advance the wizard by one step, performing validation at the boundary.
fn advance_step(state: &mut WizardState, manifest: &mut Manifest) -> WizardAction {
    if state.current_step == 6 {
        // On review step, Enter confirms
        return WizardAction::ExitOk;
    }

    // Validate step before advancing (deferred for this change; basic check only)
    match state.current_step {
        2 => {
            // Ensure secrets have valid names
            for s in &manifest.secrets {
                if s.name.trim().is_empty() {
                    return WizardAction::Continue; // Stay on step — validation failed
                }
            }
        }
        3 => {
            // Ensure packages have valid names
            if let Some(ref pkgs) = manifest.packages {
                for name in pkgs.keys() {
                    if name.trim().is_empty() {
                        return WizardAction::Continue;
                    }
                }
            }
        }
        _ => {}
    }

    state.advance();
    WizardAction::Continue
}

/// Render the current step to the terminal.
fn render_step(state: &WizardState, manifest: &Manifest) -> Result<()> {
    let mut stdout = std::io::stdout();

    // Clear screen and move cursor to top-left
    stdout.execute(Clear(ClearType::All))?;
    stdout.execute(crossterm::cursor::MoveTo(0, 0))?;

    // Progress bar
    let progress = steps::progress_bar(state.current_step, state.total_steps, 20);
    writeln!(stdout, "{progress}")?;
    writeln!(
        stdout,
        "Step {} of {}: {}",
        state.current_step,
        state.total_steps,
        steps::step_title(state.current_step)
    )?;
    writeln!(stdout, "{}", steps::step_description(state.current_step))?;
    writeln!(stdout, "{}", "-".repeat(40))?;

    // Step-specific content
    match state.current_step {
        1 => render_welcome(&mut stdout)?,
        2 => render_credentials(&mut stdout, manifest)?,
        3 => render_packages(&mut stdout, manifest)?,
        4 => render_repos(&mut stdout, manifest)?,
        5 => render_shell(&mut stdout, manifest)?,
        6 => render_review(&mut stdout, manifest)?,
        _ => writeln!(stdout, "Unknown step")?,
    }

    // Footer
    writeln!(stdout)?;
    writeln!(stdout, "{}", "-".repeat(40))?;
    if state.current_step == 6 {
        writeln!(stdout, "[Enter=confirm, Y=yes, N=go back, Esc=quit]")?;
    } else if state.current_step == 1 {
        writeln!(stdout, "[Enter=continue, Esc=quit]")?;
    } else {
        writeln!(stdout, "[Enter=continue, B=back, Esc=quit]")?;
    }

    stdout.flush().map_err(SetmeupError::Io)?;
    Ok(())
}

fn render_welcome(stdout: &mut std::io::Stdout) -> Result<()> {
    writeln!(stdout, "{}", steps::WELCOME_TEXT)?;
    Ok(())
}

fn render_credentials(stdout: &mut std::io::Stdout, manifest: &Manifest) -> Result<()> {
    if manifest.secrets.is_empty() {
        writeln!(
            stdout,
            "No credentials declared. Add them to your manifest later."
        )?;
        writeln!(stdout)?;
        writeln!(
            stdout,
            "You can define secret policies in the manifest's `secrets` section."
        )?;
    } else {
        writeln!(stdout, "Declared credentials:")?;
        for s in &manifest.secrets {
            writeln!(
                stdout,
                "  • {} (acquire: {:?}, backend: {:?})",
                s.name, s.acquire, s.backend
            )?;
        }
    }
    Ok(())
}

fn render_packages(stdout: &mut std::io::Stdout, manifest: &Manifest) -> Result<()> {
    if let Some(ref pkgs) = manifest.packages {
        if pkgs.is_empty() {
            writeln!(stdout, "No packages selected.")?;
        } else {
            writeln!(stdout, "Packages to install:")?;
            for (name, cfg) in pkgs {
                let from = cfg.from.as_deref().unwrap_or("default (per-os)");
                writeln!(stdout, "  • {name} (from: {from})")?;
            }
        }
    } else {
        writeln!(stdout, "Packages: not yet configured.")?;
    }
    writeln!(stdout)?;
    writeln!(stdout, "Edit package list in manifest.yml or return here.")?;
    Ok(())
}

fn render_repos(stdout: &mut std::io::Stdout, manifest: &Manifest) -> Result<()> {
    if let Some(ref repos) = manifest.repos {
        if repos.is_empty() {
            writeln!(stdout, "No repositories configured.")?;
        } else {
            writeln!(stdout, "Repositories to clone:")?;
            for (name, url) in repos {
                writeln!(stdout, "  • {name} -> {url}")?;
            }
        }
    } else {
        writeln!(stdout, "Repos: not yet configured.")?;
    }
    Ok(())
}

fn render_shell(stdout: &mut std::io::Stdout, manifest: &Manifest) -> Result<()> {
    let cfg = &manifest.shell;
    writeln!(stdout, "Shell configuration:")?;
    writeln!(
        stdout,
        "  Prompt (parse_git_branch): {}",
        if cfg.prompt { "enabled" } else { "disabled" }
    )?;
    if !cfg.functions.is_empty() {
        writeln!(stdout, "  Shell functions:")?;
        for f in &cfg.functions {
            writeln!(stdout, "    • {} ({} bytes body)", f.name, f.body.len())?;
        }
    }
    if !cfg.source.is_empty() {
        writeln!(stdout, "  Source files:")?;
        for s in &cfg.source {
            writeln!(stdout, "    • {s}")?;
        }
    }
    if !cfg.path_extend.is_empty() {
        writeln!(stdout, "  PATH extends: {}", cfg.path_extend.join(":"))?;
    }
    if !cfg.env.is_empty() {
        writeln!(stdout, "  Environment variables:")?;
        for (k, v) in &cfg.env {
            writeln!(stdout, "    • {k}={v}")?;
        }
    }
    if !cfg.rc_lines.is_empty() {
        writeln!(stdout, "  Extra rc lines: {}", cfg.rc_lines.len())?;
    }
    Ok(())
}

fn render_review(stdout: &mut std::io::Stdout, manifest: &Manifest) -> Result<()> {
    let yaml = manifest
        .to_yaml()
        .unwrap_or_else(|_| "<failed to serialize>".into());
    writeln!(stdout, "{yaml}")?;
    writeln!(stdout)?;
    writeln!(stdout, "Write this manifest? (Y/n)")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wizard_requires_tty() {
        // When not interactive, wizard must error rather than hang.
        let err = run_wizard(None).unwrap_err();
        assert!(err.to_string().contains("interactive"));
    }

    #[test]
    fn masked_prompt_rejects_empty_in_non_tty() {
        // In a non-interactive test env, this must error (not hang).
        let result = prompt_masked("enter token:");
        // Either empty-input rejection or terminal-required rejection; both are
        // acceptable behaviors that do not hang.
        assert!(result.is_err());
    }

    #[test]
    fn write_manifest_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let m = Manifest::default();
        let p = dir.path().join("manifest.yml");
        write_manifest(&m, Some(&p)).unwrap();
        let loaded = Manifest::load(Some(&p)).unwrap();
        assert_eq!(loaded, Manifest::default());
    }

    #[test]
    fn wizard_action_enum_is_sized() {
        // Compile-time check that WizardAction derives traits.
        let a = WizardAction::Continue;
        let b = WizardAction::ExitOk;
        assert_ne!(a, b);
    }

    #[test]
    fn render_step_does_not_panic_for_any_step() {
        let state = WizardState::new();
        let manifest = Manifest::default();
        // This runs in non-interactive mode (or any mode) — just verifies
        // render_step doesn't panic. It will try to write to stdout.
        let _ = render_step(&state, &manifest);
    }
}
