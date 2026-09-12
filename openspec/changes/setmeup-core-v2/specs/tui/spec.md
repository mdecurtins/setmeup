## ADDED Requirements

### Requirement: Linear wizard with progress bar

The `configure` command SHALL launch a 6-step linear wizard. Each step fills
the terminal and shows a progress bar at the top.

#### Scenario: Wizard renders progress bar

- **WHEN** the wizard is on step 3 of 6
- **THEN** the progress bar shows 50% filled and the step label "3 of 6"

#### Scenario: Wizard navigates forward

- **WHEN** a user completes a step (Enter on a selected option, or fills the
  required input)
- **THEN** the next step is rendered full-screen

#### Scenario: Wizard navigates backward

- **WHEN** a user presses Escape or selects "Back"
- **THEN** the previous step is rendered with its prior state preserved

### Requirement: Wizard steps

The wizard SHALL have these steps in order:

| Step | Title | Content |
|------|-------|---------|
| 1 | Welcome | Introduction, info message about what setmeup does |
| 2 | Credentials | Secret acquisition policy (paste/device-flow/env) for each declared credential; masked input for paste policy |
| 3 | Packages | Tool selection: list of pre-configured packages with toggles, `from` method shown |
| 4 | Repos | Repository URLs to clone; free-text input with add/remove |
| 5 | Shell | Shell customization: prompt toggle, function names (body deferred), PATH extends, env vars |
| 6 | Review | Summary of all selections; confirm to write manifest |

#### Scenario: Step content uses defaults-first

- **WHEN** a manifest already exists (from a prior run)
- **THEN** each step pre-fills from the existing manifest; user confirms or
  changes

#### Scenario: Wizard validates on advance

- **WHEN** a user attempts to advance past a step with invalid input
- **THEN** validation error is shown on screen and advance is blocked until
  fixed

### Requirement: Wizard handles terminal resize

The wizard SHALL handle SIGWINCH (terminal resize) events via crossterm's
event stream, redrawing the current step at the new terminal dimensions.

#### Scenario: Terminal resized during wizard

- **WHEN** a user resizes the terminal window during a wizard step
- **THEN** the wizard redraws the step content to fit the new dimensions
  without garbling the display or losing input state

### Requirement: Spinner for async operations

The wizard SHALL show a spinner during operations that may block (credential
validation, network checks).

#### Scenario: Credential validation shows spinner

- **WHEN** the wizard validates a pasted token against the provider API
- **THEN** a spinner is shown with the label "Validating token..." until
  complete

### Requirement: Non-TTY degradation

When stdin is not a terminal, the wizard SHALL error with a clear message
directing the user to non-interactive modes (manifest file directly).

#### Scenario: Wizard errors in non-TTY

- **WHEN** `setmeup configure` is run in a piped/CI context
- **THEN** the wizard prints an error and exits non-zero without hanging

## MODIFIED Requirements

### Requirement: Wizard authors the manifest

The TUI SHALL provide a wizard that collects configuration into the declarative
manifest. **The wizard is now a 6-step linear interactive flow using crossterm
instead of ratatui. The `configure` command writes the manifest; it does not
provision the system.**

#### Scenario: First-run wizard writes a manifest

- **WHEN** a user runs `setmeup configure` on a machine with no manifest
- **THEN** the wizard collects preferences across 6 steps, validates, and writes
  `manifest.yml` to the config directory

#### Scenario: Wizard edits an existing manifest

- **WHEN** a user runs `setmeup configure` against an existing manifest
- **THEN** each step pre-fills from the existing manifest; the user confirms or
  changes values before saving

### Requirement: Dashboard observes apply runs

The TUI SHALL provide a dashboard view showing live progress and per-item
outcome during `apply`. **(Unchanged from V1 — the dashboard remains a
channel-fed line-mode renderer.)**

#### Scenario: Apply run shown in dashboard

- **WHEN** a user runs apply with the TUI visible
- **THEN** items are shown with ✓/✗/⊘ state as the run proceeds

#### Scenario: Summary after apply

- **WHEN** an apply run completes in the dashboard
- **THEN** a summary of succeeded, failed, and skipped items is displayed

### Requirement: Secret acquisition rendered by policy

The wizard SHALL render the appropriate prompt type for each credential based on
the manifest's declared acquisition policy. **(Unchanged from V1 — paste path
is implemented; device-flow is deferred to #19.)**

#### Scenario: Paste-policy credential

- **WHEN** a credential is declared with `via: paste`
- **THEN** the wizard shows a masked input for pasting the token

#### Scenario: Device-flow credential

- **WHEN** a credential is declared with `via: device-flow`
- **THEN** the wizard shows a code and URL for browser authorization and polls
  for completion
- **NOTE** — device-flow rendering is deferred to follow-up #19

### Requirement: Secrets are masked

The wizard SHALL mask all secret input and SHALL NOT surface stored secrets in
plaintext. **(Unchanged from V1.)**

#### Scenario: Token entry is masked

- **WHEN** a user enters a token in the wizard
- **THEN** it is displayed masked and never echoed to output
