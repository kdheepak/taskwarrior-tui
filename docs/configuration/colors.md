# Color configuration

`taskwarrior-tui` reads color values from your taskwarrior `taskrc` file (default: `~/.taskrc`).

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

## Color formats

All color formats supported by Taskwarrior are recognized:

### Named colors (16-color)

```
black  red  green  yellow  blue  magenta  cyan  white
```

Use `on <color>` to set the background:

```
red on blue        # red foreground, blue background
on yellow          # default foreground, yellow background
```

### High-intensity (bright) colors

Use the `bright` prefix on backgrounds to get the high-intensity variant (terminal colors 8–15):

```
black on bright green    # bright green background
white on bright black    # bright black (dark gray) background
```

### 256-color indexed

Use `color0` through `color255` to access the full 256-color palette:

```
color196           # bright red
color60 on color60 # foreground and background using index 60
```

### Grayscale ramp

Use `gray0` through `gray23` (or `grey0`–`grey23`) to access the 24-step grayscale ramp
(terminal colors 232–255):

```
gray5              # dark gray (Color::Indexed(237))
white on gray10    # white text on mid-gray background
```

### RGB color cube

Use `rgbRGB` where R, G, B are each a digit from 0 to 5 to address the 6×6×6 color cube
(terminal colors 16–231):

```
rgb500             # bright red  (Color::Indexed(196))
rgb050             # bright green
rgb005             # bright blue
rgb444             # medium gray-ish
```

## Modifiers

The following text modifiers can be combined with any color:

| Modifier | Effect |
|---|---|
| `bold` | Bold text |
| `underline` | Underlined text |
| `inverse` | Swapped foreground/background |
| `italic` | Italic text |
| `strikethrough` | Strikethrough text |

Multiple modifiers can be combined:

```
bold underline red on blue
italic color111 on color60
```

## `bold` vs `bright`

- `bold` is a **text attribute** — it makes text bold and uses the regular color index.
- `bright` is a **color variant** — for backgrounds, it selects the high-intensity color (index + 8).

```
bold red           # regular red (#1) + bold attribute
on bright red      # high-intensity red background (#9)
bold on bright red # bold text + high-intensity red background
```

## Example: TokyoNight Moon theme

```
color.active=color111 on color60
color.overdue=color203
color.due.today=color215
color.due=color111
color.blocked=color245
color.tagged=color141
color.recurring=color147
color.scheduled=color109
```

See [advanced configuration](advanced.md) for TUI-specific style keys.
