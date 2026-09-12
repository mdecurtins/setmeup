//! Wizard steps: definitions, pure helpers, and state.
//!
//! This module contains the step definitions and pure helper functions used by
//! the interactive wizard. The step definitions and helpers are unit-testable
//! without a terminal.

/// The titles and descriptions of each wizard step.
pub const STEP_NAMES: &[(&str, &str); 6] = &[
    ("Welcome", "Introduction to setmeup"),
    ("Credentials", "Secret acquisition policies"),
    ("Packages", "Tool selection and package sources"),
    ("Repos", "Repository URLs to clone"),
    ("Shell", "Shell customization"),
    ("Review", "Summary and confirm"),
];

/// Return the title for a 1-indexed step number.
pub fn step_title(step: usize) -> &'static str {
    STEP_NAMES
        .get(step.saturating_sub(1))
        .map(|(t, _)| *t)
        .unwrap_or("Unknown")
}

/// Return the description for a 1-indexed step number.
pub fn step_description(step: usize) -> &'static str {
    STEP_NAMES
        .get(step.saturating_sub(1))
        .map(|(_, d)| *d)
        .unwrap_or("")
}

/// Render a text-based progress bar of `width` cells.
///
/// Filled portion represents `current / total`. Returns a string like
/// `[██████░░░░░░░░] 33%`.
pub fn progress_bar(current: usize, total: usize, width: usize) -> String {
    let filled = if total == 0 {
        0
    } else {
        (current * width).saturating_div(total)
    };
    let filled = filled.min(width);
    let empty = width.saturating_sub(filled);
    let pct = if total == 0 {
        0
    } else {
        (current as f64 / total as f64 * 100.0).round() as usize
    };

    let mut bar = String::with_capacity(width + 8);
    bar.push('[');
    for _ in 0..filled {
        bar.push('█');
    }
    for _ in 0..empty {
        bar.push('░');
    }
    bar.push_str(&format!("] {pct}%"));
    bar
}

/// State carried through the wizard.
///
/// This is the intermediate state that the wizard mutations accumulate.
/// The final conversion back to `Manifest` is handled by the wizard
/// orchestrator.
#[derive(Debug, Clone, Default)]
pub struct WizardState {
    /// The current step index (1-indexed).
    pub current_step: usize,
    /// The total number of steps.
    pub total_steps: usize,
}

impl WizardState {
    pub fn new() -> Self {
        WizardState {
            current_step: 1,
            total_steps: 6,
        }
    }

    /// Advance to the next step, returning `true` if there is a next step.
    pub fn advance(&mut self) -> bool {
        if self.current_step < self.total_steps {
            self.current_step += 1;
            true
        } else {
            false
        }
    }

    /// Go back to the previous step, returning `true` if there is a previous step.
    pub fn retreat(&mut self) -> bool {
        if self.current_step > 1 {
            self.current_step -= 1;
            true
        } else {
            false
        }
    }

    /// Reset to step 1 (used when restarting the wizard).
    pub fn reset(&mut self) {
        self.current_step = 1;
    }

    /// Progress as a fraction (0.0–1.0).
    pub fn progress(&self) -> f64 {
        if self.total_steps == 0 {
            0.0
        } else {
            (self.current_step as f64 - 1.0) / self.total_steps as f64
        }
    }
}

/// Welcome message text (multi-line).
pub const WELCOME_TEXT: &str = r#"
                 ╔══════════════════════════════════════╗
                 ║          Welcome to setmeup           ║
                 ╚══════════════════════════════════════╝

setmeup converges any fresh OS environment to your declared
desired state — idempotently.

This wizard will guide you through 6 steps to author your
manifest.yml file. The manifest describes what your machine
should look like: packages, dotfiles, shell configuration,
repos, credentials, and more.

No changes are made to your system during this wizard.
After you complete the review step, the manifest is written
to your config directory. Run `setmeup apply` later to
converge your machine.

Press Enter to continue.
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_names_have_correct_count() {
        assert_eq!(STEP_NAMES.len(), 6);
    }

    #[test]
    fn step_title_by_number() {
        assert_eq!(step_title(1), "Welcome");
        assert_eq!(step_title(3), "Packages");
        assert_eq!(step_title(6), "Review");
        assert_eq!(step_title(99), "Unknown");
    }

    #[test]
    fn progress_bar_zero_total() {
        let bar = progress_bar(0, 0, 20);
        assert!(bar.contains("] 0%"));
    }

    #[test]
    fn progress_bar_half_filled() {
        let bar = progress_bar(3, 6, 20);
        assert!(bar.contains("50%"));
        assert!(bar.starts_with('['));
        assert!(bar.ends_with('%'));
    }

    #[test]
    fn progress_bar_full() {
        let bar = progress_bar(6, 6, 20);
        assert!(bar.contains("100%"));
    }

    #[test]
    fn progress_bar_empty() {
        let bar = progress_bar(0, 6, 20);
        assert!(bar.contains("0%"));
    }

    #[test]
    fn wizard_state_advance_and_retreat() {
        let mut state = WizardState::new();
        assert_eq!(state.current_step, 1);
        assert!(state.advance());
        assert_eq!(state.current_step, 2);
        assert!(state.retreat());
        assert_eq!(state.current_step, 1);
        assert!(!state.retreat()); // can't go below 1
        state.current_step = 6;
        assert!(!state.advance()); // can't go above 6
    }

    #[test]
    fn progress_bar_width_respected() {
        for width in [10, 20, 30] {
            let bar = progress_bar(3, 6, width);
            // Count the filled block characters (count of '█' chars)
            let filled = bar.chars().filter(|&c| c == '█').count();
            let empty = bar.chars().filter(|&c| c == '░').count();
            assert_eq!(filled + empty, width, "width {width}: bar='{bar}'");
        }
    }

    #[test]
    fn welcome_text_is_nonempty() {
        assert!(!WELCOME_TEXT.is_empty());
        assert!(WELCOME_TEXT.contains("Welcome"));
        assert!(WELCOME_TEXT.contains("setmeup"));
    }
}
