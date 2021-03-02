---
name: Bug report
about: Create a bug report
title: ""
labels: bug
assignees: ""
---

<!-- Thank you for taking the time to fill out a bug report. -->

**Describe the bug**

<!-- A clear and concise description of what the bug is with screenshots if available. -->

**To Reproduce**

Steps to reproduce the behavior:

<!--
Please provide a minimal working example of the bug with screenshots if possible.

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

<!-- Please provide detailed stacktraces, screenshot, etc here. If `taskwarrior-tui` crashes, you can set the RUST_BACKTRACE=1 for a detailed stacktrace. -->
