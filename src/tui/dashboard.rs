//! Apply dashboard: channel-fed line-mode renderer for provisioning progress.
//!
//! This module is unchanged from V1. It provides live progress reporting
//! during `setmeup apply` using a channel-based event stream.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::error::{Result, SetmeupError};

use crate::tui::interactive;

/// Events streamed to the dashboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DashboardEvent {
    Started(String),
    Succeeded(String),
    Failed(String, String),
    Skipped(String, String),
    Finished,
}

/// Final tally from a dashboard render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DashboardSummary {
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// Render a simple live apply dashboard.
///
/// Consumes items from a channel of `DashboardEvent` lines and prints them
/// as they arrive, then a final summary. In non-TTY contexts this degrades to
/// line-mode output.
pub fn render_apply_dashboard(
    rx: mpsc::Receiver<DashboardEvent>,
    total: usize,
) -> Result<DashboardSummary> {
    let mut succeeded = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    while let Ok(event) = rx.recv() {
        match event {
            DashboardEvent::Started(label) => {
                if interactive() {
                    println!("▶ {label}");
                }
            }
            DashboardEvent::Succeeded(label) => {
                println!("✓ {label}");
                succeeded += 1;
            }
            DashboardEvent::Failed(label, detail) => {
                println!("✗ {label}: {detail}");
                failed += 1;
            }
            DashboardEvent::Skipped(label, _reason) => {
                println!("⊘ {label}");
                skipped += 1;
            }
            DashboardEvent::Finished => break,
        }
    }
    let _ = total;
    println!();
    println!("Done: {succeeded} succeeded, {failed} failed, {skipped} skipped");
    Ok(DashboardSummary {
        succeeded,
        failed,
        skipped,
    })
}

/// Real-time progress spinner for long-running provisioning steps.
pub fn spin_until<F>(label: impl Into<String>, f: F) -> Result<()>
where
    F: FnOnce() -> Result<()> + Send + 'static,
{
    let label = label.into();
    if !interactive() {
        // Non-interactive: just run synchronously.
        println!("{label}…");
        return f();
    }
    let handle = thread::spawn(f);
    let spinner = ['|', '/', '-', '\\'];
    let mut i = 0;
    loop {
        if handle.is_finished() {
            break;
        }
        print!("\r{label} {}", spinner[i % spinner.len()]);
        use std::io::Write;
        std::io::stdout().flush().ok();
        i += 1;
        thread::sleep(Duration::from_millis(80));
    }
    // Wait for the worker and clear the line.
    match handle.join() {
        Ok(Ok(())) => {
            print!("\r{}", " ".repeat(label.chars().count() + 3));
            use std::io::Write;
            std::io::stdout().flush().ok();
            println!("\r{label} ✓");
            Ok(())
        }
        Ok(Err(e)) => {
            println!("\r{label} ✗ {e}");
            Err(e)
        }
        Err(_) => Err(SetmeupError::Provisioning("worker panicked".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_tallies_events() {
        let (tx, rx) = mpsc::channel();
        tx.send(DashboardEvent::Started("a".into())).unwrap();
        tx.send(DashboardEvent::Succeeded("a".into())).unwrap();
        tx.send(DashboardEvent::Started("b".into())).unwrap();
        tx.send(DashboardEvent::Failed("b".into(), "bad".into()))
            .unwrap();
        tx.send(DashboardEvent::Skipped("c".into(), "unchanged".into()))
            .unwrap();
        tx.send(DashboardEvent::Finished).unwrap();
        let s = render_apply_dashboard(rx, 3).unwrap();
        assert_eq!(s.succeeded, 1);
        assert_eq!(s.failed, 1);
        assert_eq!(s.skipped, 1);
    }
}
