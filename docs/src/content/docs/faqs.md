---
title: Frequently Asked Questions
description: Answers to common behavior questions around shell commands, shortcuts, and Taskwarrior prompts.
---

### Does `taskwarrior-tui` show error messages when running shell commands or shortcuts?

`taskwarrior-tui` shows an error prompt for shell commands if:

1. The subprocess fails.
2. The subprocess succeeds but prints to stdout.
3. The subprocess command is empty.

`taskwarrior-tui` shows an error prompt for shortcuts if:

1. The shortcut fails.

If `taskwarrior-tui` encounters a prompt from a subprocess or shortcut, it will not prompt the user for input again. That means if you want to run a `taskwarrior` command as a shell command, you may want to pass `rc.confirmation=off` in the command.

See the following screencast as an example:

<video src="https://user-images.githubusercontent.com/1813121/159824511-de66d4fc-0a59-4a65-9c74-7419c127481e.mov" data-canonical-src="https://user-images.githubusercontent.com/1813121/159824511-de66d4fc-0a59-4a65-9c74-7419c127481e.mov" controls="controls" muted="muted" class="d-block rounded-bottom-2 border-top width-fit" style="max-height:640px;"></video>

```bash
task rc.confirmation=off context define test project:work
```

If you do not add `rc.confirmation=off` in the shell command, `taskwarrior-tui` will run the command but it will fail because it cannot respond to the interactive prompt.

### How do I debug startup failures, crashes, or performance issues?

See [Troubleshooting](./troubleshooting/) for a step-by-step guide to collecting logs, comparing `taskwarrior` command timings, isolating path-specific issues, and filing a useful GitHub issue.
