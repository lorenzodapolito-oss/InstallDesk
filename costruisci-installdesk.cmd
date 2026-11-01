@echo off
rem Scarica (o aggiorna) e compila InstallDesk. Metti questo file in AmbienteRust.
title Costruisci InstallDesk
set "ROOT=%~dp0"
if "%ROOT:~-1%"=="\" set "ROOT=%ROOT:~0,-1%"
set "CARGO_HOME=%ROOT%\rust\cargo"
set "RUSTUP_HOME=%ROOT%\rust\rustup"
set "PATH=%ROOT%\winlibs\mingw64\bin;%ROOT%\rust\cargo\bin;%ROOT%\git\cmd;%ROOT%\git\bin;%PATH%"
set "PROG=%ROOT%\Progetti"
if not exist "%PROG%" mkdir "%PROG%"
cd /d "%PROG%"
taskkill /f /im installdesk.exe >nul 2>&1

echo === Scarico o aggiorno InstallDesk ===
if exist "%PROG%\InstallDesk\.git" (
  git -C "%PROG%\InstallDesk" fetch origin
  if errorlevel 1 goto fail
  git -C "%PROG%\InstallDesk" reset --hard origin/main
) else (
  git clone https://github.com/lorenzodapolito-oss/InstallDesk "%PROG%\InstallDesk"
)
if errorlevel 1 goto fail

echo === Compilo ===
cd /d "%PROG%\InstallDesk"
cargo build --release
if errorlevel 1 goto fail

set "OUT=%PROG%\InstallDesk\dist\InstallDesk-portable"
if exist "%OUT%" rmdir /s /q "%OUT%"
mkdir "%OUT%"
copy /y "target\release\installdesk.exe" "%OUT%\InstallDesk.exe" >nul
if errorlevel 1 goto fail

echo.
echo FATTO: %OUT%\InstallDesk.exe
pause
exit /b 0

:fail
echo.
echo ERRORE: operazione non riuscita. Controlla i messaggi sopra.
pause
exit /b 1
