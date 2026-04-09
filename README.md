# `taskwarrior-tui`

> [!IMPORTANT]
> [`taskwarrior` v3.x](https://github.com/GothenburgBitFactory/taskwarrior/releases/tag/v3.0.0) may break `taskwarrior-tui` features in unexpected ways. Please file a bug report if you encounter a bug.
>
> taskwarrior-tui [v0.25.4](https://github.com/kdheepak/taskwarrior-tui/releases/tag/v0.25.4) is the last version supporting taskwarrior v2.x as backend.

[![CI](https://github.com/kdheepak/taskwarrior-tui/workflows/CI/badge.svg)](https://github.com/kdheepak/taskwarrior-tui/actions?query=workflow%3ACI)
[![License](https://img.shields.io/github/license/kdheepak/taskwarrior-tui)](https://github.com/kdheepak/taskwarrior-tui/blob/main/LICENSE)
[![Release](https://img.shields.io/github/v/release/kdheepak/taskwarrior-tui)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)
[![Platform](https://img.shields.io/static/v1?label=platform&message=linux-64%20|%20osx-64%20|%20win-32%20|%20win-64&color=lightgrey)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)
[![Rust](https://img.shields.io/github/languages/top/kdheepak/taskwarrior-tui)](https://github.com/kdheepak/taskwarrior-tui)
[![Coverage](https://img.shields.io/coveralls/github/kdheepak/taskwarrior-tui)](https://coveralls.io/github/kdheepak/taskwarrior-tui)
[![Docs](https://img.shields.io/badge/taskwarrior--tui-docs-red)](https://kdheepak.com/taskwarrior-tui)
[![Downloads](https://img.shields.io/github/downloads/kdheepak/taskwarrior-tui/total)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)

A Terminal User Interface (TUI) for [Taskwarrior](https://taskwarrior.org/) that you didn't know you wanted.

### Features

- vim-like navigation
- live filter updates
- add, delete, complete, log tasks
- multiple selection
- tab completion
- colors based on taskwarrior

https://github.com/user-attachments/assets/a3c4f79b-3967-4904-b614-8bbd500c54c2

### Documentation

<details>
<summary>See <a href="https://kdheepak.com/taskwarrior-tui"
class="uri">https://kdheepak.com/taskwarrior-tui</a> for
documentation.</summary>
<p>See <a href="https://kdheepak.com/taskwarrior-tui/installation/"
class="uri">https://kdheepak.com/taskwarrior-tui/installation/</a> for
installation instructions for your platform.</p>
<p>See <a href="https://kdheepak.com/taskwarrior-tui/quick_start/"
class="uri">https://kdheepak.com/taskwarrior-tui/quick_start/</a> to get
started.</p>
<p>See <a href="https://kdheepak.com/taskwarrior-tui/troubleshooting/"
class="uri">https://kdheepak.com/taskwarrior-tui/troubleshooting/</a> for
logs, timing checks, and debugging steps.</p>
<p>See <a href="https://kdheepak.com/taskwarrior-tui/configuration/keys"
class="uri">https://kdheepak.com/taskwarrior-tui/configuration/keys</a>
or <a href="https://kdheepak.com/taskwarrior-tui/configuration/colors/"
class="uri">https://kdheepak.com/taskwarrior-tui/configuration/colors/</a>
for customization options.</p>
</details>

### Installation

Unless otherwise specified, you will need to install the latest version of `taskwarrior` first. See <https://taskwarrior.org/download/> for more information.

Pre-compiled releases are available on the [GitHub repo](https://github.com/kdheepak/taskwarrior-tui):

1. Download the tar.gz file for your OS from [the latest release](https://github.com/kdheepak/taskwarrior-tui/releases/latest).
2. Unzip the tar.gz file
3. Run with `./taskwarrior-tui`.

See <https://kdheepak.com/taskwarrior-tui/installation/> on instructions for using package managers on various platforms.

If you are compiling from source, you'll need the most recent stable rust compiler.

### Configuration

`taskwarrior-tui` uses `taskwarrior`'s `.taskrc` for configuration.

See the documentation for more information:

- <https://kdheepak.com/taskwarrior-tui/configuration/keys>
- <https://kdheepak.com/taskwarrior-tui/configuration/colors>
- <https://kdheepak.com/taskwarrior-tui/configuration/advanced/>

<details>
<summary>Here is an example `.taskrc`</summary>

```.taskrc
### taskwarrior configuration options

# taskwarrior's configuration
data.location=.task
verbose=affected,blank,context,edit,header,footnote,label,new-id,project,special,sync,recur
uda.priority.values=H,M,,L
color.alternate=

# taskwarrior-tui reads color attributes from the following to display the same colors of tasks as the CLI
color.tagged=black on rgb444

# Remove age, tags from task next report.
# taskwarrior-tui reads the labels and columns from these options to display tasks the same way taskwarrior does
report.next.labels=ID,Active,Age,Deps,P,Project,Tag,Recur,S,Due,Until,Description,Urg
report.next.columns=id,start.age,entry.age,depends,priority,project,tags,recur,scheduled.countdown,due.relative,until.remaining,description.truncated_count,urgency
report.next.filter=(status:pending or status:waiting) page:limit

### taskwarrior-tui configuration options

uda.taskwarrior-tui.keyconfig.done=x
uda.taskwarrior-tui.keyconfig.delete=d
uda.taskwarrior-tui.task-report.use-alternate-style=false
uda.taskwarrior-tui.shortcuts.1=~/local/bin/task-sync.sh
uda.taskwarrior-tui.report.next.filter=(status:pending or status:waiting)
```

</details>

### References / Resources

If you like `taskwarrior-tui`, please consider donating to

- [`kdheepak`](https://github.com/sponsors/kdheepak)
- [`@GothenburgBitFactory`](https://github.com/sponsors/GothenburgBitFactory)
- and/or a charity of your choice.

<details>
<summary>Additional resources</summary>
<ul>
<li><a href="https://github.com/GothenburgBitFactory/taskwarrior"
class="uri">https://github.com/GothenburgBitFactory/taskwarrior</a></li>
<li><a href="https://github.com/GothenburgBitFactory/libshared"
class="uri">https://github.com/GothenburgBitFactory/libshared</a></li>
<li><a href="https://github.com/GothenburgBitFactory/timewarrior"
class="uri">https://github.com/GothenburgBitFactory/timewarrior</a></li>
<li><a href="https://github.com/fdehau/tui-rs"
class="uri">https://github.com/fdehau/tui-rs</a></li>
<li><a href="https://github.com/tui-rs-revival/ratatui"
class="uri">https://github.com/tui-rs-revival/ratatui</a></li>
<li><a href="https://github.com/crossterm-rs/crossterm/"
class="uri">https://github.com/crossterm-rs/crossterm/</a></li>
<li><a href="https://github.com/async-rs/async-std"
class="uri">https://github.com/async-rs/async-std</a></li>
<li><a href="https://github.com/kkawakam/rustyline"
class="uri">https://github.com/kkawakam/rustyline</a></li>
<li><a href="https://github.com/vit-project/vit"
class="uri">https://github.com/vit-project/vit</a></li>
<li><a href="https://github.com/taskchampion/taskchampion/"
class="uri">https://github.com/taskchampion/taskchampion/</a></li>
</ul>
</details>
