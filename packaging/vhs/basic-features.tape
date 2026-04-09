Output packaging/vhs/basic-features.mp4

Require bash
Require cargo
Require git
Require task

Set Shell bash
Set Theme "Builtin Dark"
Set FontSize 20
Set Width 1800
Set Height 980
Set Padding 20
Set Margin 20
Set MarginFill "#11111b"
Set BorderRadius 10
Set Framerate 30
Set PlaybackSpeed 0.45
Set CursorBlink false
Set TypingSpeed 85ms

Hide
Type "./target/debug/taskwarrior-tui"
Enter
Show

Sleep 4000ms

Type "j"
Sleep 1200ms
Type "j"
Sleep 1500ms

Type "a"
Sleep 300ms
Type "Filter demo: narrow to this task project:demo"
Enter
Sleep 3600ms

Type "/"
Sleep 300ms
Ctrl+U
Sleep 250ms
Type "Filter demo"
Enter
Sleep 3200ms

Type "/"
Sleep 300ms
Ctrl+U
Sleep 250ms
Enter
Sleep 1800ms

Type "a"
Sleep 300ms
Type "Annotate demo: capture a note on this task project:demo"
Enter
Sleep 3400ms

Type "z"
Sleep 1800ms
Type "A"
Sleep 400ms
Type "Annotation during the demo"
Enter
Sleep 3600ms
Type "z"
Sleep 1600ms

Type "a"
Sleep 300ms
Type "Start demo: toggle this task project:demo"
Enter
Sleep 3400ms

Type "s"
Sleep 3600ms
Type "s"
Sleep 3200ms

Type "a"
Sleep 300ms
Type "Toggle demo: add and remove the next tag project:demo"
Enter
Sleep 3400ms

Type "t"
Sleep 3200ms
Type "t"
Sleep 3200ms

Type "a"
Sleep 300ms
Type "Mark demo tasks as done together project:demo"
Enter
Sleep 3600ms

Type "/"
Sleep 300ms
Ctrl+U
Sleep 250ms
Type "project:demo"
Enter
Sleep 3600ms

Type "g"
Sleep 900ms
Type "v"
Sleep 1000ms
Type "j"
Sleep 900ms
Type "v"
Sleep 1000ms
Type "j"
Sleep 900ms
Type "v"
Sleep 1000ms
Type "j"
Sleep 900ms
Type "v"
Sleep 1800ms
Type "d"
Sleep 4200ms

Type "/"
Sleep 300ms
Ctrl+U
Sleep 250ms
Enter
Sleep 1800ms

Type "c"
Sleep 500ms
Type "finance"
Enter
Sleep 3600ms
Type "c"
Sleep 500ms
Type "none"
Enter
Sleep 2600ms

Type "R"
Sleep 500ms
Type "completed"
Enter
Sleep 3600ms
Type "R"
Sleep 500ms
Type "next"
Enter
Sleep 2600ms

Type "?"
Sleep 2000ms
Type "jjjjjjjjjjjjjjjjjjjj"
Sleep 1200ms
Type "jjjjjjjjjjjjjjjjjjjj"
Sleep 1200ms
Type "jjjjjjjjjjjjjjjjjjjj"
Sleep 1200ms
Type "jjjjjjjjjjjjjjjjjjjj"
Sleep 1200ms
Type "jjjjjjjjjjjjjjjjjjjj"
Sleep 2200ms
Escape
Sleep 1800ms

Type "q"
Sleep 800ms
