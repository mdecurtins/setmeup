# agent-skills Specification

## Purpose

Define the agent skill architecture for extending setmeup with new provisioning capabilities. Each capability gets a `manifest-<name>` skill that encodes the research, installation, configuration, idempotency, and wizard behavior for that ecosystem.

## Requirements

### Requirement: Agent skills per capability

Every manifest capability that requires ecosystem knowledge SHALL have a corresponding skill at `.opencode/skills/manifest-<name>/SKILL.md`.

#### Scenario: Capability skill exists

- **WHEN** a user wants to add a new capability to the manifest schema
- **THEN** the workflow begins with creating the skill for that capability

#### Scenario: Skill structure

- **WHEN** a manifest-packages skill is loaded by an agent
- **THEN** it contains: Research, Manifest schema, Provisioning, Idempotency, and Wizard sections

### Requirement: Research section

The research section SHALL tell the agent how to gather authoritative information about the tool's ecosystem:

- Official documentation URL
- Latest version/release source (GitHub releases, PyPI, npm registry, apt repo)
- Installation methods available and prerequisites
- Platform availability (Linux/macOS/Windows)

#### Scenario: Agent researches yt-dlp

- **WHEN** an agent loads the manifest-yt-dlp skill
- **THEN** the research section instructs it to fetch the yt-dlp GitHub releases page, check PyPI for the pip package name, and verify ffmpeg dependency

### Requirement: Manifest schema section

The manifest schema section SHALL specify the YAML block structure for the capability:

- Required vs optional fields
- Default values
- Example YAML
- Validation rules

#### Scenario: Agent defines package schema

- **WHEN** an agent extends the manifest schema for a new package type
- **THEN** it follows the skill's schema section to define the struct fields and serde attributes

### Requirement: Provisioning section

The provisioning section SHALL specify:

- Install command template (or delegation to existing handler)
- Config file location and format
- Verification command (how to confirm the tool is working)
- Dependencies on other capabilities

#### Scenario: Agent provisions yt-dlp

- **WHEN** an agent loads the manifest-yt-dlp skill's provisioning section
- **THEN** it reads the install method template (pip install yt-dlp), the config file location (~/.config/yt-dlp/config), the verification command (yt-dlp --version), and the dependency list (ffmpeg)

### Requirement: Idempotency section

The idempotency section SHALL specify the `check()` logic for this capability:

- How to detect that the capability is already satisfied
- What constitutes "divergent" between declared and actual state
- What state is considered "user-managed" (no setmeup marker)

#### Scenario: Agent checks nvm idempotency

- **WHEN** an agent loads the manifest-nvm skill's idempotency section
- **THEN** it reads: check satisfied by running `nvm ls | grep lts` and verifying `~/.nvm/nvm.sh` exists; divergent if nvm is installed but the declared node version is missing

### Requirement: Wizard section

The wizard section SHALL specify what the TUI step collects:

- Input type (list toggle, free text, yes/no)
- Default values
- Validation rules

#### Scenario: Agent defines wizard input for repos

- **WHEN** an agent loads the manifest-repos skill's wizard section
- **THEN** it reads: input type is free-text with add/remove UI, default is empty, validation requires a valid git URL

### Requirement: Skill creation workflow

The process for adding a new capability SHALL be documented in `docs/ADDING-A-CAPABILITY.md`. It SHALL include:

1. Create GitHub issue describing the tool/behavior
2. Write `manifest-<name>/SKILL.md` (research the ecosystem)
3. Define the YAML schema block in `manifest.rs`
4. Implement `ProvisionHandler` in `provisioning/<name>.rs`
5. Add wizard step in `tui/steps.rs`
6. Wire into manifest + provisioning dispatch
7. Verify with the done-gate

#### Scenario: Agent follows the recipe

- **WHEN** an agent is asked to add a new capability like "oh-my-zsh"
- **THEN** it loads `docs/ADDING-A-CAPABILITY.md` and follows the steps, starting with writing the oh-my-zsh skill