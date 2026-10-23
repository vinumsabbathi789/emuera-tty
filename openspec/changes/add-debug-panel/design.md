## Context

See proposal.md for why. How the pieces talk today: the VM runs on its own
thread and owns all game state; the frontend only sees what the VM hands it
(`Frontend::input(con, res, req)`), and answers with an `InputResult`. Keys
the VM must act on come back as `InputResult::Hotkey` (title, reload, config)
and the VM asks for input again afterwards (`Vm::raw_input_in`). While a
script runs, the VM calls the frontend every 150 ms (`tick` → `flush` /
`busy`). The error screen already prints a call stack from `Vm::frames`.
`EMUERA_INJECT` already compiles statements into a function at load time.
Emuera's console (`EmueraConsole.DebugCommand`) parses the line as a
statement, else wraps an expression in `DEBUGPRINTFORML {…}` / `%…%`, refuses
flow control, input, non-`METHOD_SAFE`, partial (`SIF`-unsafe) statements
and `PUTFORM` / `UPCHECK` / `CUPCHECK` / `SAVEDATA`, and saves its watch list
in `debug/watchlist.csv`, one expression per line.

## Goals / Non-Goals

**Goals:** the three views live, next to the game; Emuera's console rules;
Emuera's watch file; no cost outside debug mode or with the panel closed.

**Non-Goals:** a separate-terminal debugger (but see *Snapshot*);
breakpoints or stepping; the headless and retro frontends; Emuera's debug
configuration dialog; Emuera-EE's hotkey state (also Ctrl+D in Emuera, and
only stubbed in emuera-tty).

## Decisions

### Data flows as a snapshot down and commands up

The VM builds a `DebugSnapshot` (stack entries, system state, each watch
expression with its value or error, new console output lines) and hands it
to the frontend with each input request and each 150 ms tick, only while
the panel is open. Console lines and watch-list edits go back as
`InputResult::Debug(DebugCmd)`; the VM handles them and asks for input
again, as it does for hotkeys.

Why: the VM keeps sole ownership of the state (no locking, no second
thread touching variables), and the snapshot is plain data, so a later
`--debug-attach` can send the same snapshot over a socket and the same
commands back.

Alternatives: the frontend reading VM state directly (shared ownership
across threads: locks in the hot interpreter loop); a socket from the start
(more work before anything is usable).

### Console lines: Emuera's parse, our compile

A line is first parsed as a statement (the parser's statement entry); if
that fails or the line starts with `@`, `"` or `\`, it is parsed as an
expression and turned into `DEBUGPRINTFORML {…}` (integer) or `%…%`
(string), as Emuera does. The refused statements are checked against the
command table (flow control, input, `NOT_IN_FUNCTION`, partial statements,
the four named ones). The line is compiled into a one-statement function
`@__DEBUG_CONSOLE` and called; `DEBUGPRINT` output goes to the console tab.

Errors (`Ctrl::Error`) are caught around that call and shown in the console;
`Ctrl::Quit` / `BEGIN` from a console line can't happen (refused). The VM
runs console lines only between inputs, so the game's frames are never in
the middle of a statement.

### Watch values: evaluated per snapshot

Each watch expression is parsed once (re-parsed when edited) and evaluated
for every snapshot; evaluation errors become the row's text. A watch that
calls a user function runs it, as Emuera's does; the cost is the player's
choice. With the panel closed nothing is evaluated.

### Stack: from the frames, with Emuera's information

Each frame gives its function, file and current line; the line it was
called from is the caller frame's current line. The system state (TITLE,
SHOP, TRAIN, …) comes from the system loop. The labels are translated
(`t!`); file paths are shown as Emuera shows them (relative, `\` as `/`).

### The panel in the frontend

`tty/debug.rs` holds the panel's state (open, height, tab, focus, console
history and input line, watch list cursor) and draws it into the bottom
rows; `tty.rs` gives the game `rows - panel_height` rows and reports that
height as `CLIENTHEIGHT` (`Frontend::rows`). Height: Ctrl-Up / Ctrl-Down
while the panel has focus (5 rows minimum, the game keeps at least 10).
Tabs: Left / Right on an empty input line, or a click on the tab names.
Focus: opening gives the panel the keyboard; Esc gives it back to the game;
a click focuses what is clicked. The console's input line reuses the input
line editor (`tty/lineedit.rs`: cursor, history).

Why the game's height changes rather than staying full: a picture menu sized
from `CLIENTHEIGHT` (pops-tw's title) would otherwise be drawn under the
panel. Emuera games already handle a shorter window, as when the window is
resized.

### Keys

Ctrl-D (0x04, unused today) toggles everywhere; F12 toggles when keyboard
macros are off (with macros on it stays a macro key). Outside debug mode
both do what they did.

## Risks / Trade-offs

- [Console statements change the game state with no undo] → as in Emuera;
  the save history (`save-safety`) keeps earlier saves; the panel says so on
  first use.
- [A watch with side effects (a function that changes variables) runs on
  every snapshot] → evaluated only with the panel open, at input and at
  most every 150 ms; documented.
- [A shorter `CLIENTHEIGHT` can change a game's layout while the panel is
  open] → only in debug mode, and closing the panel restores it.
- [Emuera-EE's hotkey state uses Ctrl+D in Emuera] → not implemented here;
  if it is added, it gets another key in debug mode.

## Migration Plan

Nothing to migrate. Outside debug mode nothing changes; `debug/watchlist.csv`
is only written when the watch list is edited.
