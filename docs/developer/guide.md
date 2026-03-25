# Developer guide

## Installing mise

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

See the official mise install docs for other install methods such as Homebrew, apt, dnf, pacman, Scoop, and winget.

The rest of this guide assumes `mise` is activated in your shell, so the toolchain and project environment from [`.config/mise.toml`](../../.config/mise.toml) are available automatically.

## Running tests

```bash
git clone https://github.com/kdheepak/taskwarrior-tui
cd taskwarrior-tui

mise run test
```

`mise run test` fetches `taskwarrior-testdata` at a pinned commit for deterministic runs.

## Building the CLI

```bash
mise run build
mise run build --release
```

## Running debug build

```bash
mise run run
```

## Running the TUI with local fixture data

Import `tests/data/export.json` into `tests/data/.task`:

```bash
mise run import-taskdata
```

Run the TUI against that imported data and remove `tests/data/.task` when the TUI exits:

```bash
mise run run-taskdata
```

Use a release build instead:

```bash
mise run run-taskdata --release
```

Remove `tests/data/.task` without starting the TUI:

```bash
mise run clean-taskdata
```

## Running release build

```bash
mise run run --release
```

## Testing individual function

If you want to test the `test_taskwarrior_timing` function in `src/app.rs`:

```bash
mise run setup-tests
cargo test -- app::tests::test_taskwarrior_timing --nocapture
```

## Getting logs

With `mise` activated, `TASKWARRIOR_TUI_LOG_LEVEL=debug` is already set for this repo.

```bash
taskwarrior-tui

# OR

cargo run
```

## Contributing to documentation

See `docs/` folder in the repository: <https://github.com/kdheepak/taskwarrior-tui>

Build the docs locally with:

```bash
mise run docs-build
```

Regenerate the man page with:

```bash
mise run man
```

When you make a PR to the repository, a preview of the documentation is rendered and a link is posted to the PR.

## Internals of `taskwarrior-tui`

`taskwarrior-tui` is a state driven terminal user interface.
Keyboard events are read asynchronously and is communicated using channels.
Most of the logic is implemented in `src/app.rs`.
The difference between the previous state and the current state of the TUI is rendered every `Tick` by `ratatui`.
`app.draw_...` functions are responsible for rendering the UI.
Actions for key presses are taken in [`app.handle_input(&mut self, input: Key)`](https://github.com/kdheepak/taskwarrior-tui/blob/f7f89cbff180f81a3b27112d676d6101b0b552d8/src/app.rs#L1893).
