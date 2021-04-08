# Advanced configuration

Other `taskwarrior-tui` configuration options are possible using the user defined attribute feature of `taskwarrior`.
All `taskwarrior-tui` specific configuration options will begin with `uda.taskwarrior-tui.`.
The following is a full list of all the options available and their default values implemented by `taskwarrior-tui` if not defined in your `taskrc` file.

```plaintext
uda.taskwarrior-tui.selection.indicator=•
uda.taskwarrior-tui.selection.bold=yes
uda.taskwarrior-tui.selection.italic=no
uda.taskwarrior-tui.selection.dim=no
uda.taskwarrior-tui.selection.blink=no
uda.taskwarrior-tui.mark.indicator=✔
uda.taskwarrior-tui.unmark.indicator= 
uda.taskwarrior-tui.calendar.months-per-row=4
uda.taskwarrior-tui.task-report.show-info=true
uda.taskwarrior-tui.task-report.looping=true
uda.taskwarrior-tui.task-report.jump-on-task-add=true
uda.taskwarrior-tui.style.context.active=black on rgb444
uda.taskwarrior-tui.style.calendar.title=black on rgb444
```

## Configure user defined shortcuts:

You can configure shortcuts to execute custom commands from your `taskwarrior`'s `taskrc` file (default: `~/.taskrc`).
You can do this by mapping a shortcut to an executable file:

```plaintext
uda.taskwarrior-tui.shortcuts.1=~/.config/taskwarrior-tui/shortcut-scripts/add-personal-tag.sh
uda.taskwarrior-tui.shortcuts.2=~/.config/taskwarrior-tui/shortcut-scripts/sync.sh
...
```

The file can have any name in any location, but must be executable.
By default, keys `1`-`9` are available to run shortcuts.

When you hit the shortcut, the script will be executed with the `selected_task_uuid` as an argument:

```bash
~/.config/taskwarrior-tui/shortcut-scripts/add-personal-tag.sh $selected_tasks_uuid
```

For example, you can add the `personal` tag to the currently selected task with the following script in `~/.config/taskwarrior-tui/shortcut-scripts/add-personal-tag.sh` :

```plaintext
task rc.bulk=0 rc.confirmation=off rc.dependency.confirmation=off rc.recurrence.confirmation=off "$@" modify +personal
```

By default, shortcuts are linked to the `1-9` number row keys.
They can be customized as any other keys through `uda.taskwarrior-tui.keyconfig.shortcut1=<key>`.
For example:

```plaintext
uda.taskwarrior-tui.keyconfig.shortcut1=n
```

You can set up shortcuts to run `task sync` or any custom bash script that you'd like.
