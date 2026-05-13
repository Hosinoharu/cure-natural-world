"""
借用 PyPDF2 读取 PDF 标签
主要是获取章节等的序号、名字、对应的页数

"""
import PyPDF2 as pdf
from pprint import pprint

def extract(source):
    global outlines
    for item in source:
        if isinstance(item, list):
            extract(item)
        else:
            outlines.append((item.title.strip(), item.page.idnum))


# source_pdf_file = input('输入 PDF 路径: ')
source_pdf_file = 'f:/test/a.pdf'
# output_bookmark_file = input('输入保存为书签文件的路径: ')
output_bookmark_file = 'f:/test/obm.txt'


reader = pdf.PdfFileReader(source_pdf_file)

"""
PDF Reader 对象的 outlines 只读属性[实际上是访问了 .getOutlines()]
重点包含如下属性:
    /Title: 书签名
    /Page: 所在的页，是一个对象
如何确定书签对应 PDF 的页数??
    利用 /Page 属性！因为有 getPage() 获取每一页，与 /Page 比较就知道了!
    即比较 PageObject
如何确定层级关系??
    其实书签名前面就带了层级序号，添加书签时可以解析出层级关系
"""
all_page_num = reader.getNumPages()
old_outlines = reader.getOutlines()  # old_outlines 还包含数组，要全部提取出来

outlines = []
extract(old_outlines)

all_outline_num = len(outlines)

index = 0 # 如果 outlines 匹配了一个，则 index + 1

with open(output_bookmark_file, 'w', encoding='utf-8') as fp:
    for i in range(all_page_num):
        current_page = reader.getPage(i)
        for j in range(index, all_outline_num):
            title, idnum = outlines[j]
            if current_page.indirectRef.idnum == idnum:  # 书签匹配上了该页
                output_text = f'{title} {i + 1}\n' # 书签名 对应的 PDF 页数
                fp.write(output_text)
                print(f'{title} {i + 1}')
                index += 1
                
        
