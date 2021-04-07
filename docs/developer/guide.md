# Developer guide

## Running tests

```
git clone https://github.com/kdheepak/taskwarrior-tui
cd taskwarrior-tui

git clone github.com/kdheepak/taskwarrior-testdata tests/data
source .env

cargo test
```

## Running debug build

```
cargo run
```

## Running release build

```
cargo run --release
```

## Testing individual function

If you want to test the `test_taskwarrior_timing` function in `src/app.rs`:

```
cargo test -- app::tests::test_taskwarrior_timing --nocapture
```

## Contributing to documentation

See `docs/` folder in the repository: <https://github.com/kdheepak/taskwarrior-tui>

When you make a PR to the repository, a preview of the documentation is rendered and a link is posted to the PR.

## Internals of `taskwarrior-tui`

`taskwarrior-tui` is a state driven terminal user interface.
Keyboard events are read asynchronously and is communicated using channels.
Most of the logic is implemented in `src/app.rs`.
The difference between the previous state and the current state of the TUI is rendered every `Tick` by `tui-rs`.
`app.draw_...` functions are responsible for rendering the UI.
Actions for key presses are taken in [`app.handle_input(&mut self, input: Key)`](https://github.com/kdheepak/taskwarrior-tui/blob/f7f89cbff180f81a3b27112d676d6101b0b552d8/src/app.rs#L1893).
