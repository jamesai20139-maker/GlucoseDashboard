$ErrorActionPreference = 'Stop'
$InstallRoot = Join-Path $env:LOCALAPPDATA 'GlucoseDashboard'
New-Item -ItemType Directory -Force -Path $InstallRoot | Out-Null
Write-Host 'Glucose Dashboard installer scaffold'
Write-Host "Install target: $InstallRoot"
Write-Host 'Release packaging will copy the backend executable and frontend assets here.'
