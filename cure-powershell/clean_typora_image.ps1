<#
.SYNOPSIS
移出 Typora 附件目录中未曾使用的图片

.DESCRIPTION
在使用 Typora 时，如果是直接粘贴剪切板的照片，
则会将照片放入到一个预定义的文件夹（assets 文件夹）中，然后在 MD 文件中引用它。
如果在 MD 中不使用该图片了，但在 assets 中依然存在，所以本程序就是清除这些多余的图片

如果文件名为 x.md，则附件目录默认是 x.assets 哟

基本方法
    1. 提取出 MD 文件中所有的 .assets/ 后面的文件名，组成集合 a
    2. 获取该 MD 文件的附件文件夹 assets 中所有的图片名，组成集合 b
    3. 执行 b - a，得到集合 c，然后遍历集合 c，删除对应的文件
#>
[CmdletBinding()]
param (
    [Parameter()]
    [string]
    $File = "*"
)

#  检查传入的 markdown 文件名是否有效，并转为 C# 的文件对象
function Test-MDFile ([string]$MDFile) {
    $msg = [string]::Empty

    if (-not (
        (Test-Path -IsValid $MDFile) -and (Test-Path -Path $MDFile -PathType Leaf)
        )) {
        $msg = "Error: 不是有效的文件名。"
    }
    elseif (-not (Test-Path -Path $MDFile)) {
        $msg = "Error: 文件不存在。" 
    }
    elseif (-not ($MDFile.EndsWith(".md"))) {
        $msg = "Error: 不是 MarkDown 后缀的文件。" 
    }

    if ($msg -ne [string]::Empty) {
        Write-Host $msg -ForegroundColor Red
        return $null
    }

    Get-Item -Path $MDFile
}

# 获取 Markdown 文件中使用的所有本地图片的文件名
function Get-ImageFromMDFile ([string]$MDFile) {
    $content = Get-Content -Path $MDFile -Raw

    $pattern = "assets/(.*?)[\)`"]"
    $result = [regex]::Matches($content, $pattern)

    # 如果正则没有匹配到结果，最后 $images 为 $null 哟
    $images = $result | ForEach-Object -Process { $_.Groups[1].Value }
    $images
}

# 获取图片附件目录中的、所有本地图片的文件名
function Get-ImagesFromAssets ([string]$MDAssets) {
    # 如果是空目录则 $images 为 null
    $images = Get-ChildItem -Path $Assets | Select-Object -ExpandProperty Name
    $images
}

# 处理一个 Markdown 文件的冗余图片
function Start-SingleMDFile ([System.IO.FileInfo]$MDFile) {
    if ($MDFile.Length -eq 0) {
        Write-Host "INFO: 空文件！" -ForegroundColor Cyan
        return
    }

    # 确定 Assets 附件的目录
    $Assets = Join-Path -Path $MDFile.Directory -ChildPath "$($MDFile.BaseName).assets"
    if (-not (Test-Path $Assets)) {
        Write-Host "INFO: 没有附件目录！" -ForegroundColor Cyan
        return
    }
    $Assets = Get-Item -Path $Assets

    # 确定垃圾箱目录，其格式为 markdwon 文件名 + .trash 目录
    # 如 /test/main.md 文件对应的垃圾箱为 /test/main.trash 目录
    $trash = Join-Path -Path $MDFile.Directory -ChildPath "$($MDFile.BaseName).trash"


    # 先获取 MD 文件中使用的文件名，构成集合 A
    $ima = Get-ImageFromMDFile -MDFile $MDFile.FullName
    # 然后获取 assets 目录中的文件名，构成集合 B
    $imb = Get-ImagesFromAssets -MDAssets $Assets.FullName


    # 如果 $imb 为 $null，则附件目录中没有图片，故则什么都不做
    # 如果 $ima 为 $null $imb 不为 null，则清空附件目录吗？？不需要，它就相当于垃圾箱了
    if ($ima -and $imb) {  
        # 两个集合计算差集 B - A，然后从 B 中删除多余的内容即可
        $unused_im = Compare-Object -ReferenceObject $ima -DifferenceObject $imb
        | Where-Object SideIndicator -eq "=>"
        | Select-Object -ExpandProperty InputObject

        if ($unused_im) {
            # 先创建垃圾箱目录
            if (-not (Test-Path -Path $trash)) {
                mkdir $trash | Out-Null
            }
            $trash = Get-Item -Path $trash

            # 然后对每个文件处理
            foreach ($item in $unused_im) {
                $file = Join-Path -Path $Assets -ChildPath $item
                Move-Item -Path $file -Destination $trash
                Write-Host "移动图片到垃圾箱: $item => $($trash.Name)"
            }

            return
        }
        
    }
    Write-Host "INFO: 一切安好！" -ForegroundColor Cyan
}

# 循环处理当前目录下所有的 MD 文件！
function  Start-AllMDFile() {
    # 首先获取当前目录下的 MD 文件
    $files = Get-ChildItem -Path . -Filter *.md -File
    foreach ($file in $files) {
        Write-Host "`n===> 处理文件: $($file.Name)`n" -ForegroundColor Green
        Start-SingleMDFile -MDFile $file
    }
}

if ($File -eq "*") {
    Start-AllMDFile
}
else {
    [System.IO.FileInfo]$MDFile = Test-MDFile $File
    if ($MDFile) {
        Start-SingleMDFile -MDFile $MDFile
    }
}