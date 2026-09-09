## ADDED Requirements

### Requirement: Deterministic tool installation
Quality-gate tool installation in the CI workflow SHALL declare the exact tool via an explicit `with: tool:` input to the installer action, rather than relying on the installer's default. This keeps the installed tool stable across installer-action SHA bumps, whose defaults may change between releases.

#### Scenario: Installer default changes between versions
- **WHEN** `taiki-e/install-action` (or an equivalent installer action) is bumped to a SHA whose default tool differs from the previously pinned SHA
- **THEN** each quality-gate job still installs its declared tool — `cargo-deny` for the `deps` job, `cargo-llvm-cov` for the `coverage` job — because the `with: tool:` input is explicit

#### Scenario: Tool input omitted
- **WHEN** a CI step installs a quality-gate tool without an explicit `with: tool:` input
- **THEN** it is a spec violation: the step must add the explicit input, because omitting it couples CI to the installer's changing default (see issue #31)