# `taskwarrior-tui`

[![CI](https://github.com/kdheepak/taskwarrior-tui/workflows/CI/badge.svg)](https://github.com/kdheepak/taskwarrior-tui/actions?query=workflow%3ACI)
[![](https://img.shields.io/github/license/kdheepak/taskwarrior-tui)](./LICENSE)
[![](https://img.shields.io/github/v/release/kdheepak/taskwarrior-tui)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)
[![](https://img.shields.io/static/v1?label=platform&message=linux-32%20%7C%20linux-64%20%7C%20osx-64%20%7C%20win-32%20%7C%20win-64&color=lightgrey)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)

A Terminal User Interface for [Taskwarrior](https://taskwarrior.org/).

![](https://user-images.githubusercontent.com/1813121/97066323-acd41500-1571-11eb-90c2-d74faa21e1ad.png)

## Installation

You will need to install `taskwarrior` first. See <https://taskwarrior.org/download/> for more information.

**Manual**

1. Download the tar.gz file for your OS from [the latest release](https://github.com/kdheepak/taskwarrior-tui/releases/latest).
2. Unzip the tar.gz file
3. Run with `./taskwarrior-tui`.

**Using `brew`**

```bash
brew install taskwarrior-tui
```

**Using `cargo`**

```bash
git clone https://github.com/kdheepak/taskwarrior-tui.git
cd taskwarrior-tui
cargo build --release
```

**Using `AUR`**

Use your favorite installation method to download the [AUR package](https://aur.archlinux.org/packages/taskwarrior-tui-git/) maintained by [**@loki7990**](https://github.com/loki7990).

**Using [`zdharma/zinit`](https://github.com/zdharma/zinit)**

Add the following to your `~/.zshrc`:

```zsh
zinit ice wait:2 lucid extract"" from"gh-r" as"command" mv"taskwarrior-tui* -> tt"
zinit load kdheepak/taskwarrior-tui
```

## Usage

See it in action here:

![](https://user-images.githubusercontent.com/1813121/89620056-4ed64200-d84c-11ea-9153-9e08bc26d3b4.gif)

### Easy to use interface:

<details>

<summary> Click to expand! </summary>

- `/`: `task {string}`                       - Filter task report
- `a`: `task add {string}`                   - Add new task
- `d`: `task {selected} done`                - Mark task as done
- `e`: `task {selected} edit`                - Open selected task in editor
- `j`: `{selected+=1}`                       - Move down in task report
- `k`: `{selected-=1}`                       - Move up in task report
- `l`: `task log {string}`                   - Log new task
- `m`: `task {selected} modify {string}`     - Modify selected task
- `q`: `exit`                                - Quit
- `s`: `task {selected} start/stop`          - Toggle start and stop
- `u`: `task undo`                           - Undo
- `x`: `task delete {selected}`              - Delete
- `z`: `toggle task info`                    - Toggle task info view
- `A`: `task {selected} annotate {string}`   - Annotate current task
- `?`: `help`                                - Help menu
- `!`: `{string}`                            - Custom shell command
- `]`: `Next view`                           - Go to next view
- `[`: `Previous view`                       - Go to previous view

![](https://user-images.githubusercontent.com/1813121/88654924-40896880-d08b-11ea-8709-b29cc970da4c.gif)

</details>

### `readline`-like functionality:

<details>

<summary> Click to expand! </summary>

- `<Ctrl-a>` : Go to beginning of the line
- `<Ctrl-e>` : Go to end of the line
- `<Ctrl-u>` : Delete from beginning of the line
- `<Ctrl-k>` : Delete to end of the line
- `<Ctrl-w>` : Delete previous word

![](https://user-images.githubusercontent.com/1813121/95651612-ce7cc900-0aa8-11eb-8686-42442ed9ee43.gif)

</details>

### Calendar view

<details>

<summary> Click to expand! </summary>

`taskwarrior-tui` supports a Calendar view, which you can get to by hitting the `]` key:

![](https://user-images.githubusercontent.com/1813121/96957124-0c211f00-14b7-11eb-9d29-b3b68420af44.gif)

This highlights the days for your due tasks in a calendar view.
You can configure the number of months in a row by changing the `uda.taskwarrior-tui.calendar.months_per_row` attribute in your `taskrc` file.
See the next section for more information.

You can switch back to the task view by hitting the `[` key.

</details>

### Configure `taskwarrior-tui` using `~/.taskrc`:

<details>

<summary> Click to expand! </summary>

`taskwarrior-tui` reads values from your `taskwarrior`'s `taskrc` file (default: `~/.taskrc`).

![](https://user-images.githubusercontent.com/1813121/96684390-bf173e80-1338-11eb-971c-ae64233d142e.png)

For example, `color.active` is used to style the active task.
If you would like to try it, open your `taskrc` file and change `color.active=white on blue`.

So `color.active` will take precedence over `color.overdue`. You can see what `color.active` is by running `task show color.active` in your favorite shell prompt.

The following color attributes are supported:

```plaintext
color.deleted
color.completed
color.active
color.overdue
color.scheduled
color.due.today
color.due
color.blocked
color.blocking
color.recurring
color.tagged
```

Other `taskwarrior-tui` configuration options are possible using the user defined attribute feature of `taskwarrior`.
All `taskwarrior-tui` specific configuration options will begin with `uda.taskwarrior-tui.`.
The following is a full list of all the options available and their default values implemented by `taskwarrior-tui` if not defined in your `taskrc` file.

```plaintext
uda.taskwarrior-tui.selection.indicator=•
uda.taskwarrior-tui.selection.bold=yes
uda.taskwarrior-tui.selection.italic=no
uda.taskwarrior-tui.selection.dim=no
uda.taskwarrior-tui.selection.blink=no
uda.taskwarrior-tui.calendar.months_per_row=4
```

</details>
