Output packaging/vhs/basic-features.mp4

Require bash
Require cargo
Require git
Require task

Set Shell bash
Set Theme "Builtin Dark"
Set FontSize 18
Set Width 1160
Set Height 820
Set Padding 12
Set Margin 12
Set MarginFill "#0b1220"
Set BorderRadius 12
Set Framerate 30
Set PlaybackSpeed 0.75
Set CursorBlink false
Set TypingSpeed 80ms

Hide
Type "./target/debug/taskwarrior-tui"
Enter
Show

Sleep 2500ms

Type "j"
Sleep 800ms
Type "k"
Sleep 1000ms

Type "a"
Sleep 250ms
Type "Filter demo: live filter to this task"
Ctrl+E
Type " project:de"
Sleep 300ms
Tab
Sleep 500ms
Enter
Sleep 2200ms

Type "/"
Sleep 250ms
Ctrl+U
Sleep 200ms
Type "Filter demo"
Enter
Sleep 2200ms

Type "/"
Sleep 250ms
Ctrl+U
Sleep 200ms
Enter
Sleep 1200ms

Type "a"
Sleep 250ms
Type "Annotate demo: capture a note on this task"
Ctrl+E
Type " project:de"
Sleep 300ms
Tab
Sleep 500ms
Enter
Sleep 2200ms

Type "z"
Sleep 1200ms
Type "A"
Sleep 250ms
Type "Annotation demo"
Enter
Sleep 2200ms
Type "z"
Sleep 1000ms

Type "a"
Sleep 250ms
Type "Start demo: toggle this task"
Ctrl+E
Type " project:de"
Sleep 300ms
Tab
Sleep 500ms
Enter
Sleep 2200ms

Type "s"
Sleep 2200ms
Type "s"
Sleep 1800ms

Type "a"
Sleep 250ms
Type "Toggle demo: add and remove the 'next' tag"
Ctrl+E
Type " project:de"
Sleep 300ms
Tab
Sleep 500ms
Enter
Sleep 2200ms

Type "t"
Sleep 2000ms
Type "t"
Sleep 1600ms

Type "a"
Sleep 250ms
Type "Mark demo tasks as done together"
Ctrl+E
Type " project:de"
Sleep 300ms
Tab
Sleep 500ms
Enter
Sleep 2200ms

Type "/"
Sleep 250ms
Ctrl+U
Sleep 200ms
Type "project:de"
Sleep 300ms
Tab
Sleep 500ms
Enter
Sleep 2200ms

Type "g"
Sleep 600ms
Type "v"
Sleep 700ms
Type "j"
Sleep 600ms
Type "v"
Sleep 700ms
Type "j"
Sleep 600ms
Type "v"
Sleep 1200ms
Type "d"
Sleep 2600ms

Type "/"
Sleep 250ms
Ctrl+U
Sleep 200ms
Enter
Sleep 1200ms

Type "R"
Sleep 350ms
Type "completed"
Enter
Sleep 2600ms

Type "?"
Sleep 1400ms
Type "jjjjjjjjjjjjjjjjjjjjjjjj"
Sleep 700ms
Type "jjjjjjjjjjjjjjjjjjjjjjjj"
Sleep 1400ms
Escape
Sleep 1000ms

Type "q"
Sleep 600ms
