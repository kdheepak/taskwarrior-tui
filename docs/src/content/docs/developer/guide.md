---
title: Developer Guide
description: Install the repo toolchain, run tests, work with fixture data, and build the Starlight docs site.
---

## Installing `mise`

Install `mise`:

```bash
curl https://mise.run | sh
```

Activate it in your shell:

```bash
# zsh
echo 'eval "$(mise activate zsh)"' >> "${ZDOTDIR-$HOME}/.zshrc"

# bash
echo 'eval "$(mise activate bash)"' >> ~/.bashrc

# fish
echo 'mise activate fish | source' >> ~/.config/fish/config.fish
```

Restart your shell, then install this project's tools:

```bash
mise install
```

The rest of this guide assumes `mise` is activated in your shell, so the toolchain and project environment from [`.config/mise.toml`](https://github.com/kdheepak/taskwarrior-tui/blob/main/.config/mise.toml) are available automatically. Pinned values like the Taskwarrior source tag and testdata ref live there, while the task entrypoints themselves live in [`.config/mise/tasks/taskwarrior-tui/`](https://github.com/kdheepak/taskwarrior-tui/tree/main/.config/mise/tasks/taskwarrior-tui/).

## Running Tests

```bash
git clone https://github.com/kdheepak/taskwarrior-tui
cd taskwarrior-tui

mise run taskwarrior-tui:cargo-test
```

`mise run taskwarrior-tui:cargo-test` fetches `taskwarrior-testdata` at a pinned commit for deterministic runs.

## Building the CLI

```bash
cargo build
cargo build --release
```

## Running a Debug Build

```bash
cargo run
```

## Running the TUI with Local Fixture Data

Import `tests/data/export.json` into `tests/data/.task`:

```bash
mise run taskwarrior-tui:import-taskdata
```

Run the TUI against that imported data and remove `tests/data/.task` when the TUI exits:

```bash
mise run taskwarrior-tui:run-taskdata
```

Use a release build instead:

```bash
mise run taskwarrior-tui:run-taskdata --release
```

Remove `tests/data/.task` without starting the TUI:

```bash
mise run taskwarrior-tui:clean-taskdata
```

## Running a Release Build

```bash
cargo run --release
```

## Testing an Individual Function

If you want to test the `test_taskwarrior_timing` function in `src/app.rs`:

```bash
mise run taskwarrior-tui:setup-testdata
cargo test -- app::tests::test_taskwarrior_timing --nocapture
```

## Getting Logs

With `mise` activated, `TASKWARRIOR_TUI_LOG_LEVEL=debug` is already set for this repo.

```bash
taskwarrior-tui

# OR

cargo run
```

## Contributing to Documentation

The Starlight site lives in `docs/`. Content pages live in `docs/src/content/docs/`. Packaging assets that used to live alongside the docs now live in `packaging/`.

Build the docs locally with:

```bash
mise run taskwarrior-tui:docs-build
```

For iterative docs work, install the site dependencies once and start the Astro dev server:

```bash
mise run taskwarrior-tui:docs-preview
```

Regenerate the man page with:

```bash
mise run taskwarrior-tui:man
```

When you make a PR to the repository, a preview of the documentation is rendered and a link is posted to the PR.

## Internals of `taskwarrior-tui`

`taskwarrior-tui` is a state-driven terminal user interface. Keyboard events are read asynchronously and communicated using channels. Most of the logic is implemented in `src/app.rs`. The difference between the previous state and the current state of the TUI is rendered every `Tick` by `ratatui`. The `app.draw_...` functions are responsible for rendering the UI. Actions for key presses are taken in [`app.handle_input(&mut self, input: Key)`](https://github.com/kdheepak/taskwarrior-tui/blob/f7f89cbff180f81a3b27112d676d6101b0b552d8/src/app.rs#L1893).
