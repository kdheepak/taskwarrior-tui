---
name: Bug report
about: Create a bug report
title: ""
labels: bug
assignees: ""
---

<!--

Thank you for taking the time to fill out a bug report.

If you like `taskwarrior-tui`, please consider donating to [@GothenburgBitFactory](https://github.com/sponsors/GothenburgBitFactory).

-->

**Describe the bug**

<!-- A clear and concise description of what the bug is with screenshots if available. -->

**To Reproduce**

- [ ] Reproducible using the test data located here: <https://github.com/kdheepak/taskwarrior-testdata/>

Steps to reproduce the behavior:

<!--
Please provide a minimal working example of the bug with screenshots if possible.
If not possible, please provide a anonymized version of your `.taskrc` file and the output of `task next` (or whatever relevant taskwarrior feature you are using).

You can set the TASKDATA and TASKRC environment variables to point to a different location for temporary fresh taskwarrior session.

You can use the following fake task list to reproduce your error:

```bash
git clone https://github.com/kdheepak/taskwarrior-testdata/
```

After you clone the above repository, run the following lines in your shell.

```bash
export TASKRC=`pwd`/taskwarrior-testdata/.taskrc
export TASKDATA=`pwd`/taskwarrior-testdata/.task
```

Then run the following:

```bash
task import `pwd`/taskwarrior-testdata/export.json
```

See taskwarrior documentation for more information.

Use your favorite tool to generate a screenshot or a gif of the error.
-->

**Environment (please complete the following information):**

- Operating System: <!-- Windows | Mac | Linux -->
- Installation: <!-- github releases | homebrew | arch | zinit -->
- taskwarrior-tui version:

```bash
taskwarrior-tui --version
```

- task version:

```bash
task --version
```

**Additional context and information**

<!--

Please provide detailed stacktraces, screenshot, etc here.
If `taskwarrior-tui` crashes, you can set the RUST_BACKTRACE=1 for a detailed stacktrace.

The following is the data directory that `taskwarrior-tui` uses:

Platform | Value                                |  Example
--------------------------------------------------------------------------------------------
Linux    | $XDG_DATA_HOME or $HOME/.local/share |  /home/alice/.local/share
macOS    | $HOME/Library/Application Support    |  /Users/Alice/Library/Application Support
Windows  | {FOLDERID_LocalAppData}              |  C:\Users\Alice\AppData\Local

If an appropriate log level is set, the following file may have useful information: ${data-dir}/taskwarrior-tui/taskwarrior-tui.log

```bash
export TASKWARRIOR_TUI_LOG_LEVEL=debug
export RUST_BACKTRACE=1
taskwarrior-tui

# OR

export TASKWARRIOR_TUI_LOG_LEVEL=trace
export RUST_BACKTRACE=1
taskwarrior-tui
```
-->
