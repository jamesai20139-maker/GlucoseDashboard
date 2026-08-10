#Requires -Version 5.1
<#
.SYNOPSIS
  Glucose Dashboard 一鍵安裝／啟動腳本（PowerShell）。
.DESCRIPTION
  從 GitHub Releases 下載最新版單一可執行檔（前端已嵌入），
  安裝到 %LOCALAPPDATA%\GlucoseDashboard，建立 .cmd 啟動捷徑並加入
  使用者 PATH，最後直接啟動 Dashboard（後端會自動開啟瀏覽器）。
  不會覆蓋既有的設定檔 .glucose-dashboard.json。
.PARAMETER Repo
  GitHub 倉庫（owner/repo），預設 gaistudio138/GlucoseDashboard。
.PARAMETER NoLaunch
  安裝完成後不自動啟動。
.EXAMPLE
  irm https://raw.githubusercontent.com/gaistudio138/GlucoseDashboard/main/installer/get.ps1 | iex
#>
[CmdletBinding()]
param(
    [string]$Repo = 'gaistudio138/GlucoseDashboard',
    [switch]$NoLaunch
)

$ErrorActionPreference = 'Stop'
$InstallRoot = Join-Path $env:LOCALAPPDATA 'GlucoseDashboard'
$ExeName = 'glucose-dashboard.exe'
$ExePath = Join-Path $InstallRoot $ExeName
$CmdName = 'glucose-dashboard.cmd'
$CmdPath = Join-Path $InstallRoot $CmdName
$ConfigName = '.glucose-dashboard.json'
$ConfigPath = Join-Path $InstallRoot $ConfigName

function Write-Step([string]$msg) { Write-Host "==> $msg" -ForegroundColor Cyan }
function Write-Ok([string]$msg) { Write-Host "    $msg" -ForegroundColor Green }
function Write-Warn([string]$msg) { Write-Host "    $msg" -ForegroundColor Yellow }

Write-Step 'Glucose Dashboard 安裝程式'
Write-Host "    安裝目錄：$InstallRoot"

# 1. 建立安裝目錄
New-Item -ItemType Directory -Force -Path $InstallRoot | Out-Null

# 2. 查詢最新 release 的可執行檔下載 URL
Write-Step "查詢最新版本（$Repo）"
$apiUrl = "https://api.github.com/repos/$Repo/releases/latest"
$release = $null
try {
    $release = Invoke-RestMethod -Uri $apiUrl -Headers @{ 'User-Agent' = 'glucose-dashboard-installer' } -ErrorAction Stop
} catch {
    throw "無法查詢最新版本：$($_.Exception.Message)`n請確認已發布 Release（tag push 觸發 CI 產物）。"
}
$asset = $release.assets | Where-Object { $_.name -eq $ExeName } | Select-Object -First 1
if (-not $asset) {
    throw "最新 release 找不到資產 $ExeName。已發布資產：$($release.assets.name -join ', ')"
}
$version = $release.tag_name
$downloadUrl = $asset.browser_download_url
Write-Ok "最新版本：$version"

# 3. 下載可執行檔到安裝目錄
Write-Step "下載 $ExeName"
$TmpExe = Join-Path $env:TEMP "$ExeName.$version.download"
Invoke-WebRequest -Uri $downloadUrl -OutFile $TmpExe -UseBasicParsing
Move-Item -Path $TmpExe -Destination $ExePath -Force
Write-Ok "已下載到 $ExePath"

# 4. 產生 .cmd shim：直接執行 exe（exe 內部會啟動本機服務並開瀏覽器）。
Write-Step "建立啟動指令 $CmdName"
$cmdContent = "@echo off`r`n`"$ExePath`" %*"
Set-Content -Path $CmdPath -Value $cmdContent -Encoding ascii
Write-Ok "啟動指令：$CmdPath"

# 5. 將安裝目錄加入使用者 PATH（若尚未存在）
Write-Step '設定 PATH'
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($userPath -and ($userPath.Split(';') -contains $InstallRoot)) {
    Write-Ok 'PATH 已包含安裝目錄，略過'
} else {
    $newPath = if ($userPath) { "$userPath;$InstallRoot" } else { $InstallRoot }
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    Write-Ok "已加入使用者 PATH（新開的終端機／PowerShell 才會生效）"
}

# 6. 確保既有設定檔不被覆蓋
if (Test-Path $ConfigPath) {
    Write-Warn "已存在設定檔 $ConfigPath，已保留不覆蓋"
} else {
    Write-Ok '尚未設定，首次啟動後可於應用程式內設定 Google Sheet'
}

# 7. 啟動
if ($NoLaunch) {
    Write-Step '安裝完成（未啟動）'
    Write-Host "    之後可在新開的 PowerShell 執行：glucose-dashboard"
} else {
    Write-Step '啟動 Glucose Dashboard'
    Start-Process -FilePath $ExePath
    Write-Ok '已在背景啟動，瀏覽器將自動開啟 Dashboard'
}

Write-Host ''
Write-Host '完成！之後任何新開的 PowerShell 視窗直接輸入「glucose-dashboard」即可啟動。' -ForegroundColor Green