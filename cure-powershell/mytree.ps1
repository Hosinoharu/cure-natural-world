<# 
.DESCRIPTION
我的自定义 `tree` 命令。
`mytree D:\TEMP` 生成 `D:\TEMP` 的目录树，默认最大深度是 10.
`mytree D:\TEMP -Depth 2` 生成 `D:\TEMP` 的目录树，指定深度为 2.
`mytree D:\TEMP -DirOnly` 生成 `D:\TEMP` 的目录树，但忽略文件。
#>
[CmdletBinding()]
param (
    # 要查看的目录
    [Parameter()]
    [string]
    $Path = ".",
    # 目录树的深度
    [Parameter()]
    [int]
    $Depth = 10,
    # 只显示目录而不显示文件
    [Parameter()]
    [switch]
    $DirOnly = $false
)

# 默认的前缀
$BASE_PREFIX = "|  "
# 控制目录的层级，也就是缩进，默认是 4 个空格
$INDENT = "    "

# 处理某个目录下的文件
# function Show-Files {
#     param (
#         [System.IO.FileInfo[]]$File,
#         # 输出这些文件时需要的前缀
#         [string]$Prefix
#     )
#     foreach ($file in $Files) {
#         Write-Host "$Prefix$($File.Name)"
#     }
# }

# 处理某个目录
function Show-Dir {
    param (
        # 目录的路径
        [string]$CurPath,
        # 目录的当前层级
        [int]$CurDepth,
        # 目录所用到的前缀
        [string]$CurPrefix,
        # 是否为某个目录的最后一个目录
        [switch]$IsLast
    )
    <#
        目录在位置上有两种，输出时会选择不同的符号
        默认使用 ├─，但该目录是其父目录的最后一个目录时则使用 └─
        所以这里先确认目录的位置
    #>
    $icon = "├─"
    if ($IsLast) {
        $icon = "└─"
    }
    <#
        确认目录所处的位置后，就可以输出当前目录本身
    #>
    if ($CurDepth -ne 0) {
        # 获取目录名
        $dir_name = Get-ItemPropertyValue -Path $CurPath -Name BaseName
        $dir_info = "$CurPrefix$icon$dir_name"
        Write-Host $dir_info -ForegroundColor Green
    }
    # 超过遍历深度，那么输出当前目录名即可，不需要再处理文件、子目录等
    if ($CurDepth -gt $Depth) {
        return
    }

    # 获取目录下面的目录与文件
    $dirs = Get-ChildItem -Path $CurPath -Directory
    $files = Get-ChildItem -Path $CurPath -File
    <#
        是否有子目录决定了如何展示其下的文件哟。
        所以这里确认目录的是否有子目录、文件。
    #>

    # 一切准备就绪！！
    # 先处理文件，根据目录的位置、是否有子目录会使用不同的前缀来输出哟
    # 先处理文件，根据是否有子目录而不同
    if (-not $DirOnly -and $files) {
        # 零层级特殊对待
        if ($CurDepth -eq 0) {
            if($dirs) {
                $file_prefix = $BASE_PREFIX
            }
            else {
                $file_prefix = ""
            }
        }
        # 当前目录是最后一个目录的情况，此时会使用 "└─"
        elseif ($IsLast) {
            if($dirs) {
                $file_prefix = $CurPrefix + $INDENT + $BASE_PREFIX
            }
            else {
                $file_prefix = $CurPrefix + $INDENT * 2
            }
        }
        # 并非是最后一个目录，此时会使用 "├─"
        else {
            if($dirs) {
                $file_prefix = $CurPrefix + $BASE_PREFIX * 2
            } else {
                $file_prefix = $CurPrefix + $BASE_PREFIX + $INDENT
            }
        }

        foreach ($file in $files) {
            Write-Host "$file_prefix$($File.Name)"
        }

        # 再输出一个空行，避免内容拥挤
        Write-Host $file_prefix
    }

    # 处理子目录
    if ($dirs) {
        # 零层级特殊对待
        if ($CurDepth -eq 0) {
            $sub_prefix = ""
        }
        # 先确定前缀
        elseif ($IsLast) {
            $sub_prefix = $CurPrefix + $INDENT
        }
        else {
            $sub_prefix = $CurPrefix + $BASE_PREFIX
        }

        $sub_depth = $CurDepth + 1

        # 处理最后一个子目录之前的目录
        for ($i = 0; $i -lt ($dirs.Count - 1); $i++) {
            $sub_dir = $dirs[$i]

            Show-Dir -CurPath $sub_dir.FullName -CurDepth $sub_depth -CurPrefix $sub_prefix
        }
        # 处理最后一个目录
        $sub_dir = $dirs[$dirs.Count - 1]
        Show-Dir -CurPath $sub_dir.FullName -CurDepth $sub_depth -CurPrefix $sub_prefix -IsLast
    }
}


# main
$fullname = Get-ItemPropertyValue -Path $Path -Name FullName

Write-Host "`n* MyTree to " -NoNewline
Write-Host "$fullname" -BackgroundColor DarkGreen -NoNewline
Write-Host "`n"

Show-Dir -CurPath $fullname -CurDepth 0 -CurPrefix ""