## Why

Emuera's debug mode has a debug window that game developers and modders
rely on: a console that runs eraBASIC statements and shows the values of
expressions, a watch list of variables, and the running line with its call
stack. emuera-tty runs in debug mode (`--debug`, or *Restart in debug mode*)
but has nothing to look inside the game with, so checking a CFLAG means
editing scripts. A terminal can't open a second window, and players want to
see the game and the debugger together.

## What Changes

- A **debug panel** docked at the bottom of the game screen, in debug mode
  only, opened and closed with **Ctrl-D** (everywhere) or **F12** (in games
  with keyboard macros off), its height adjustable.
- Three tabs, as Emuera's debug window:
  - **Console**: type a statement (`CFLAG:5:好感度 = 300`) or an expression
    (`ABL:パルスィ:テストアビリチ`); statements run, expressions print their
    value, errors show in the console instead of stopping the game. Emuera's
    rules for which statements are allowed.
  - **Watch**: a list of expressions with their values, **updated
    automatically** (Emuera needs a button); kept in the game's
    `debug/watchlist.csv`, the file Emuera uses, so lists carry over.
  - **Stack**: the running line (file, line, function) and the call stack,
    updated live.
- The game gets the rows above the panel: `CLIENTHEIGHT` reports them, as
  when Emuera's window is made shorter.
- All panel text through the interface translation of `add-ui-i18n`.

Out of scope for now: a debugger in a separate terminal (`--debug-attach`;
the design keeps the panel's data separable so it can come later),
breakpoints and stepping (Emuera has neither in its window), and Emuera's
debug *config* dialog.

## Capabilities

### New Capabilities

- `debug-panel`: the in-game debugger of debug mode: how it opens and where
  it shows, its console, watch list and stack views, and how they update.

### Modified Capabilities

(none: no specs exist yet)

## Impact

- `tty.rs`: panel drawing, focus and keys, the game area's height; a new
  module for the panel's views (`tty/debug.rs`).
- `frontend.rs`: a way to pass console lines to the VM (as hotkeys are) and
  a snapshot of debug data from the VM.
- `vm/`: compiling and running one console line (the parser already compiles
  `EMUERA_INJECT` text into a function), evaluating watch expressions,
  describing the call stack (the error screen already prints one).
- Depends on `add-ui-i18n` for its text (`t!`); can be built before
  translations exist.
- No effect outside debug mode; saves unchanged (console statements can
  change the game state, as in Emuera).
