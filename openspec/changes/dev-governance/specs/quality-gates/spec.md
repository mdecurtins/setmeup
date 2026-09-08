## ADDED Requirements

### Requirement: Rust toolchain pinned
The repository SHALL pin the Rust toolchain via a `rust-toolchain.toml` to latest stable at time of writing, enabling reproducible builds without going stale.

#### Scenario: Toolchain file present
- **WHEN** a developer or agent builds the project
- **THEN** the pinned toolchain from `rust-toolchain.toml` is used

### Requirement: Rust quality gates in CI
CI SHALL enforce `cargo fmt --check`, `clippy -D warnings`, and `cargo test`, and SHALL fail the build if any gate fails.

#### Scenario: Formatting drift
- **WHEN** the codebase drifts from `rustfmt` output
- **THEN** CI fails the fmt check

#### Scenario: Clippy warning
- **WHEN** clippy emits any warning under `-D warnings`
- **THEN** CI fails

#### Scenario: Test failure
- **WHEN** any test fails
- **THEN** CI fails

### Requirement: Focused library coverage floor
CI SHALL enforce a coverage floor via `cargo-llvm-cov` on the core logical modules of the library (manifest parsing, secret backend logic, identity orchestration), where behavior is testable-away from OS specifics.

#### Scenario: Core module coverage below floor
- **WHEN** coverage on a core logical module falls below the configured floor
- **THEN** CI fails

#### Scenario: OS backend covered by integration
- **WHEN** an OS backend is exercised by integration tests on a runner that supports it
- **THEN** it is treated as covered without a unit-level numeric gate

### Requirement: No blanket crate-wide coverage number
The coverage gate SHALL NOT apply a single numeric threshold to the whole crate, to avoid gaming and false confidence on OS-specific branches.

#### Scenario: Reviewer inspects gate
- **WHEN** a reviewer checks the CI configuration
- **THEN** the coverage gate targets core logical modules, not a whole-crate blanket percentage

### Requirement: Dependency governance via cargo deny/audit
CI SHALL gate on dependency advisories and license/source hygiene via `cargo deny` (as primary) or `cargo audit` (as complement), and SHALL fail when a known advisory is present.

#### Scenario: Advisory introduced
- **WHEN** a dependency update brings a known-advisory version
- **THEN** the gate fails and the advisory must be addressed

#### Scenario: License/source concern
- **WHEN** a dependency introduces a disallowed license or unvetted source
- **THEN** the deny gate flags it and the repository's policy is applied deliberately

### Requirement: Exact dependency pinning
The repository SHALL pin exact dependency versions (no caret/tilde ranges) for application dependencies, per engineering principles.

#### Scenario: Cargo.toml version
- **WHEN** a dependency is added
- **THEN** its version requirement is exact, not a range

### Requirement: Shell quality gates
Any shell script in the repository SHALL pass `shellcheck` and `shfmt --check` in CI.

#### Scenario: Bootstrap passes gates
- **WHEN** `bootstrap.sh` is present
- **THEN** CI runs shellcheck and shfmt --check on it and fails on violations

#### Scenario: Shell added later
- **WHEN** a shell script is added to the repository
- **THEN** it is subject to the same shellcheck and shfmt gates