# Frequently Asked Questions (FAQs)

### Does `taskwarrior-tui` show error messages when running shell commands or shortcuts

`taskwarrior-tui` shows an error prompt for shell if:

1. the subprocess fails
2. the subprocess succeeds but prints to stdout
3. the subprocess is empty

`taskwarrior-tui` shows an error prompt for shortcuts if:

1. the shortcut fails
