//! TUI: the manifest-authoring wizard and the apply dashboard.

pub mod dashboard;
pub mod steps;
pub mod wizard;

// Re-exports from dashboard
pub use dashboard::{DashboardEvent, DashboardSummary, render_apply_dashboard, spin_until};

// Re-exports from wizard
pub use wizard::{prompt_masked, run_wizard, write_manifest};

/// Whether we are attached to an interactive terminal.
pub fn interactive() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}
