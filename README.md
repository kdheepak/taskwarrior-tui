# `taskwarrior-tui`

[![CI](https://github.com/kdheepak/taskwarrior-tui/workflows/CI/badge.svg)](https://github.com/kdheepak/taskwarrior-tui/actions?query=workflow%3ACI)
[![](https://img.shields.io/github/license/kdheepak/taskwarrior-tui)](./LICENSE)
[![](https://img.shields.io/github/v/release/kdheepak/taskwarrior-tui)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)
[![](https://img.shields.io/static/v1?label=platform&message=linux-64%20|%20osx-64%20|%20win-32%20|%20win-64&color=lightgrey)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)
[![](https://img.shields.io/github/languages/top/kdheepak/taskwarrior-tui)](https://github.com/kdheepak/taskwarrior-tui)
[![](https://img.shields.io/coveralls/github/kdheepak/taskwarrior-tui)](https://coveralls.io/github/kdheepak/taskwarrior-tui)
[![](https://img.shields.io/badge/taskwarrior--tui-docs-red)](https://kdheepak.com/taskwarrior-tui)
[![](https://img.shields.io/github/downloads/kdheepak/taskwarrior-tui/total)](https://github.com/kdheepak/taskwarrior-tui/releases/latest)

A Terminal User Interface (TUI) for [Taskwarrior](https://taskwarrior.org/) that you didn't know you wanted.

### Features

- vim-like navigation
- live filter updates
- add, delete, complete, log tasks
- multiple selection
- tab completion
- colors based on taskwarrior

![](https://user-images.githubusercontent.com/1813121/113252474-21c61c00-9281-11eb-8292-bf6a3553251e.png)

### Showcase

<details>
<summary><b>Demo</b>: (video)</summary>
<a href="https://www.youtube.com/watch?v=0ZdkfNrIAcw"><img src="https://img.youtube.com/vi/0ZdkfNrIAcw/0.jpg" /></a>
</details>

<details>
  <summary><b>User Interface</b>: (gif)</summary>
  <img src="https://user-images.githubusercontent.com/1813121/113251568-bdef2380-927f-11eb-8cb6-5d95b00eee53.gif"></img>
</details>

<details>
  <summary><b>Multiple selection</b>: (gif)</summary>
  <img src="https://user-images.githubusercontent.com/1813121/113252636-4e7a3380-9281-11eb-821d-874c86d11105.gif"></img>
</details>

<details>
  <summary><b>Tab completion</b>: (gif)</summary>
  <img src="https://user-images.githubusercontent.com/1813121/113711977-cfcb2f00-96a2-11eb-8b06-9fd17903561d.gif"></img>
  <img src="https://user-images.githubusercontent.com/1813121/152730495-f0abd6b9-d710-44e6-a7f9-c15a68cc8233.png"></img>
  <img src="https://user-images.githubusercontent.com/1813121/152730497-44ce00d1-3a7c-4658-80d1-4df8d161cab8.png"></img>
  <img src="https://user-images.githubusercontent.com/1813121/152730498-cd75efed-d2c0-48e6-b82f-594e0a2a5dff.png"></img>
  <img src="https://user-images.githubusercontent.com/1813121/152731028-7ec9b388-37f6-4aa1-994c-0e4e8e0c205a.png"></img>
</details>

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

If you are compiling from source, you'll need to most recent stable rust compiler.

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
uda.taskwarrior-tui.shortcuts.0=~/local/bin/task-sync.sh
uda.taskwarrior-tui.report.next.filter=(status:pending or status:waiting)
```

</details>

### References / Resources

If you like `taskwarrior-tui`, consider donating to [@GothenburgBitFactory](https://github.com/GothenburgBitFactory) at <https://github.com/sponsors/GothenburgBitFactory> or a charity of your choice.

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
