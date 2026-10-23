# Spec Delta

## Purpose

The debugger of debug mode: a panel under the game with a console, a watch
list and the call stack, as in Emuera's debug window, visible together with
the game and kept up to date.

## ADDED Requirements

### Requirement: The panel exists in debug mode only

The debug panel SHALL be available only when the game runs in debug mode.
Outside debug mode its keys SHALL do what they did before.

#### Scenario: Not in debug mode

- **WHEN** the game runs without debug mode and the player presses Ctrl-D
- **THEN** no panel opens

### Requirement: Ctrl-D and F12 open and close the panel

Ctrl-D SHALL open and close the panel in every game. F12 SHALL do the same
in games with keyboard macros off; with macros on, F12 stays a macro key.

#### Scenario: Opening with Ctrl-D

- **WHEN** the game runs in debug mode and the player presses Ctrl-D
- **THEN** the panel opens at the bottom of the screen, and pressing Ctrl-D again closes it

#### Scenario: F12 with keyboard macros on

- **WHEN** keyboard macros are on and the player presses F12
- **THEN** F12 acts as a macro key and the panel doesn't open

### Requirement: The game and the panel show together

The panel SHALL take rows at the bottom of the screen and the game the rows
above it; the game SHALL be told its height is the rows it has
(`CLIENTHEIGHT`), and the game's width SHALL NOT change. The player SHALL be
able to change the panel's height.

#### Scenario: The game's height while the panel is open

- **WHEN** a 45-row screen shows a 15-row panel
- **THEN** the game is laid out in the 30 rows above it and `CLIENTHEIGHT` reports 30 rows' height

#### Scenario: Closing the panel

- **WHEN** the panel is closed
- **THEN** the game has the whole screen again

### Requirement: Keyboard focus moves between the game and the panel

When the panel opens it SHALL take the keyboard, and Esc SHALL give the
keyboard back to the game without closing the panel. Clicks SHALL go to
whichever of the two is clicked.

#### Scenario: Typing in the console while the game waits

- **WHEN** the panel has the keyboard and the player types and presses Enter
- **THEN** the line goes to the console, not to the game's input

### Requirement: The console runs statements and shows expressions

A line typed in the console tab SHALL run as one eraBASIC statement when it
is one, and otherwise SHALL be evaluated as an expression and its value
shown. The same statements Emuera's debug console refuses (flow control,
input, statements a function can't use, `PUTFORM`, `UPCHECK`, `CUPCHECK`,
`SAVEDATA`) SHALL be refused with a message.

#### Scenario: Reading a value

- **WHEN** the player enters `ABL:パルスィ:テストアビリチ`
- **THEN** the console shows the line and its value, for example `5`

#### Scenario: Changing a value

- **WHEN** the player enters `CFLAG:5:好感度 = 300`
- **THEN** the variable is set and later reads give 300

#### Scenario: A refused statement

- **WHEN** the player enters `BEGIN SHOP`
- **THEN** the console shows that the statement can't be used here, and the game is unaffected

### Requirement: Console errors stay in the console

An error from a console line (a parse error, an unknown variable, an
out-of-range index) SHALL be shown in the console and SHALL NOT stop the
game or show the game's error screen.

#### Scenario: An index out of range

- **WHEN** the player enters `FLAG:99999999`
- **THEN** the console shows the error, and the game waits for input as before

### Requirement: The console works while the game waits

Console lines SHALL run while the game waits for input (any input, wait or
mouse-and-key menu), as Emuera only runs them when no script is running.
While a script is running, the console SHALL say it is busy.

#### Scenario: A busy game

- **WHEN** a script is computing and the player enters a console line
- **THEN** the console says it will run when the game waits for input, and runs it then

### Requirement: The watch list shows values that update by themselves

The watch tab SHALL list expressions the player adds, each with its current
value, updated whenever the game waits for input and while a script runs,
without a button. An expression that can't be evaluated SHALL show its
error in place of the value.

#### Scenario: A value that changes

- **WHEN** `FLAG:0` is in the watch list and a command raises it from 3 to 4
- **THEN** the watch tab shows 4 by the time the game next waits for input

#### Scenario: A bad expression

- **WHEN** the watch list holds `NOSUCHVAR:0`
- **THEN** its row shows the error and the other rows still show their values

### Requirement: The watch list is kept in Emuera's file

The watch list SHALL be read from and saved to `debug/watchlist.csv` in the
game folder, one expression per line, the file Emuera's debug window uses.

#### Scenario: A list made in Emuera

- **WHEN** Emuera saved a watch list for the game and the game starts in emuera-tty's debug mode
- **THEN** the watch tab lists the same expressions

### Requirement: The stack tab shows where the game is

The stack tab SHALL show the running line (file, line number, function) and
the call stack, innermost first, each entry with the function, its file and
line, and the line it was called from, followed by the system state (for
example `BEGIN TRAIN`). It SHALL update as the game runs.

#### Scenario: Inside a kojo function

- **WHEN** the game waits for input in `@TRAIN_MESSAGE`, called from `@KOJO_MESSAGE_SEND`, called from `@SOURCE_CHECK` during training
- **THEN** the stack tab shows the running line in `@TRAIN_MESSAGE`, then the three calls with their files and lines, then `BEGIN TRAIN`

### Requirement: The panel's text is translated

All text the panel shows, other than game values and the player's input,
SHALL go through the interface translation.

#### Scenario: The panel in Japanese

- **WHEN** the interface language is Japanese and the Japanese catalogue translates the tab names
- **THEN** the panel's tabs show the Japanese names
