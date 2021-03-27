# Default Keybindings

Keybindings:

    Esc:                                 - Exit current action

    ]: Next view                         - Go to next view

    [: Previous view                     - Go to previous view


Keybindings for task report:

    /: task {string}                     - Filter task report

    a: task add {string}                 - Add new task

    d: task {selected} done              - Mark task as done

    e: task {selected} edit              - Open selected task in editor

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

    u: task undo                         - Undo

    v: {toggle mark on selected}         - Toggle mark on selected

    V: {toggle marks on all tasks}       - Toggle marks on tasks in current filter report

    x: task {selected} delete            - Delete

    z: toggle task info                  - Toggle task info view

    A: task {selected} annotate {string} - Annotate current task

    Ctrl-e: scroll down task details     - Scroll task details view down one line

    Ctrl-y: scroll up task details       - Scroll task details view up one line

    !: {string}                          - Custom shell command

    1-9: {string}                        - Run user defined shortcuts

    c: context switcher menu             - Open context switcher menu

    ?: help                              - Help menu

Keybindings for context switcher:

    j: {selected+=1}                     - Move forward a context

    k: {selected-=1}                     - Move back a context

    Enter: task context {selected}       - Select highlighted context


Keybindings for calendar:

    j: {selected+=1}                     - Move forward a year in calendar

    k: {selected-=1}                     - Move back a year in calendar

    J: {selected+=10}                    - Move forward a decade in calendar

    K: {selected-=10}                    - Move back a decade in calendar
