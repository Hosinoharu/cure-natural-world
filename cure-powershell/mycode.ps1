<#
.DESCRIPTION
因为我把 VSCode 的插件从默认的目录 `C:\Users\HOSINO\.vscode\extensions`
移动到了另外的目录 `E:\Microsoft VS Code\installed_extensions\extensions`。

所以用命令行工具 `code` 打开项目时需要指定插件所在的目录，
即使用命令 `code xxx --extensions-dir <dir>`，

本脚本的唯一用法 `mycode xxx` 就是上述命令的简化！
#>
param(
    # VSCode 要打开的项目的目录
    # [Parameter(Mandatory = $true)]
    [string]
    $Path = ".",
    # 不使用我的插件目录，而使用默认的插件目录 `C:\Users\HOSINO\.vscode\extensions`
    [switch]
    $NoMyExtension = $false
)

if (Test-Path -Path $Path) {
    if ($NoMyExtension) {
        code --new-window $Path # --extensions-dir "C:\Users\HOSINO\.vscode\extensions"
    } else {
        # 自己修改的、存放插件的目录记录在了环境变量中哟
        code --new-window $Path --extensions-dir $env:MyVSCodeExtentionPath
    }
} else {
    Write-Host "Error: Folder is not exist" -ForegroundColor Red
}