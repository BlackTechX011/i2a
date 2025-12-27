@echo off
echo Starting i2a Installer...
PowerShell -NoProfile -ExecutionPolicy Bypass -Command "& '%~dp0install.ps1'"
pause
