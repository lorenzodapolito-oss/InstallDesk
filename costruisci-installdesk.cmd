@echo off
chcp 65001 >nul
setlocal
rem Compila InstallDesk con il Rust portatile e crea dist\InstallDesk-portable.
set "PROG=%~dp0"
if exist "D:\AmbienteRust\cargo\bin\cargo.exe" (
    set "CARGO_HOME=D:\AmbienteRust\cargo"
    set "RUSTUP_HOME=D:\AmbienteRust\rustup"
    set "PATH=D:\AmbienteRust\cargo\bin;%PATH%"
)
where cargo >nul 2>&1 || (echo Cargo non trovato. Controlla D:\AmbienteRust & pause & exit /b 1)
if not exist "%PROG%InstallDesk\Cargo.toml" (
    git clone https://github.com/lorenzodapolito-oss/InstallDesk "%PROG%InstallDesk" || (pause & exit /b 1)
) else (
    git -C "%PROG%InstallDesk" pull
)
cd /d "%PROG%InstallDesk"
cargo build --release || (echo Compilazione fallita & pause & exit /b 1)
if exist dist rmdir /s /q dist
mkdir "dist\InstallDesk-portable"
copy /y "target\release\installdesk.exe" "dist\InstallDesk-portable\" >nul
echo.
echo Pronto: %CD%\dist\InstallDesk-portable\installdesk.exe
pause
