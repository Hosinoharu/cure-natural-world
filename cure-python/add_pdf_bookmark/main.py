"""
给 PDF 添加书签。具体说明见 readme.md 文档
"""

import re  # 正则表达式，用于处理目录文件
import pathlib  # 处理路径

import PyPDF2 as pdf  # 读写 PDF


# done
def test_chapter_pattern(chapter) -> str:
    """
    根据文章的标题，返回正确的、处理章节名的正则。

    因为章节名有不同情况，所以需要不同的正则表达式，现在就是根据章节信息 chapter
    来测试、找到适合的正则表达式
    """
    patterns = (
        r"第.*?章",  # 用于中文书籍的正则，如 第1章、第一章
        r"^\d{1,2}",  # 用于英文书籍的正则，因为章节以数字开始
    )
    for p in patterns:
        if re.search(p, chapter):
            return p
    return ""


# done
def yield_bookmark(cat_file: pathlib.Path):
    """
    核心之核心，解析目录文件，生成: 标签级数、标签名、页号（实际书签在 PDF 文件中的页数。

    @params
        cat_file: 目录文件的路径
    """

    # 用于判断章节的正则，它有多种情况，后面会自动找出合适的正则
    chapter_pattern = ""
    """ 
    用于判断子标签的正则，通常是 [1.1 xxx]、[1.1.1 xxxx]，即都是数字 x.y.z 的形式
    通常最多也只有 3 层子目录，但至少也是 x.y 的形式
    """
    sub_pattern = r"""
        (\d+)\.  # 第一个数字必须存在，并且含有一个小数点才行，这样才是一个子标签哟
        (\d+)\.? # 第二个数字也必须存在。
        (\d*)   # 最多有 3 个数字，
    """

    with cat_file.open("r", encoding="utf") as fp:
        gap = int(fp.readline())  # 读取第一行，为页偏移

        for line in fp:  # 此时文件指针已经移动到下一行了
            line = line.strip()  # 去掉两侧的空格
            if not line:  # 空白行，跳过
                continue

            """ 
            rank = 0，默认放在最高层
            rank = 1, 则是章节层级，虽然也是最高层，但添加标签后要保存该标签，以便作为后续标签的父标签
            rank = 2, 二级标签，也要保存该标签
            rank = 3，三级标签，如果它是最底层的标签，那就不需要保存该标签
            ......
            """
            rank = 0  # 标签级数，默认为 0 表示最高级的标签

            # 如果当前行并没有页数，那就默认设置为当前内容、第 1 页
            title = line  # 标签名
            page = 1  # 页号

            ###########################################################
            # 核心之处: 不同的目录文件请修改这里
            ###########################################################
            if chapter_pattern == "":  # 还没有确定章节的正则，那就测试一下
                # 此时 chapter_pattern 可能为 "" 或者某个符合要求的正则
                # 为 "" 说明当前行可能不是章节行哟
                chapter_pattern = test_chapter_pattern(line)

            # 当前行是章节标签，则 rank = 1
            if chapter_pattern != "" and re.search(chapter_pattern, line):
                rank = 1
            else:
                # 因为 sub_pattern 时多行字符串，所以使用 re.X
                if result := re.search(sub_pattern, line, flags=re.X):
                    if result.group(3):  # 有内容，则表示匹配到了 1.2.3 中的 3，所以是三级标签
                        rank = 3
                    else:
                        rank = 2

            # 匹配数字结尾，即页号，可包含负数，如 1.1 xxxx 9
            if info := re.search(r"(.*?)(-?\d+$)", line):
                title, page = info.groups()
                title = title.strip()  # 去掉两侧的空白
                page = int(page) + gap

            yield rank, title, page


