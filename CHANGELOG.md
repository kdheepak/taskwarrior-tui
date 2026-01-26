# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.26.6](https://github.com/kdheepak/taskwarrior-tui/compare/v0.26.5...v0.26.6) - 2026-01-26

### Fixed

- Apply color.uda.* configuration for all UDAs ([#652](https://github.com/kdheepak/taskwarrior-tui/pull/652))

### Other

- *(deps)* bump clechasseur/rs-cargo from 2 to 4 ([#665](https://github.com/kdheepak/taskwarrior-tui/pull/665))
- *(deps)* bump unicode-truncate from 2.0.0 to 2.0.1 in the cargo-dependencies group ([#663](https://github.com/kdheepak/taskwarrior-tui/pull/663))
- *(deps)* bump dawidd6/action-homebrew-bump-formula from 3 to 7 ([#662](https://github.com/kdheepak/taskwarrior-tui/pull/662))
- *(deps)* bump actions/github-script from 7 to 8 ([#661](https://github.com/kdheepak/taskwarrior-tui/pull/661))
- *(deps)* bump amannn/action-semantic-pull-request from 5 to 6 ([#660](https://github.com/kdheepak/taskwarrior-tui/pull/660))
- *(deps)* bump actions/setup-python from 5 to 6 ([#658](https://github.com/kdheepak/taskwarrior-tui/pull/658))
- *(deps)* bump actions/checkout from 4 to 6 ([#659](https://github.com/kdheepak/taskwarrior-tui/pull/659))
- *(deps)* bump the cargo-dependencies group across 1 directory with 21 updates ([#657](https://github.com/kdheepak/taskwarrior-tui/pull/657))
- *(deps)* bump actions/upload-artifact from 4 to 6 ([#576](https://github.com/kdheepak/taskwarrior-tui/pull/576))
- update ratatui to `0.30.0` ([#655](https://github.com/kdheepak/taskwarrior-tui/pull/655))

## [0.26.5](https://github.com/kdheepak/taskwarrior-tui/compare/v0.26.4...v0.26.5) - 2025-12-18

### Added

- add desktop entry under docs ([#650](https://github.com/kdheepak/taskwarrior-tui/pull/650))
- help screen shows user keybindings ([#645](https://github.com/kdheepak/taskwarrior-tui/pull/645))

### Fixed

- Fixes duplicate key bug when assigning key to `edit` ([#644](https://github.com/kdheepak/taskwarrior-tui/pull/644))
- README.md typo ([#640](https://github.com/kdheepak/taskwarrior-tui/pull/640))

### Other

- *(setup)* add just recipes for local test data ([#646](https://github.com/kdheepak/taskwarrior-tui/pull/646))

## [0.26.4](https://github.com/kdheepak/taskwarrior-tui/compare/v0.26.3...v0.26.4) - 2024-11-14

### Added

- Add code coverage to CI
- Cache taskwarrior compilation
- Build taskwarrior CI against stable

### Fixed

- Disable broken MacOS compression
- Append target for unique artifact names
- Update the upload-artifacts due to deprecation of v2
- Deprecation warning

### Other

- Add us as Co-Maintainers ([#606](https://github.com/kdheepak/taskwarrior-tui/pull/606))
- Apply clippy lint
- Merge build workflows
- Modernize CI/CD components
- Use config for selection mark/unmark symbols ([#594](https://github.com/kdheepak/taskwarrior-tui/pull/594))
- *(deps)* bump the cargo-dependencies group with 4 updates ([#584](https://github.com/kdheepak/taskwarrior-tui/pull/584))
- *(deps)* bump tokio from 1.37.0 to 1.38.0 in the cargo-dependencies group ([#582](https://github.com/kdheepak/taskwarrior-tui/pull/582))
- *(deps)* bump the cargo-dependencies group with 3 updates ([#580](https://github.com/kdheepak/taskwarrior-tui/pull/580))
- Update taskwarrior-tui.bash
- *(deps)* bump the cargo-dependencies group across 1 directory with 20 updates ([#573](https://github.com/kdheepak/taskwarrior-tui/pull/573))
- *(deps)* Bump to ratatui v0.26
- *(deps)* bump actions/checkout from 2 to 4 ([#569](https://github.com/kdheepak/taskwarrior-tui/pull/569))
- *(deps)* bump peaceiris/actions-gh-pages from 3 to 4 ([#568](https://github.com/kdheepak/taskwarrior-tui/pull/568))
- *(deps)* bump actions/setup-python from 1 to 5 ([#566](https://github.com/kdheepak/taskwarrior-tui/pull/566))
- *(deps)* bump actions/setup-python from 1 to 5
- Add dependabot.yml

## [0.26.3](https://github.com/kdheepak/taskwarrior-tui/compare/v0.26.2...v0.26.3) - 2024-05-12

### Other
- Update cd.yml

## [0.26.2](https://github.com/kdheepak/taskwarrior-tui/compare/v0.26.1...v0.26.2) - 2024-05-12

### Added
- Add task duplicate
- Add scheduled
- Add scheduled countdown
- Add recur to autocomplete options

### Other
- fix clippy issues
- Update release-plz.yml with token

## [0.26.1](https://github.com/kdheepak/taskwarrior-tui/compare/v0.26.0...v0.26.1) - 2024-05-12

### Other
- Remove snap, appimage and crates.io
