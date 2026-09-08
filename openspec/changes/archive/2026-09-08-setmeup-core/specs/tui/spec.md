## ADDED Requirements

### Requirement: Wizard authors the manifest
The TUI SHALL provide a wizard that collects configuration into the declarative manifest, not directly into the system.

#### Scenario: First-run wizard writes a manifest
- **WHEN** a user runs `setmeup configure` (wizard-lite) on a machine with no manifest
- **THEN** an empty manifest is validated and written to manifest.yml, and no system provisioning happens during the wizard

#### Scenario: Wizard edits an existing manifest
- **WHEN** a user runs `setmeup configure` against an existing manifest
- **THEN** the existing manifest is loaded, validated, and saved on completion (pre-filled editing)
- **NOTE** — full interactive wizard screens (identity/credential/tool/dotfiles/shell/review) are deferred to follow-up #20 and are NOT part of this change's contract.

### Requirement: Dashboard observes apply runs
The TUI SHALL provide a dashboard view showing live progress and per-item outcome during `apply`.

#### Scenario: Apply run shown in dashboard
- **WHEN** a user runs apply with the TUI visible
- **THEN** items are shown with running/succeeded/failed state as the run proceeds

#### Scenario: Summary after apply
- **WHEN** an apply run completes in the dashboard
- **THEN** a summary of succeeded, failed, and skipped items is displayed

### Requirement: Secret acquisition rendered by policy
The wizard SHALL render the appropriate prompt type for each credential based on the manifest's declared acquisition policy.

#### Scenario: Paste-policy credential
- **WHEN** a credential is declared with `via: paste`
- **THEN** the wizard shows a masked input for pasting the token

#### Scenario: Device-flow credential
- **WHEN** a credential is declared with `via: device-flow`
- **THEN** the wizard shows a code and URL for browser authorization and polls for completion
- **NOTE** — device-flow rendering is deferred to follow-up #19 and is NOT part of this change's contract.

### Requirement: Secrets are masked
The wizard SHALL mask all secret input and SHALL NOT surface stored secrets in plaintext.

#### Scenario: Token entry is masked
- **WHEN** a user enters a token in the wizard
- **THEN** it is displayed masked and never echoed to output