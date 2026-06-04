import logging
from pathlib import Path
from urllib import parse
from dataclasses import dataclass
import requests
from curl_cffi import requests as curl_cffi_req
from lxml import etree
from time import sleep

mylog = logging
mylog.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s"
)
HOST = "http://www.netbian.com/"


EXISTED_COUNT = 0
"记录因为图片已经存在而跳过的次数"

MAX_EXISTED_COUNT = 20
"EXISTED_COUNT 超过该次数就停止下载，后续就不用下载了"


@dataclass
class PicInfo:
    id: str
    "图片的 id"
    url: str
    "图片的 url"
    name: str
    "图片的名称"


COOKIE = ""
SESSTION = requests.Session()


def init_cookie():
    "从文件中读取 cookie"
    global COOKIE
    curr = Path(__file__).parent
    cookie_path = curr / "cookie.txt"
    with open(cookie_path, "r") as f:
        COOKIE = f.read().strip()

    mylog.info("Init cookie: %s", COOKIE)


def save_progress(page: int):
    "保存进度，下载到了第几页"
    curr = Path(__file__).parent
    cookie_path = curr / "progress.txt"
    with open(cookie_path, "w") as f:
        f.write(str(page))


def load_progress():
    "加载进度，从第几页开始下载"
    curr = Path(__file__).parent
    cookie_path = curr / "progress.txt"
    if not cookie_path.exists():
        return 1

    with open(cookie_path, "r") as f:
        return int(f.read().strip())


def be_good_name(name: str) -> str:
    "将图片名称中的非法字符替换为下划线"
    invalid_chars = '<>:"/\\|?*'
    for char in invalid_chars:
        name = name.replace(char, "_")
    return name


def request_one_page(url: str, referer: str = HOST):
    "请求一个图片浏览页面，并返回 html"
    response = SESSTION.get(url)
    response.raise_for_status()
    response.encoding = response.apparent_encoding
    return response.text


def get_pic_id_and_url(url: str) -> tuple[str, str]:
    "从图片浏览页面的 url 中提取出图片的 id 和 url"
    # 根据链接如 http://img.netbian.com/file/2024/1115/small003331SFMeU1731602011.jpg
    # 需要删除 small、删除尾部的 10 个数字，得到 003331SFMeU
    # 当然，实践过程中也有链接不符号这个规律 —— 可能是旧的图片接口吧，暂时忽略
    part = url.split("/")[-1].split(".")[0]
    if not part.startswith("small"):
        return "", ""

    _id = part[5:-10]
    return _id, url.replace(part, _id)


def get_max_page_count(html: str) -> int:
    "从 html 中提取出最大页数"
    html = etree.HTML(html)
    # 获取倒数第 2 个 a 标签
    max_page = html.xpath("//div[@class='page']/a[last()-1]/text()")
    if len(max_page) == 0:
        return 0
    return int(max_page[0])


def get_pic_info(html: str) -> list[PicInfo]:
    "从 html 中提取出所有图片的信息"
    html = etree.HTML(html)
    _xpath = '//*[@id="main"]/div[@class="list"]/ul/li/a'
    res = html.xpath(_xpath)

    pics: list[PicInfo] = []
    for t in res:
        # img 标签的 src 属性
        temp_url = t.xpath("./img/@src")[0]
        pic_id, pic_url = get_pic_id_and_url(temp_url)
        if pic_id == "":
            mylog.info(f"Ignore: {temp_url}")
            continue

        # 获取 b 标签的文本
        pic_name: str = t.xpath("./b/text()")[0]
        if pic_name == "":
            pic_name = pic_id
        else:
            # 去除掉所有空白
            pic_name = "".join(pic_name.split())

        pic = PicInfo(pic_id, pic_url, pic_name)
        pics.append(pic)

    return pics


