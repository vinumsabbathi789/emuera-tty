@echo off
rem tools\test-game.bat [--fast] GAME [emuera-tty options...] - build
rem emuera-tty from this checkout, then run GAME with it.
rem
rem GAME is a game folder (or archive), or the name of a game in the
rem launcher's library: its folder name, its title, or a part of either.
rem --fast builds the "fastbuild" profile (no LTO: much quicker to link,
rem a little slower to run) instead of "release". Anything after GAME goes
rem to emuera-tty (--headless, --check...). The installed copy is left alone.
setlocal
set "REPO=%~dp0.."
set "PROFILE=release"
if /i "%~1"=="--fast" (
  set "PROFILE=fastbuild"
  shift
)
if "%~1"=="" (
  echo usage: %~nx0 [--fast] GAME [emuera-tty options...]
  exit /b 2
)
rem (SHIFT doesn't change %%*, so the arguments are gathered one by one)
set "ARGS="
:args
if "%~1"=="" goto build
set ARGS=%ARGS% "%~1"
shift
goto args
:build
cargo build --profile %PROFILE% --manifest-path "%REPO%\Cargo.toml" || exit /b 1
"%REPO%\target\%PROFILE%\emuera-tty.exe" %ARGS%
