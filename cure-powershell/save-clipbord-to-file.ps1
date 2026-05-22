<#
.SYNOPSIS
将剪贴板文本内容保存到文件

.DESCRIPTION
读取系统剪贴板中的文本内容，保存到指定路径的文件
如果文件所在目录不存在，会自动创建

.PARAMETER Path
目标文件路径（必填）

.EXAMPLE
.\save-clipboard-to-file.ps1 C:\output.txt
将剪贴板内容保存到 C:\output.txt
#>

param(
    [Parameter(Mandatory=$true)]
    [string]$Path
)

$content = Get-Clipboard

if ([string]::IsNullOrEmpty($content)) {
    Write-Host "No Content In Clipboard" -ForegroundColor Yellow
    return
}

$dir = Split-Path $Path -Parent
if ($dir -and -not (Test-Path $dir)) {
    New-Item -ItemType Directory -Path $dir -Force | Out-Null
}

$content | Out-File -FilePath $Path -Encoding UTF8