def create_pdfWriter(pdf_file: pathlib.Path):
    """
    创建 PDF Writer，如果原 PDF 已经有标签，通过本方法创建的 PDF 是没有标签的
    如果使用 writer.cloneDocumentFromReader 会复制原 PDF 的标签，从而导致添加标签时出错
    """
    reader = pdf.PdfReader(str(pdf_file))
    writer = pdf.PdfWriter()  # 创建写对象
    for num in range(len(reader.pages)):  # 逐页复制到 pdfWriter
        page = reader.pages[num]
        writer.add_page(page)
    return writer


# done
def test_catalog(cat_file: pathlib.Path):
    """
    用于显示程序提取出标签名、标签页数以及标签的层级关系，并人为校验是否正确

    @params
        cat_file: 目录文件的路径
    """

    count = 0  # 用于统计有多少个标签
    """ 
        rank: 标签的层级，有 0、1、2、3 个级别
        title: 标签的名字
        page: 实际要插入的页数
    """
    for rank, title, page in yield_bookmark(cat_file):
        count += 1
        # 不同层级的标签，有不同的缩进，从而体现层级关系
        # rank 为 0 和 1 时都是顶层标签
        indent = {0: "", 1: "", 2: " " * 4, 3: " " * 8}
        print(f"{indent[rank]}{title} - {page}")

    print(f"\n共 {count} 个标签")


def main(source: pathlib.Path, des: pathlib.Path, cat_file: pathlib.Path):
    """
    :source: 原 PDF
    :des: 新 PDF
    :cat_file: 目录文件
    """
    pdfWriter = create_pdfWriter(source)

    # ====== 添加标签 =========
    # 表示最近的上一级标签(不是同级哟)，因为 3 是最低级，所以没有 3，这样添加标签时可以找到最近的上一级
    recent = {
        1: None,
        2: None,
    }
    for rank, title, page in yield_bookmark(cat_file):
        """
        rank 表示当前标签的层级，要判断，是否保存它作为其他标签的父标签
        比如保存章节标签，这样，到了 1.2 这样的小节，还是能添加到章节标签中取地
        """
        # 获取当前标签的父标签
        # 若 rank 为 0, 1，都是最高级标签，没有上一级标签，所以返回 None
        # 若 rank 为 2, 3，则父标签就是最近添加的 recent[rank - 1]，即最近添加的它们的上一级标签
        parent_mark = recent.get(rank - 1, None)
        last_mark = pdfWriter.add_outline_item(
            # 实际上插入标签时的页数一定要减 1，因为编程中从 0 开始计数
            title, page - 1, parent_mark
        )  # 添加当前标签之后，作为了当前层级的最新的标签
        if rank == 1 or rank == 2:
            # 保存最近添加的标签，因为 0 是最高层、3 是最底层，都不需要再保存了，因为它们不会用作是父标签
            recent[rank] = last_mark

    # ===== 写入文件 =====
    with des.open("wb") as fout:
        pdfWriter.write(fout)


def check_file(path: pathlib.Path, err_info: str):
    if not path.exists():
        raise ValueError(err_info)


if __name__ == "__main__":
    # 一些相关的东西都放在该目录中哟，即当前项目所在目录的 "data" 目录中
    target_path = pathlib.Path(__file__).parent / "data"

    input(f"请确保目录文件（设置了页偏移）已经按照要求放在:\n\t{str(target_path)}\n")

    # PDF 源文件，要给它添加书签，规定了名字，需要将原来的 PDF
    pdf_file = pathlib.Path(input("输入源 PDF 文件的路径[程序未检查路径正确性哟]:"))
    des_pdf_file = target_path / "output.pdf"  # 最终要生成的 PDF 新文件
    cat_file = target_path / "catalog.txt"  # 目录文件，也固定了它的名字

    # 测试文件是否存在
    check_file(pdf_file, "源 PDF 文件不存在！")
    check_file(cat_file, "目录文件不存在！")

    # 测试目录文件是否正确，以此测试 yieldBookmark
    test_catalog(cat_file)

    input("确认标签正确后按 <Enter> 输出 . . . ")

    main(pdf_file, des_pdf_file, cat_file)
