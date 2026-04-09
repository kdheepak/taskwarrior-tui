---
title: Advanced Configuration
description: Configure taskwarrior-tui report behavior, styles, shortcuts, background tasks, and environment overrides.
---

`taskwarrior-tui` parses the output of `task show` to get configuration data. This allows `taskwarrior-tui` to use the same defaults as `taskwarrior` and configure additional options as required.

## `taskrc` Configuration File Options

Other `taskwarrior-tui` configuration options are possible using Taskwarrior user-defined attributes. All `taskwarrior-tui` specific configuration options begin with `uda.taskwarrior-tui.`. The following is the full list of options available and their default values if they are not defined in your `taskrc` file.

```plaintext
uda.taskwarrior-tui.selection.indicator=•
uda.taskwarrior-tui.selection.bold=yes
uda.taskwarrior-tui.selection.italic=no
uda.taskwarrior-tui.selection.dim=no
uda.taskwarrior-tui.selection.blink=no
uda.taskwarrior-tui.selection.reverse=no
uda.taskwarrior-tui.mark.indicator=✔
uda.taskwarrior-tui.unmark.indicator=
uda.taskwarrior-tui.mark-selection.indicator=⦿
uda.taskwarrior-tui.unmark-selection.indicator=⦾
uda.taskwarrior-tui.calendar.months-per-row=4
uda.taskwarrior-tui.task-report.show-info=true
uda.taskwarrior-tui.task-report.looping=true
uda.taskwarrior-tui.task-report.use-alternate-style=true
uda.taskwarrior-tui.task-report.jump-on-task-add=true
uda.taskwarrior-tui.task-report.prompt-on-undo=false
uda.taskwarrior-tui.task-report.prompt-on-delete=false
uda.taskwarrior-tui.task-report.prompt-on-done=false
uda.taskwarrior-tui.style.report.selection=
uda.taskwarrior-tui.style.context.active=black on rgb444
uda.taskwarrior-tui.style.report-menu.active=black on rgb444
uda.taskwarrior-tui.style.calendar.title=black on rgb444
uda.taskwarrior-tui.style.report.scrollbar=black
uda.taskwarrior-tui.scrollbar.indicator=█
uda.taskwarrior-tui.style.report.scrollbar.area=white
uda.taskwarrior-tui.scrollbar.area=║
uda.taskwarrior-tui.task-report.next.filter=$(task show report.next.filter)
uda.taskwarrior-tui.task-report.auto-insert-double-quotes-on-add=true
uda.taskwarrior-tui.task-report.auto-insert-double-quotes-on-annotate=true
uda.taskwarrior-tui.task-report.auto-insert-double-quotes-on-log=true
uda.taskwarrior-tui.task-report.reset-filter-on-esc=true
uda.taskwarrior-tui.context-menu.select-on-move=false
uda.taskwarrior-tui.context-menu.close-on-select=true
uda.taskwarrior-tui.report-menu.select-on-move=false
uda.taskwarrior-tui.report-menu.close-on-select=true
uda.taskwarrior-tui.tabs.change-focus-rotate=false
uda.taskwarrior-tui.quick-tag.name=next
# UI chrome styles (support all Taskwarrior color formats)
uda.taskwarrior-tui.style.title=         # default: LightCyan foreground
uda.taskwarrior-tui.style.title.border=  # default: White foreground
uda.taskwarrior-tui.style.help.gauge=    # default: Gray foreground
uda.taskwarrior-tui.style.command.error= # default: Red foreground
```

See [color configuration](./colors.md) for supported color formats and additional TUI style keys such as `uda.taskwarrior-tui.style.navbar` and `uda.taskwarrior-tui.style.command`.

The `uda.taskwarrior-tui.task-report.next.filter` variable defines the default view at program startup. Set this to any preconfigured report from `task reports`, or create your own report in Taskwarrior and specify its name here.

## Command-Line Options

`-r` specifies a report to be shown and overrides `uda.taskwarrior-tui.task-report.next.filter` for that instance.

