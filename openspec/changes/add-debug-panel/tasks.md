## 1. VM side

- [ ] 1.1 `DebugSnapshot` / `DebugCmd` types in `frontend.rs`; `InputResult::Debug`, handled in `Vm::raw_input_in` like hotkeys (ask for input again)
- [ ] 1.2 Console line compiler: statement or expression (Emuera's `DebugCommand` rules), refused statements with a message, compiled as `@__DEBUG_CONSOLE` and run with errors caught; `DEBUGPRINT` output collected for the console
- [ ] 1.3 Watch evaluation: parse once per expression, evaluate per snapshot, errors as text
- [ ] 1.4 Stack description: running line, call stack (function, file, line, called-from line), system state
- [ ] 1.5 Snapshots built at each input and each 150 ms tick, only while the panel is open; unit tests for the compiler's accept/refuse rules and for the stack text

## 2. The panel

- [ ] 2.1 `tty/debug.rs`: panel state, tab bar, the three views drawn into the bottom rows
- [ ] 2.2 Ctrl-D / F12 (macros off) toggle in debug mode only; focus moves (Esc to the game, clicks); Ctrl-Up / Ctrl-Down height; Left / Right or clicks for tabs
- [ ] 2.3 Game area shortened by the panel; `CLIENTHEIGHT` follows; full height again when closed
- [ ] 2.4 Console tab: input line with cursor and history (`lineedit`), scrollback of commands and results, "busy" message while a script runs
- [ ] 2.5 Watch tab: add, edit, delete expressions; values beside them; `debug/watchlist.csv` read at start and written on change (Emuera's format)
- [ ] 2.6 Stack tab, updated live
- [ ] 2.7 All panel text through `t!` (after `add-ui-i18n`'s framework)

## 3. Tests

- [ ] 3.1 Console: reading `FLAG:0`, setting it, a string expression, a refused `BEGIN`, an out-of-range index shown as an error with the game unaffected
- [ ] 3.2 Watch: values updating after a command, a bad expression's error, a list read from an Emuera-written `watchlist.csv`
- [ ] 3.3 TUI (`tools/gamed.py`): panel opens with Ctrl-D in debug mode only, the game area shrinks and `CLIENTHEIGHT` follows, Esc returns the keyboard

## 4. Docs and finishing

- [ ] 4.1 README (*Debug mode*: the panel, keys, the console rules, `watchlist.csv`), man page (Ctrl-D, F12)
- [ ] 4.2 `cargo test`, `cargo clippy`, `cargo fmt --check`; Windows and big-endian checks
