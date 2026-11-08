# download_onnxruntime.ps1
# 下载 ONNX Runtime 1.31.0 Windows x64（ort-sys 2.0.0-rc.15 依赖）
# 用法：.\scripts\download_onnxruntime.ps1

$ErrorActionPreference = "Stop"

$OrtVersion = "1.31.0"
$OrtZip = "onnxruntime-win-x64-$OrtVersion.zip"
$DownloadUrl = "https://github.com/microsoft/onnxruntime/releases/download/v$OrtVersion/$OrtZip"
$ProxyUrl = "https://gh-proxy.org/$DownloadUrl"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Resolve-Path "$ScriptDir/.."
$OrtDir = "$ProjectRoot/models/onnxruntime"

# 代理配置
$env:HTTP_PROXY = "http://10.89.122.10:13873"
$env:HTTPS_PROXY = "http://10.89.122.10:13873"

Write-Host "=== 大禹记忆系统 · ONNX Runtime 下载 ===" -ForegroundColor Cyan
Write-Host "版本: $OrtVersion"
Write-Host "目标: $OrtDir"
Write-Host "代理: $ProxyUrl"
Write-Host ""

# 检查是否已存在
if (Test-Path "$OrtDir/lib/onnxruntime.dll") {
    Write-Host "[SKIP] ONNX Runtime 已存在: $OrtDir" -ForegroundColor Yellow
    exit 0
}

# 下载
$ZipPath = "$env:TEMP/$OrtZip"
Write-Host "[DOWNLOAD] $OrtZip ..." -ForegroundColor Green

try {
    Invoke-WebRequest -Uri $ProxyUrl -OutFile $ZipPath -UseBasicParsing
    $Size = (Get-Item $ZipPath).Length
    Write-Host "  -> 完成 ($([math]::Round($Size/1MB, 2)) MB)" -ForegroundColor Green
} catch {
    Write-Host "  -> 代理下载失败，尝试直连 ..." -ForegroundColor Yellow
    try {
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing
        Write-Host "  -> 直连完成" -ForegroundColor Green
    } catch {
        Write-Host "  -> 下载失败: $_" -ForegroundColor Red
        exit 1
    }
}

# 解压
Write-Host "[EXTRACT] 解压到 $OrtDir ..." -ForegroundColor Green
Expand-Archive -Path $ZipPath -DestinationPath "$ProjectRoot/models" -Force

# 重命名（解压后目录名可能是 onnxruntime-win-x64-1.31.0）
$ExtractedDir = Get-ChildItem -Path "$ProjectRoot/models" -Directory | Where-Object { $_.Name -like "onnxruntime-win-x64-*" } | Select-Object -First 1
if ($ExtractedDir) {
    if (Test-Path $OrtDir) { Remove-Item $OrtDir -Recurse -Force }
    Move-Item -Path $ExtractedDir.FullName -Destination $OrtDir -Force
}

# 清理
Remove-Item $ZipPath -Force

# 验证
Write-Host ""
Write-Host "=== 验证 ===" -ForegroundColor Cyan
$DllPath = "$OrtDir/lib/onnxruntime.dll"
if (Test-Path $DllPath) {
    $DllInfo = Get-Item $DllPath
    Write-Host "  ✅ onnxruntime.dll ($([math]::Round($DllInfo.Length/1MB, 2)) MB)" -ForegroundColor Green
} else {
    Write-Host "  ❌ onnxruntime.dll 未找到！" -ForegroundColor Red
    Get-ChildItem $OrtDir -Recurse -File | ForEach-Object { Write-Host "    $($_.FullName)" }
}
