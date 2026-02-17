# Key configuration

Configure `taskwarrior-tui` using `~/.taskrc`:

`taskwarrior-tui` reads values from your `taskwarrior`'s `taskrc` file (default: `~/.taskrc`).

```plaintext
uda.taskwarrior-tui.keyconfig.quit=q
uda.taskwarrior-tui.keyconfig.refresh=r
uda.taskwarrior-tui.keyconfig.go-to-bottom=G
uda.taskwarrior-tui.keyconfig.go-to-top=g
uda.taskwarrior-tui.keyconfig.down=j
uda.taskwarrior-tui.keyconfig.up=k
uda.taskwarrior-tui.keyconfig.page-down=J
uda.taskwarrior-tui.keyconfig.page-up=K
uda.taskwarrior-tui.keyconfig.delete=x
uda.taskwarrior-tui.keyconfig.done=d
uda.taskwarrior-tui.keyconfig.start-stop=s
uda.taskwarrior-tui.keyconfig.quick-tag=t
uda.taskwarrior-tui.keyconfig.undo=u
uda.taskwarrior-tui.keyconfig.edit=e
uda.taskwarrior-tui.keyconfig.duplicate=y
uda.taskwarrior-tui.keyconfig.modify=m
uda.taskwarrior-tui.keyconfig.shell=!
uda.taskwarrior-tui.keyconfig.log=l
uda.taskwarrior-tui.keyconfig.add=a
uda.taskwarrior-tui.keyconfig.annotate=A
uda.taskwarrior-tui.keyconfig.filter=/
uda.taskwarrior-tui.keyconfig.zoom=z
uda.taskwarrior-tui.keyconfig.context-menu=c
uda.taskwarrior-tui.keyconfig.next-tab=]
uda.taskwarrior-tui.keyconfig.previous-tab=[
uda.taskwarrior-tui.keyconfig.help=?
uda.taskwarrior-tui.keyconfig.priority-h=H
uda.taskwarrior-tui.keyconfig.priority-m=M
uda.taskwarrior-tui.keyconfig.priority-l=L
uda.taskwarrior-tui.keyconfig.priority-n=N
uda.taskwarrior-tui.keyconfig.priority-up=+
uda.taskwarrior-tui.keyconfig.priority-down=-
uda.taskwarrior-tui.keyconfig.scroll-down=<C-e>
uda.taskwarrior-tui.keyconfig.scroll-up=<C-y>
uda.taskwarrior-tui.keyconfig.jump=:
uda.taskwarrior-tui.keyconfig.reset-filter=<C-r>
```

## Key notation

Single characters are specified directly:

```plaintext
uda.taskwarrior-tui.keyconfig.quit=q
```

Special keys and modifier combinations use angle bracket notation:

```plaintext
uda.taskwarrior-tui.keyconfig.quit=<Esc>
uda.taskwarrior-tui.keyconfig.filter=<C-f>
uda.taskwarrior-tui.keyconfig.add=<F2>
```

### Modifiers

| Prefix | Meaning |
|--------|---------|
| `C-`   | Ctrl    |
| `A-`   | Alt     |
| `M-`   | Meta (alias for Alt) |
| `S-`   | Shift (only for `<S-Tab>`) |

### Special key names

| Key | Aliases |
|-----|---------|
| `<Esc>` | `<Escape>` |
| `<Enter>` | `<CR>`, `<Return>` |
| `<Tab>` | |
| `<BackTab>` | `<S-Tab>` |
| `<BS>` | `<Backspace>` |
| `<Del>` | `<Delete>` |
| `<Ins>` | `<Insert>` |
| `<Space>` | |
| `<Up>` | |
| `<Down>` | |
| `<Left>` | |
| `<Right>` | |
| `<PageUp>` | `<PgUp>` |
| `<PageDown>` | `<PgDn>` |
| `<Home>` | |
| `<End>` | |
| `<F1>` – `<F12>` | |
| `<Null>` | |
| `<Nop>` | Disables the keybinding (no operation) |

### Ctrl and Alt with special keys

| Key | Result |
|-----|--------|
| `<C-Backspace>` | Ctrl+Backspace |
| `<C-Delete>` | Ctrl+Delete |
| `<A-Backspace>` | Alt+Backspace |
| `<A-Delete>` | Alt+Delete |

Key names are case-insensitive: `<C-e>`, `<c-E>`, and `<C-E>` are all equivalent.

### Disabling keybindings

Use `<Nop>` (no operation) to disable a keybinding. The action will not respond
to any key press, and the help screen will show `<Nop>` for that entry.

```plaintext
uda.taskwarrior-tui.keyconfig.shell=<Nop>
uda.taskwarrior-tui.keyconfig.log=<Nop>
```

Multiple keybindings can be set to `<Nop>` without triggering a duplicate key
conflict.
