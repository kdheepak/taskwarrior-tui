% taskwarrior-tui(1) v0.12.0

<!-- This is the taskwarrior-tui(1) man page, written in Markdown. -->
<!-- To generate the roff version, run `just man`, -->
<!-- and the man page will appear in the ‘target’ directory. -->


NAME
====

taskwarrior-tui — A terminal user interface for taskwarrior (https://github.com/kdheepak/taskwarrior-tui)


SYNOPSIS
========

`taskwarrior-tui`

**`taskwarrior-tui`** is a terminal user interface for `taskwarrior`.


EXAMPLES
========

`taskwarrior-tui`
: Starts a terminal user interface for `taskwarrior`.

`alias tt=taskwarrior-tui`
: Add the above to your dotfiles to use `tt` to start `taskwarrior-tui`.

KEYBINDINGS
===========


Keybindings:

`Esc`
: Exit current action

`]`
: Next view                         - Go to next view

`[`
: Previous view                     - Go to previous view


Keybindings for task report:

`/`
: task {string}                     - Filter task report

`a`
: task add {string}                 - Add new task

`d`
: task {selected} done              - Mark task as done

`e`
: task {selected} edit              - Open selected task in editor

`j`
: {selected+=1}                     - Move down in task report

`k`
: {selected-=1}                     - Move up in task report

`J`
: {selected+=pageheight}            - Move page down in task report

`K`
: {selected-=pageheight}            - Move page up in task report

`g`
: {selected=first}                  - Go to top

`G`
: {selected=last}                   - Go to bottom

`l`
: task log {string}                 - Log new task

`m`
: task {selected} modify {string}   - Modify selected task

`q`
: exit                              - Quit

`s`
: task {selected} start/stop        - Toggle start and stop

`u`
: task undo                         - Undo

`x`
: task delete {selected}            - Delete

`z`
: toggle task info                  - Toggle task info view

`A`
: task {selected} annotate {string} - Annotate current task

`!`
: {string}                          - Custom shell command

`1-9`
: {string}                          - Run user defined shortcuts

`c`
: context switcher menu             - Open context switcher menu

`?`
: help                              - Help menu


Keybindings for context switcher:

`j`
: {selected+=1}                     - Move forward a context

`k`
: {selected-=1}                     - Move back a context


Keybindings for calendar:

`j`
: {selected+=1}                     - Move forward a year in calendar

`k`
: {selected-=1}                     - Move back a year in calendar

`J`
: {selected+=10}                    - Move forward a decade in calendar

`K`
: {selected-=10}                    - Move back a decade in calendar

EXIT STATUSES
=============

0
: If everything goes OK.


AUTHOR
======

`taskwarrior-tui` is maintained by Dheepak ‘kdheepak’ Krishnamurthy and other contributors.

**Source code:** `https://github.com/kdheepak/taskwarrior-tui/` \
**Contributors:** `https://github.com/kdheepak/taskwarrior-tui/graphs/contributors`
