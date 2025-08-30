To install you need to build the project using cargo. create a .bat file in the build folder and add this

@echo off
setlocal enabledelayedexpansion

:: Define installation path
set INSTALL_DIR=%ProgramFiles%\GitGetter
set EXECUTABLE=GitGetter.exe

:: Check if the executable exists before proceeding
if not exist "%~dp0%EXECUTABLE%" (
    echo ERROR: %EXECUTABLE% not found in %~dp0
    pause
    exit /b 1
)

:: Create installation directory
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

:: Copy the executable
copy /Y "%~dp0%EXECUTABLE%" "%INSTALL_DIR%\%EXECUTABLE%"

:: Add to system PATH
set PATH_KEY="HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
set PATH_VALUE="Path"

for /f "tokens=2* delims= " %%A in ('reg query %PATH_KEY% /v %PATH_VALUE%') do set OLD_PATH=%%B
echo %OLD_PATH% | findstr /C:"%INSTALL_DIR%" >nul
if %errorlevel% neq 0 reg add %PATH_KEY% /v %PATH_VALUE% /t REG_EXPAND_SZ /d "%OLD_PATH%;%INSTALL_DIR%" /f

echo Installation complete! Restart your terminal to use GitGetter.
pause

After which you'll run it and its now installed. You can then run the program with .\GitGetter which will download all the repos from the repos.json file.
