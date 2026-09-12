# Adding a Capability to setmeup

This document describes the workflow for extending setmeup with a new
provisioning capability (e.g., nvm, oh-my-zsh, pyenv, docker). Every new
capability follows the same pattern: skill → schema → handler → wizard → verify.

## Prerequisites

You should be familiar with:
- setmeup's manifest schema (flat sections, strongly typed)
- The `ProvisionHandler` trait (check before provision)
- The linear wizard architecture (crossterm, 6 steps)
- This repo's agentic development workflow (OpenSpec, skills, done-gate)

## Step-by-Step Recipe

### 1. Create a GitHub issue

Create an issue describing the tool or behavior you want setmeup to manage.
Use the `enhancement` label. The issue should capture:

- What tool/behavior needs to be managed
- Why it's worth modeling as a first-class capability
- Any existing manual setup steps you currently follow

### 2. Research the ecosystem

Before writing any code, understand the tool's installation and configuration
ecosystem. Load the existing skill for a similar capability as a reference, or
research from scratch:

- **Installation**: What are the install methods? (apt, pip, download script,
  manual binary?) Are there prerequisites (curl, git, build tools)?
- **Configuration**: Where do config files live? What format? Can multiple
  config sources be merged?
- **Idempotency**: How do you check if it's already installed? How do you
  verify the config is correct?
- **Latest version**: Where is the canonical version source? (GitHub releases,
  PyPI, npm registry?)

### 3. Write the skill

Create `.opencode/skills/manifest-<name>/SKILL.md` with these sections:

```markdown
# Research
- Official docs: <URL>
- Latest version: <source>
- Install methods: <list>

# Manifest schema
```yaml
# Example YAML block
<name>:
  option: value
```

# Provisioning
- Install: <command template>
- Config: <path and format>
- Verify: <command>

# Idempotency
- Check: <how to detect satisfied state>
- Divergent: <what constitutes a difference>

# Wizard
- Step: <which wizard step this belongs in (Packages, Shell, etc.)>
- Input: <list toggle, free text, yes/no>
- Defaults: <default values>
- Validation: <rules>
```

### 4. Define the manifest schema

In `src/manifest.rs`:

1. Define a new struct for the capability's config (serde, kebab-case)
2. Add a field to `Manifest` with `#[serde(default)]`
3. Add validation to `Manifest::validate()`
4. Add unit tests for parse + validation

### 5. Implement the handler

Create `src/provisioning/<name>.rs`:

```rust
use crate::error::Result;
use crate::manifest::Os;
use super::{CheckResult, ProvisionContext, ProvisionHandler, ProvisionResult};

#[derive(Debug)]
pub struct NameHandler {
    config: NameConfig,
}

impl ProvisionHandler for NameHandler {
    fn section_name(&self) -> &'static str { "<name>" }

    fn check(&self, os: Os, ctx: &ProvisionContext) -> Result<CheckResult> {
        // Return Satisfied if already done, NeedsProvision if not,
        // Divergent if partially done, ManagedByUser if user owns the state.
    }

    fn provision(&self, os: Os, ctx: &ProvisionContext) -> Result<Vec<ProvisionResult>> {
        // Install, configure, verify. Return per-item results.
    }
}
```

### 6. Wire the handler

In `src/provisioning.rs`:

1. Add the handler module: `pub mod <name>;`
2. Add a variant in `capability_handlers()` when the manifest section is present
3. Slot it into the correct position in the provisioning order

### 7. Add the wizard step

In `src/tui/steps.rs`:

1. If the new capability needs user input during `configure`, add or extend a
   wizard step
2. Follow the existing step pattern: clear screen → progress bar → content →
   input → validate → advance
3. Wire it into the wizard orchestrator's step sequence

### 8. Verify

Run the done-gate:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
openspec validate
```

### 9. Ship

1. Commit with Conventional Commits: `feat(manifest): add <name> capability`
2. Open a PR referencing both the change and the issue
3. The PR review gate checks: skill accuracy, handler correctness, idempotency
   test, no-clobber contract, spec alignment

## Checklist

- [ ] Issue created (or task references existing issue)
- [ ] Skill written (`.opencode/skills/manifest-<name>/SKILL.md`)
- [ ] Schema defined (`manifest.rs`: struct + field + validation + tests)
- [ ] Handler implemented (`provisioning/<name>.rs`: check + provision + tests)
- [ ] Handler wired into dispatch (`provisioning.rs`: module + capability_handlers)
- [ ] Wizard step updated (`tui/steps.rs`)
- [ ] Spec updated (`openspec/specs/<name>/spec.md`)
- [ ] Done-gate green (fmt, clippy, test, validate)
- [ ] Conventional commit with scope

## Adding a capability that has no install step

Some capabilities only need configuration, not installation (e.g., a shell
function, an environment variable, an alias). For these:

1. Skip the install step in `provision()` — just configure in place
2. `check()` verifies the config exists and matches
3. The handler can be a sub-section of an existing handler (e.g., shell
  functions live in `ShellHandler`, not their own)

## Adding a capability that is a wrapper around another

If the new capability is syntactic sugar over an existing one (e.g., a compound
"set up node" that does nvm + npmrc + global tools), implement it by composing
existing handlers:

```rust
fn provision(&self, os: Os, ctx: &ProvisionContext) -> Result<Vec<ProvisionResult>> {
    let mut results = Vec::new();
    // Delegate to existing handlers
    results.extend(packages::provision_yt_dlp(os, ctx)?);
    results.extend(configs::write_yt_dlp_config(os, ctx)?);
    Ok(results)
}
```
