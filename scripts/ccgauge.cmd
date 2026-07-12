@echo off
REM ccgauge — thin CLI wrapper around the installed ClaudeGauge widget.
REM Put this folder (or the install dir) on your PATH, then use:
REM   ccgauge start | stop | toggle | hide
REM
REM It forwards all args to ClaudeGauge.exe. A second launch is routed to the
REM already-running instance via single-instance, so `ccgauge stop` etc. work.

set "EXE=%LOCALAPPDATA%\ClaudeGauge\ClaudeGauge.exe"
if not exist "%EXE%" set "EXE=%ProgramFiles%\ClaudeGauge\ClaudeGauge.exe"
if not exist "%EXE%" (
  echo ClaudeGauge.exe not found. Edit EXE path in ccgauge.cmd or install the .msi.
  exit /b 1
)

start "" "%EXE%" %*
