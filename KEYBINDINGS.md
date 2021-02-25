Keybindings:

    Esc:                                 - Exit current action

    ]: Next view                         - Go to next view

    [: Previous view                     - Go to previous view


Keybindings for task report:

    /: task {string}                     - Filter task report

    a: task add {string}                 - Add new task

    e: task {selected} edit              - Open selected task in editor

    j: {selected+=1}                     - Move down in task report

    k: {selected-=1}                     - Move up in task report

    J: {selected+=pageheight}            - Move page down in task report

    K: {selected-=pageheight}            - Move page up in task report

    gg: {selected=last}                  - Go to top

    G: {selected=last}                   - Go to bottom

    l: task log {string}                 - Log new task

    m: task {selected} modify {string}   - Modify selected task

    q: exit                              - Quit

    s: task {selected} start/stop        - Toggle start and stop

    u: task undo                         - Undo

    x: task {selected} done              - Mark task as done

    z: toggle task info                  - Toggle task info view

    A: task {selected} annotate {string} - Annotate current task

    dd: task delete {selected}           - Delete current task with prompt

    gg: task top of list                 - Go to top of the task list

    G: task bottom of list               - Go to bottom of the task list

    !: {string}                          - Custom shell command

    c: context switcher menu             - Open context switcher menu

    ?: help                              - Help menu


Keybindings for context switcher:

    j: {selected+=1}                     - Move forward a context

    k: {selected-=1}                     - Move back a context


Keybindings for calendar:

    j: {selected+=1}                     - Move forward a year in calendar

    k: {selected-=1}                     - Move back a year in calendar

    J: {selected+=10}                    - Move forward a decade in calendar

    K: {selected-=10}                    - Move back a decade in calendar
