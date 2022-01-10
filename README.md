# `taskwarrior-tui`

[![CI](https://github.com/kdheepak/taskwarrior-tui/workflows/CI/badge.svg)](https://github.com/kdheepak/taskwarrior-tui/actions?query=workflow%3ACI)
[![](https://img.shields.io/github/license/kdheepak/taskwarrior-tui)](./LICENSE)
[![](https://img.shields.io/github/v/release/kdheepak/taskwarrior-tui)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)
[![](https://img.shields.io/static/v1?label=platform&message=linux-64%20|%20osx-64%20|%20win-32%20|%20win-64&color=lightgrey)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)
[![](https://img.shields.io/github/languages/top/kdheepak/taskwarrior-tui)](https://github.com/kdheepak/taskwarrior-tui)
[![](https://img.shields.io/coveralls/github/kdheepak/taskwarrior-tui)](https://coveralls.io/github/kdheepak/taskwarrior-tui)
[![](https://img.shields.io/badge/taskwarrior--tui-docs-red)](https://kdheepak.com/taskwarrior-tui)

A Terminal User Interface for [Taskwarrior](https://taskwarrior.org/).

![](https://user-images.githubusercontent.com/1813121/113252474-21c61c00-9281-11eb-8292-bf6a3553251e.png)

Showcase of features: <https://youtu.be/0ZdkfNrIAcw>

[![](https://img.youtube.com/vi/0ZdkfNrIAcw/0.jpg)](https://www.youtube.com/watch?v=0ZdkfNrIAcw)

**User Interface**

![](https://user-images.githubusercontent.com/1813121/113251568-bdef2380-927f-11eb-8cb6-5d95b00eee53.gif)

**Multiple selection**

![](https://user-images.githubusercontent.com/1813121/113252636-4e7a3380-9281-11eb-821d-874c86d11105.gif)

**Tab completion**

![](https://user-images.githubusercontent.com/1813121/113711977-cfcb2f00-96a2-11eb-8b06-9fd17903561d.gif)

### Documentation

See <https://kdheepak.com/taskwarrior-tui> for documentation.

See <https://kdheepak.com/taskwarrior-tui/installation/> for installation instructions for your platform.

See <https://kdheepak.com/taskwarrior-tui/quick_start/> to get started.

See <https://kdheepak.com/taskwarrior-tui/configuration/> for customization options.

### Installation

Unless otherwise specified, you will need to install the latest version of `taskwarrior` first. See <https://taskwarrior.org/download/> for more information.

**Manual** [![](https://img.shields.io/github/v/tag/kdheepak/taskwarrior-tui)](https://github.com/kdheepak/taskwarrior-tui/releases/latest) [![](https://img.shields.io/github/downloads/kdheepak/taskwarrior-tui/total)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)

1. Download the tar.gz file for your OS from [the latest release](https://github.com/kdheepak/taskwarrior-tui/releases/latest).
2. Unzip the tar.gz file
3. Run with `./taskwarrior-tui`.

See <https://kdheepak.com/taskwarrior-tui/installation/> on instructions for using package managers on various platforms.

If you are compiling from source, you'll need to most recent stable rust compiler.

### Configuration

`taskwarrior-tui` uses `taskwarrior`'s `.taskrc` for configuration.

Here is an example `.taskrc`

```taskwarrior
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
uda.taskwarrior-tui.shortcuts.0=~/local/bin/task-sync.sh
uda.taskwarrior-tui.report.next.filter=(status:pending or status:waiting)
```

`taskwarrior-tui` parses the output of `task show` to get configuration data.
This allows `taskwarrior-tui` to use the same defaults as `taskwarrior` and configure additional options as required.

See the documentation for more information:

- <https://kdheepak.com/taskwarrior-tui/configuration/keys>
- <https://kdheepak.com/taskwarrior-tui/configuration/colors>
- <https://kdheepak.com/taskwarrior-tui/configuration/advanced/>

## References / Resources

If you like `taskwarrior-tui`, please consider donating to [@GothenburgBitFactory](https://github.com/sponsors/GothenburgBitFactory).

- <https://github.com/GothenburgBitFactory/taskwarrior>
- <https://github.com/GothenburgBitFactory/libshared>
- <https://github.com/GothenburgBitFactory/timewarrior>
- <https://github.com/fdehau/tui-rs>
- <https://github.com/crossterm-rs/crossterm/>
- <https://github.com/async-rs/async-std>
- <https://github.com/kkawakam/rustyline>
- <https://github.com/vit-project/vit>
- <https://github.com/taskchampion/taskchampion/>
