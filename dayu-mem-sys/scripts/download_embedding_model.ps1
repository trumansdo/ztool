# download_embedding_model.ps1
# 下载 pizza-embed-zh-small-v1 嵌入模型（ONNX 格式）
# 来源：HuggingFace infiniLabs/pizza-embed-zh-small-v1
#
# 用法：.\scripts\download_embedding_model.ps1
# 输出：models/infinilabs/pizza-embed-zh-small-v1/
#
# 优先使用 huggingface_hub CLI（--local-dir 下载真实文件，非软链接），
# 不可用时回退到 Invoke-WebRequest 直链下载。

param(
    [string]$ModelName = "infiniLabs/pizza-embed-zh-small-v1",
    [string]$OutputDir = "models/infinilabs/pizza-embed-zh-small-v1"
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Resolve-Path "$ScriptDir/.."
Set-Location $ProjectRoot

# ============================================================
# 镜像配置
# ============================================================
$env:HF_ENDPOINT = "https://hf-mirror.com"
$PipIndex = "https://pypi.tuna.tsinghua.edu.cn/simple"

Write-Host "=== 大禹记忆系统 · 嵌入模型下载 ===" -ForegroundColor Cyan
Write-Host "模型: $ModelName"
Write-Host "输出: $OutputDir"
Write-Host "镜像: $env:HF_ENDPOINT"
Write-Host ""

# ============================================================
# 方法 1：huggingface_hub CLI（推荐）
# ============================================================
function Download-WithHfCli {
    Write-Host "[INFO] 尝试 huggingface_hub CLI ..." -ForegroundColor Cyan

    # 检查 hf 命令是否可用
    $hfCmd = Get-Command hf -ErrorAction SilentlyContinue
    if (-not $hfCmd) {
        Write-Host "  hf 命令不可用，尝试安装 huggingface_hub ..." -ForegroundColor Yellow
        try {
            pip install -U huggingface_hub -i $PipIndex 2>&1 | Out-Null
            Write-Host "  huggingface_hub 安装完成" -ForegroundColor Green
        } catch {
            Write-Host "  pip 安装失败: $_" -ForegroundColor Red
            return $false
        }
    }

    # 确保输出目录存在
    New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

    Write-Host "  下载中 ..." -ForegroundColor Green
    try {
        # --local-dir 下载真实文件（非软链接），与 git-lfs 行为一致
        hf download $ModelName --local-dir "$OutputDir" 2>&1
        Write-Host "  huggingface_hub 下载完成" -ForegroundColor Green
        return $true
    } catch {
        Write-Host "  hf download 失败: $_" -ForegroundColor Red
        return $false
    }
}

# ============================================================
# 方法 2：Invoke-WebRequest 直链下载（回退）
# ============================================================
function Download-WithWebRequest {
    Write-Host "[INFO] 回退到 Invoke-WebRequest 直链下载 ..." -ForegroundColor Cyan

    New-Item -ItemType Directory -Force -Path "$OutputDir/onnx" | Out-Null

    # ONNX 模型文件（在 onnx/ 子目录）
    $OnnxFiles = @(
        "onnx/model.onnx",
        "onnx/model.onnx.data"
    )

    foreach ($File in $OnnxFiles) {
        $Url = "$env:HF_ENDPOINT/$ModelName/resolve/main/$File"
        $Output = "$OutputDir/$File"

        if (Test-Path $Output) {
            Write-Host "[SKIP] $File 已存在" -ForegroundColor Yellow
            continue
        }

        Write-Host "[DOWNLOAD] $File ..." -ForegroundColor Green
        try {
            Invoke-WebRequest -Uri $Url -OutFile $Output -UseBasicParsing
            $Size = (Get-Item $Output).Length
            Write-Host "  -> 完成 ($([math]::Round($Size/1MB, 2)) MB)" -ForegroundColor Green
        } catch {
            Write-Host "  -> 失败: $_" -ForegroundColor Red
            Remove-Item $Output -ErrorAction SilentlyContinue
        }
    }

    # Tokenizer 文件（在根目录）
    $TokenizerFiles = @(
        "tokenizer.json",
        "tokenizer_config.json"
    )

    foreach ($File in $TokenizerFiles) {
        $Url = "$env:HF_ENDPOINT/$ModelName/resolve/main/$File"
        $Output = "$OutputDir/$File"

        if (Test-Path $Output) {
            Write-Host "[SKIP] $File 已存在" -ForegroundColor Yellow
            continue
        }

        Write-Host "[DOWNLOAD] $File ..." -ForegroundColor Green
        try {
            Invoke-WebRequest -Uri $Url -OutFile $Output -UseBasicParsing
            $Size = (Get-Item $Output).Length
            Write-Host "  -> 完成 ($([math]::Round($Size/1KB, 1)) KB)" -ForegroundColor Green
        } catch {
            Write-Host "  -> 失败: $_" -ForegroundColor Red
            Remove-Item $Output -ErrorAction SilentlyContinue
        }
    }

    return $true
}

# ============================================================
# 主流程
# ============================================================

$success = Download-WithHfCli
if (-not $success) {
    $success = Download-WithWebRequest
}

# ============================================================
# 验证
# ============================================================
Write-Host ""
Write-Host "=== 下载完成 ===" -ForegroundColor Cyan
Write-Host "验证："
Get-ChildItem $OutputDir -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object {
    $size = if ($_.Length -gt 1MB) { "$([math]::Round($_.Length/1MB, 2)) MB" } else { "$([math]::Round($_.Length/1KB, 1)) KB" }
    Write-Host "  $($_.FullName.Replace($ProjectRoot.Path, '')) ($size)"
}

if (-not (Get-ChildItem $OutputDir -Recurse -File -ErrorAction SilentlyContinue)) {
    Write-Host "  ⚠️ 未找到任何文件，下载可能失败" -ForegroundColor Red
}
