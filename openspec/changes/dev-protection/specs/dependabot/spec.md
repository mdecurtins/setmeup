## ADDED Requirements

### Requirement: Dependabot dependency updates
The repository SHALL enable Dependabot with a monthly cadence and grouped dependency PRs to reduce update noise.

#### Scenario: Dependency update available
- **WHEN** an enabled dependency has an available update and the monthly window opens
- **THEN** Dependabot opens a grouped PR with the update

#### Scenario: Update PR merged
- **WHEN** a Dependabot PR passes CI and review
- **THEN** it merges and the dependency is updated