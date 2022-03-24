# Frequently Asked Questions (FAQs)

### Does `taskwarrior-tui` show error messages when running shell commands or shortcuts

`taskwarrior-tui` shows an error prompt for shell if:

1. the subprocess fails
2. the subprocess succeeds but prints to stdout
3. the subprocess is empty

`taskwarrior-tui` shows an error prompt for shortcuts if:

1. the shortcut fails

If `taskwarrior-tui` encounters a prompt by the subprocess or the shortcut, `taskwarrior-tui` will not prompt the user for input again.
This means, if you want to run a `taskwarrior` command as a shell command, you may want to pass `rc.confirmation=off` in the command.
See the following screencast as an example:

<video src="https://user-images.githubusercontent.com/1813121/159824511-de66d4fc-0a59-4a65-9c74-7419c127481e.mov"></video>

```bash
task rc.confirmation=off context define test project:work
```

If you don't add `rc.confirmation=off` in the shell command, `taskwarrior-tui` will command the command but it'll fail because it won't receive any prompt.