## Configure Quick Tag

The quick-tag action toggles a single tag on the selected task. By default it uses the tag `next`, and the default keybinding is `t`.

Configure the tag name in your `taskrc` with a bare tag value:

```plaintext
uda.taskwarrior-tui.quick-tag.name=next
```

Do not include a leading `+`. `taskwarrior-tui` adds and removes the tag for you, so `next` becomes `+next` or `-next` when you trigger the quick-tag action.

## Configure User-Defined Shortcuts

You can configure shortcuts from your Taskwarrior `taskrc` file (default: `~/.taskrc`) by mapping them to executable files:

```plaintext
uda.taskwarrior-tui.shortcuts.1=~/.config/taskwarrior-tui/shortcut-scripts/add-personal-tag.sh
uda.taskwarrior-tui.shortcuts.2=~/.config/taskwarrior-tui/shortcut-scripts/sync.sh
...
```

The executable file can be placed in any location.

To make a file executable:

1. Run `chmod +x /path/to/script` to modify the executable flag.
2. Add `#!/usr/bin/env bash`, `#!/usr/bin/env python`, or whatever is appropriate for your script.

By default, keys `1`-`9` are available to run shortcuts.

When you hit the shortcut, the script will be executed with `selected_tasks_uuid` as an argument:

```bash
~/.config/taskwarrior-tui/shortcut-scripts/add-personal-tag.sh $selected_tasks_uuid
```

For example, you can add the `personal` tag to the currently selected task with the following script in `~/.config/taskwarrior-tui/shortcut-scripts/add-personal-tag.sh`:

```plaintext
task rc.bulk=0 rc.confirmation=off rc.dependency.confirmation=off rc.recurrence.confirmation=off "$@" modify +personal
```

By default, shortcuts are linked to the `1-9` number row keys. They can be customized as any other keys through `uda.taskwarrior-tui.keyconfig.shortcut1=<key>`. For example:

```plaintext
uda.taskwarrior-tui.keyconfig.shortcut1=n
```

You can set up shortcuts to run `task sync` or any custom script that you like.

## Configure One Background Task

You can configure one background task to run periodically:

```plaintext
uda.taskwarrior-tui.background_process=task sync
uda.taskwarrior-tui.background_process_period=60
```

This runs `task sync` every 60 seconds. If `background_process` is an empty string, which is the default, then no process is run. Only if `background_process` is defined and runs successfully will it continue to run every `background_process_period` seconds, which defaults to 60. If it fails even once, it will not be run again until `taskwarrior-tui` is restarted.

## Environment Variables

### `TASKWARRIOR_TUI_DATA`

Overrides the `taskwarrior-tui` data directory used for logs and history files.

By default this uses your platform's local data directory, typically `~/.local/share/taskwarrior-tui/` on Linux.

This is useful when debugging because it lets you collect `taskwarrior-tui.log` in a temporary or project-local folder:

```bash
TASKWARRIOR_TUI_DATA=/tmp/taskwarrior-tui-debug taskwarrior-tui
```

### `TASKWARRIOR_TUI_LOG_LEVEL`

Controls how much information is written to `taskwarrior-tui.log`.

Supported values are `off`, `warn`, `info`, `debug`, and `trace`.

Examples:

```bash
TASKWARRIOR_TUI_LOG_LEVEL=debug taskwarrior-tui
TASKWARRIOR_TUI_LOG_LEVEL=trace taskwarrior-tui
```

### `TASKWARRIOR_TUI_TASKWARRIOR_CLI`

Overrides the path to the Taskwarrior executable. Defaults to `task`, resolved via `PATH`.

Use this if Taskwarrior is installed in a non-standard location, or if you want to wrap the `task` binary with a script:

```bash
TASKWARRIOR_TUI_TASKWARRIOR_CLI=/usr/local/bin/task taskwarrior-tui
```

You can also set it permanently in your shell profile:

```bash
export TASKWARRIOR_TUI_TASKWARRIOR_CLI=/usr/local/bin/task
```
