<#
.SYNOPSIS
Installs the latest version of i2a from GitHub Releases.
#>

$Repo = "BlackTechX011/i2a"
$BinaryName = "i2a.exe"
$InstallDir = "$env:LOCALAPPDATA\i2a"
$AssetName = "i2a-windows-amd64.zip"
$DownloadUrl = "https://github.com/$Repo/releases/latest/download/$AssetName"
$ZipPath = "$env:TEMP\i2a_install.zip"

Write-Host "Starting i2a Installer..." -ForegroundColor Cyan
Write-Host "Target: $InstallDir" -ForegroundColor Gray

try {
    Write-Host "Downloading latest release..." -ForegroundColor Green
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -ErrorAction Stop

    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir | Out-Null
    }

    Write-Host "Extracting..."
    Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force

    # Clean up
    Remove-Item $ZipPath -ErrorAction SilentlyContinue

    # Add to PATH
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -notlike "*$InstallDir*") {
        Write-Host "Adding to PATH..." -ForegroundColor Yellow
        [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
        $env:Path += ";$InstallDir"
        Write-Host "Path updated. You may need to restart your terminal." -ForegroundColor Magenta
    }

    Write-Host "Installation Complete!" -ForegroundColor Green
    Write-Host "Run 'i2a --help' to start."
}
catch {
    Write-Error "Installation failed: $_"
    exit 1
}