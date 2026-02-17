# Default Keybindings

> [!NOTE]
> If any bindings are changed in the `.taskrc`, the built-in help menu will show
> them instead of the defaults. Please see
> [configuration/keys](./configuration/keys.md) for details on custom
> keybindings.

Keybindings:

    Esc:                                 - Exit current action

    ]: Next view                         - Go to next view

    [: Previous view                     - Go to previous view

Keybindings for task report:

    /: task {string}                     - Filter task report

    a: task add {string}                 - Add new task

    d: task {selected} done              - Mark task as done

    e: task {selected} edit              - Open selected task in editor

    y: task {selected} duplicate         - Duplicate tasks

    j: {selected+=1}                     - Move down in task report

    k: {selected-=1}                     - Move up in task report

    J: {selected+=pageheight}            - Move page down in task report

    K: {selected-=pageheight}            - Move page up in task report

    g: {selected=first}                  - Go to top

    G: {selected=last}                   - Go to bottom

    l: task log {string}                 - Log new task

    m: task {selected} modify {string}   - Modify selected task

    q: exit                              - Quit

    s: task {selected} start/stop        - Toggle start and stop

    t: task {selected} +{tag}/-{tag}     - Toggle {uda.taskwarrior-tui.quick-tag.name} (default: `next`)

    u: task undo                         - Undo

    v: {toggle mark on selected}         - Toggle mark on selected

    V: {toggle marks on all tasks}       - Toggle marks on all tasks in current filter report

    x: task {selected} delete            - Delete

    z: toggle task info                  - Toggle task info view

    A: task {selected} annotate {string} - Annotate current task

    Ctrl-e: scroll down task details     - Scroll task details view down one line

    Ctrl-y: scroll up task details       - Scroll task details view up one line

    !: {string}                          - Custom shell command

    1-9: {string}                        - Run user defined shortcuts

    :: {task id}                         - Jump to task id

    c: context switcher menu             - Open context switcher menu

    ?: help                              - Help menu

    H: priority H                        - Set priority to High

    M: priority M                        - Set priority to Medium

    L: priority L                        - Set priority to Low

    N: priority N                        - Remove priority

    +: priority up                       - Increase priority (wraps: None → L → M → H → None)

    -: priority down                     - Decrease priority (wraps: None → H → M → L → None)

Keybindings for filter / command prompt:

    Ctrl-r: reset filter                 - Reset filter to default

    Ctrl + f | Right: move forward       - Move forward one character

    Ctrl + b | Left: move backward       - Move backward one character

    Ctrl + h | Backspace: backspace      - Delete one character back

    Ctrl + d | Delete: delete            - Delete one character forward

    Ctrl + a | Home: home                - Go to the beginning of line

    Ctrl + e | End: end                  - Go to the end of line

    Ctrl + k: delete to end              - Delete to the end of line

    Ctrl + u: delete to beginning        - Delete to the beginning of line

    Ctrl + w: delete previous word       - Delete previous word

    Alt + d: delete next word            - Delete next word

    Alt + b: move to previous word       - Move to previous word

    Alt + f: move to next word           - Move to next word

    Alt + t: transpose words             - Transpose words

    Up: scroll history                   - Go backward in history matching from beginning of line to cursor

    Down: scroll history                 - Go forward in history matching from beginning of line to cursor

    TAB | Ctrl + n: tab complete         - Open tab completion and selection first element OR cycle to next element

    BACKTAB | Ctrl + p: tab complete     - Cycle to previous element

Keybindings for context switcher:

    j: {selected+=1}                     - Move forward a context

    k: {selected-=1}                     - Move back a context

    Enter: task context {selected}       - Select highlighted context

Keybindings for calendar:

    j: {selected+=1}                     - Move forward a year in calendar

    k: {selected-=1}                     - Move back a year in calendar

    J: {selected+=10}                    - Move forward a decade in calendar

    K: {selected-=10}                    - Move back a decade in calendar