def download_pic(url: str, filename: Path):
    "下载一张图片"
    if filename.exists():
        return
    
    # 不能使用 Session？不清楚原因，就这样吧
    headers = {
        "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
        "Accept-Language": "en",
        "Cache-Control": "no-cache",
        "Connection": "keep-alive",
        "Pragma": "no-cache",
        "Sec-Fetch-Dest": "document",
        "Sec-Fetch-Mode": "navigate",
        "Sec-Fetch-Site": "none",
        "Sec-Fetch-User": "?1",
        "Upgrade-Insecure-Requests": "1",
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36 Edg/148.0.0.0",
        "sec-ch-ua": "\"Chromium\";v=\"148\", \"Microsoft Edge\";v=\"148\", \"Not/A)Brand\";v=\"99\"",
        "sec-ch-ua-mobile": "?0",
        "sec-ch-ua-platform": "\"Windows\""
    }
    r = curl_cffi_req.get(url, headers=headers)
    try:
        r.raise_for_status()
    except Exception as e:
        mylog.error("download_pic error:", e)
        return
    
    with open(filename, "wb") as f:
        f.write(r.content)

    sleep(1)


def download_pics(pics: list[PicInfo], save_path: Path):
    "下载多张图片并保存"
    global EXISTED_COUNT
    for pic in pics:
        filename = save_path / f"{be_good_name(pic.name)}.jpg"
        if filename.exists():
            mylog.info(f"existted {pic.name}")
            EXISTED_COUNT += 1
            if EXISTED_COUNT > MAX_EXISTED_COUNT:
                mylog.info(f"stop download")
                exit()
            continue
        mylog.info(f"Downloading {pic.name}")
        download_pic(pic.url, filename)


def download_one_page(url: str, save_path: Path, max_page: bool = False):
    "下载一页的图片，并保存到指定路径，看是否返回最大页数"
    max_page_count = 0
    html = request_one_page(url, HOST)
    if max_page:
        max_page_count = get_max_page_count(html)
        mylog.info(f"Max page count: {max_page_count}")

    # 获取这一页中所有图片的信息
    mylog.info("Getting all pictures url")
    pics = get_pic_info(html)
    mylog.info(f"Found {len(pics)} pictures, and start to download")
    download_pics(pics, save_path)
    return max_page_count


def download_pages(url: str, save_path: Path):
    "下载多页的图片，并保存到指定路径"
    # 第一页单独下载，可以检测 cookie 是否过期等等
    mylog.info("Downloading page: 1")
    max_page_count = download_one_page(url, save_path, True)

    start_page = load_progress()
    if start_page == 1:
        start_page += 1

    # 准备好字符串模板， 开始循环下载
    for i in range(start_page, max_page_count + 1):
        curr_progress = i
        mylog.info(f"Downloading page: {i} / {max_page_count}")
        url = parse.urljoin(url, f"index_{i}.htm")
        try:
            download_one_page(url, save_path)
        finally:
            save_progress(curr_progress)

        sleep(10)

    print("\nDone")


if __name__ == "__main__":
    mylog.info("Start")
    init_cookie()
    SESSTION.headers.update(
        {
            "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
            "accept-language": "en,zh-CN;q=0.9,zh;q=0.8,en-US;q=0.7",
            "priority": "u=0, i",
            "referer": "https://www.netbian.com/",
            "sec-ch-ua": "\"Chromium\";v=\"148\", \"Microsoft Edge\";v=\"148\", \"Not/A)Brand\";v=\"99\"",
            "sec-ch-ua-mobile": "?0",
            "sec-ch-ua-platform": "\"Windows\"",
            "sec-fetch-dest": "document",
            "sec-fetch-mode": "navigate",
            "sec-fetch-site": "same-origin",
            "sec-fetch-user": "?1",
            "upgrade-insecure-requests": "1",
            "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36 Edg/148.0.0.0",
            "cookie": COOKIE,
        }
    )

    # 主要是下载风景类图片
    pic_url = "https://www.netbian.com/fengjing/"
    save_path = Path(__file__).parent / "pics"
    download_pages(pic_url, save_path)
