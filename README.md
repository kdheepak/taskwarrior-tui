# `taskwarrior-tui`

[![CI](https://github.com/kdheepak/taskwarrior-tui/workflows/CI/badge.svg)](https://github.com/kdheepak/taskwarrior-tui/actions?query=workflow%3ACI)
[![](https://img.shields.io/github/license/kdheepak/taskwarrior-tui)](./LICENSE)
[![](https://img.shields.io/github/v/release/kdheepak/taskwarrior-tui)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)
[![](https://img.shields.io/static/v1?label=platform&message=linux-64%20|%20osx-64%20|%20win-32%20|%20win-64&color=lightgrey)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)
[![](https://img.shields.io/github/languages/top/kdheepak/taskwarrior-tui)](https://github.com/kdheepak/taskwarrior-tui)
[![](https://img.shields.io/coveralls/github/kdheepak/taskwarrior-tui)](https://coveralls.io/github/kdheepak/taskwarrior-tui)

A Terminal User Interface for [Taskwarrior](https://taskwarrior.org/).

![](https://user-images.githubusercontent.com/1813121/97066323-acd41500-1571-11eb-90c2-d74faa21e1ad.png)

## Installation

Unless otherwise specified, you will need to install `taskwarrior` first. See <https://taskwarrior.org/download/> for more information.

**Manual** ( _Recommended_ ) [![](https://img.shields.io/github/v/tag/kdheepak/taskwarrior-tui)](https://github.com/kdheepak/taskwarrior-tui/releases/latest) [![](https://img.shields.io/github/downloads/kdheepak/taskwarrior-tui/total)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)

1. Download the tar.gz file for your OS from [the latest release](https://github.com/kdheepak/taskwarrior-tui/releases/latest).
2. Unzip the tar.gz file
3. Run with `./taskwarrior-tui`.

**Install from source** [![](https://img.shields.io/badge/branch-master-red)](https://github.com/kdheepak/taskwarrior-tui)

```bash
git clone https://github.com/kdheepak/taskwarrior-tui.git
cd taskwarrior-tui
cargo build --release
```

**Using [`brew`](https://brew.sh/)** [![](https://img.shields.io/homebrew/v/taskwarrior-tui)](https://formulae.brew.sh/formula/taskwarrior-tui)

This installs `task` from `homebrew` as well.

```bash
brew install taskwarrior-tui
```

**Installation for Arch Linux** [![](https://img.shields.io/archlinux/v/community/x86_64/taskwarrior-tui)](https://archlinux.org/packages/community/x86_64/taskwarrior-tui/) [![](https://img.shields.io/aur/version/taskwarrior-tui-git)](https://aur.archlinux.org/packages/taskwarrior-tui-git/)

Use [pacman](https://wiki.archlinux.org/index.php/Pacman) to install it from the [community repository](https://archlinux.org/packages/community/x86_64/taskwarrior-tui/):

```bash
pacman -S taskwarrior-tui
```

Or use your favorite [AUR helper](https://wiki.archlinux.org/index.php/AUR_helpers) to download the [git](https://aur.archlinux.org/packages/taskwarrior-tui-git/) package maintained by [**@loki7990**](https://github.com/loki7990). For example:

```bash
yay -S taskwarrior-tui-git # build from source master
```

**Using [`snap`](https://snapcraft.io/)** [![](https://snapcraft.io/taskwarrior-tui/badge.svg)](https://snapcraft.io/taskwarrior-tui)

```bash
snap install taskwarrior-tui
```

<!--
**Using [`cargo`](https://crates.io/)** [![](https://img.shields.io/crates/v/taskwarrior-tui)](https://libraries.io/cargo/taskwarrior-tui)

```bash
cargo install taskwarrior-tui
```
-->

**Using [`zdharma/zinit`](https://github.com/zdharma/zinit)** [![](https://img.shields.io/github/v/tag/kdheepak/taskwarrior-tui)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)

Add the following to your `~/.zshrc`:

```zsh
zinit ice wait:2 lucid extract"" from"gh-r" as"command" mv"taskwarrior-tui* -> tt"
zinit load kdheepak/taskwarrior-tui
```

## Usage

_Tip_: Alias `tt` to `taskwarrior-tui`.

Add the following to your dotfiles (e.g. `~/.bashrc`, `~/.zshrc`):

```
alias tt="taskwarrior-tui"
```

### See it in action:

<details>

<summary> Click to expand! </summary>

![](https://user-images.githubusercontent.com/1813121/89620056-4ed64200-d84c-11ea-9153-9e08bc26d3b4.gif)

</details>

### Easy to use interface:

<details>

<summary> Click to expand! </summary>

See [KEYBINDINGS.md](./KEYBINDINGS.md) for full list.

![](https://user-images.githubusercontent.com/1813121/88654924-40896880-d08b-11ea-8709-b29cc970da4c.gif)

</details>

### Context switcher:

<details>

<summary> Click to expand! </summary>

![](https://user-images.githubusercontent.com/1813121/97959948-a746ae00-1d6d-11eb-9ffe-8f76f2a2b32d.gif)

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
You can configure the number of months in a row by changing the `uda.taskwarrior-tui.calendar.months-per-row` attribute in your `taskrc` file.
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
uda.taskwarrior-tui.calendar.months-per-row=4
uda.taskwarrior-tui.task-report.show-info=true
uda.taskwarrior-tui.task-report.looping=true
uda.taskwarrior-tui.style.context.active=black on rgb444
uda.taskwarrior-tui.style.calendar.title=black on rgb444
```

</details>

# Related

For a similar effort, check out `vit`:

- `vit`: <https://github.com/vit-project/vit>
