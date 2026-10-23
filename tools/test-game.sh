#!/bin/sh
# tools/test-game.sh [--fast] GAME [emuera-tty options…] — build emuera-tty
# from this checkout, then run GAME with it.
#
# GAME is a game folder (or archive), or the name of a game in the
# launcher's library: its folder name, its title, or a part of either.
# --fast builds the `fastbuild` profile (no LTO: much quicker to link,
# a little slower to run) instead of `release`. Anything after GAME goes
# to emuera-tty (`--headless`, `--check`…). The installed copy is left alone.
set -e
repo=$(cd "$(dirname "$0")/.." && pwd)
profile=release
if [ "$1" = "--fast" ]; then
  profile=fastbuild
  shift
fi
if [ $# -eq 0 ]; then
  sed -n '2,10s/^# \{0,1\}//p' "$0"
  exit 2
fi
# rustup puts cargo here, which a non-login shell may not have on PATH
command -v cargo >/dev/null 2>&1 || PATH="$HOME/.cargo/bin:$PATH"
cargo build --profile "$profile" --manifest-path "$repo/Cargo.toml"
exec "$repo/target/$profile/emuera-tty" "$@"
