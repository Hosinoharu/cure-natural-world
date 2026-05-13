<#
.SYNOPSIS
	快速创建 python 环境。

.DESCRIPTION
	当想要开启一个 python 项目时，步骤如下
		1. 创建目录，并进入其中
		2. 使用 python -m venv .venv 创建虚拟环境
		3. 创建 main.py 文件，代码只有一句 `print("Hello World")`
		4. 启动虚拟环境，并测试 main.py 是否正常工作
	现在就是简化这个步骤。

	使用命令 `create-pythonenv project` 即可快速创建项目 project
 #>
 
[CmdletBinding()]    
param(
	# 项目的目录名称，如果是相对路径则默认是当前所在目录
	[Parameter(Mandatory = $true)]
    [string]
    $folder
)


# 如果 $folder 不存在则创建
if ($folder -and (-not (Test-Path -Path $folder))) {
    New-Item -Path $folder -ItemType Directory | Out-Null
    # 输出绝对路径
    Write-Verbose -Message "成功创建目录: $(Convert-Path -Path $folder)" -Verbose
}


# 创建虚拟环境。如果没有 .venv 目录则创建
$venv = "$folder`/.venv"
if (-not (Test-Path -Path $venv )) {
	# 需要能访问到 python.exe，通常这种方式是访问最新安装的 python 版本
    python -m venv $venv
    Write-Verbose -Message "成功创建虚拟环境: .venv"  -Verbose
    # 创建 main.py 并写入测试代码，启动虚拟环境并测试
    New-Item -Path $folder -Name main.py -Value 'print("Hello World!")' -ItemType File | Out-Null
    & "$venv\Scripts\Activate.ps1"

    Write-Host
    Write-Debug -Message "* * * * * 测试环境 * * * * *" -Debug
    Set-Location -Path $folder | Out-Null
    python main.py
    Write-Host
    Write-Verbose -Message "已进入到环境中"  -Verbose
}
else {
    Write-Error -Message "已存在 .venv 目录，这是已经存在的 python 项目"
}