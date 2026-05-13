from main import *


def read(file: str):
    with open(file, "r", encoding="utf-8") as f:
        return f.read()


def test_get_max_page_count():
    html = read("./test/for_one_page.html")
    assert get_max_page_count(html) == 210


def test_get_pic_id_and_url():
    url = "http://img.netbian.com/file/2024/1115/small003331SFMeU1731602011.jpg"
    want = "http://img.netbian.com/file/2024/1115/003331SFMeU.jpg"
    _, got = get_pic_id_and_url(url)
    assert want == got


def test_get_pic_info():
    html = read("./test/for_one_page.html")
    pics = get_pic_info(html)
    for pic in pics:
        print("[*] Name:", pic.name)
        print("Url:", pic.url)
