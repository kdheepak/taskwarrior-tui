---
title: Troubleshooting
description: Collect logs, compare Taskwarrior commands, and isolate configuration or filesystem issues when taskwarrior-tui is slow or fails.
---

If `taskwarrior-tui` is slow, fails to start, or behaves differently than `task`, use this checklist to narrow the problem down before opening an issue.

## Capture Version and Environment Details

Start by recording the versions involved:

```bash
taskwarrior-tui --version
task --version
task diagnostics
```

If you launch `taskwarrior-tui` with custom paths or wrappers, also record:

```bash
echo "$TASKRC"
echo "$TASKDATA"
echo "$TASKWARRIOR_TUI_DATA"
echo "$TASKWARRIOR_TUI_TASKWARRIOR_CLI"
```

These environment variables can help identify when there's a bug in `taskwarrior-tui` versus a configuration or environment issue.

## Collect a Log File

`taskwarrior-tui` writes a `taskwarrior-tui.log` file to its data directory.

- By default it uses your platform's local data directory.
- Override the log location with `--data /path/to/folder` or `TASKWARRIOR_TUI_DATA=/path/to/folder`.

To enable verbose logs:

```bash
TASKWARRIOR_TUI_LOG_LEVEL=debug taskwarrior-tui
```

Use `trace` for the most verbose logs:

```bash
TASKWARRIOR_TUI_LOG_LEVEL=trace taskwarrior-tui
```

The log includes timestamps on each line for logging information in `taskwarrior-tui`.

## Compare With Plain Taskwarrior

Many issues are easier to isolate by comparing the presentation of the data in `taskwarrior-tui` with the underlying `task` commands directly. Use the same `TASKRC`, `TASKDATA`, report, and shell environment as the `taskwarrior-tui` session.

```bash
time task show
time task export next
time task context
time task summary
time task <uuid>
```

Replace `next` with the report you open in `taskwarrior-tui`, and replace `<uuid>` with a task visible in the TUI view.

## Isolate the UI Refresh Path

Try these temporary `taskrc` changes one at a time:

```plaintext
uda.taskwarrior-tui.task-report.show-info=0
uda.taskwarrior-tui.task-report.task-detail-prefetch=0
uda.taskwarrior-tui.tick-rate=0
```

What they help isolate:

- `task-report.show-info=0` disables the task details pane.
- `task-detail-prefetch=0` keeps the details pane on, but reduces prefetching while you move through tasks.
- `tick-rate=0` disables periodic refresh ticks.

Restore your normal settings after the test.

## Disable Local Customizations

If you use any of the following, test once without them:

- `TASKWARRIOR_TUI_TASKWARRIOR_CLI`
- `uda.taskwarrior-tui.background_process`
- custom shortcuts or shell commands
- a heavily customized `taskrc`

This helps separate a core bug from a wrapper script or environment-specific behavior.

## Capture Crash Information

If `taskwarrior-tui` exits with a panic, rerun it with:

```bash
RUST_BACKTRACE=1 taskwarrior-tui
```

Attach the panic output together with the relevant log excerpt.

## What to Include in a GitHub Issue

Please include:

- `taskwarrior-tui` version
- `task` version
- OS, terminal emulator, and shell
- whether the issue reproduces with a minimal `taskrc`
- whether the issue reproduces with local paths instead of mounted or network paths
- relevant log excerpts
- timings from the `time task ...` comparison
- exact steps to reproduce

Redact private paths, project names, or task descriptions if needed.
