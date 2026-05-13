"""
重写之前的【下载每日必应壁纸并作为桌面】的代码，减少一些没必要的弹窗输出
加入【日志】功能，使用的 Python 环境是全局虚拟环境 Python3.10.2
"""

import requests
import pickle
# from re import search
from datetime import datetime
from ctypes import windll  # 用于将下载到的图片设置为壁纸
from bs4 import BeautifulSoup
from os.path import exists, join

import settings
from mylogging import MyLogging  # 我的默认日志记录
from win10toast import ToastNotifier

log = MyLogging(settings.LOGGING_DIRECTORY)  # 所有日志都记录到此文件夹中


#####################################################################################
# ========================= 网络访问 =================================================
#####################################################################################

def get_content(mode=None, **kwargs):
    """访问网络，获取内容"""
    r = requests.get(**kwargs)  # 传入参数，直接访问
    r.raise_for_status()
    r.encoding = r.apparent_encoding

    if mode == 't':  # 返回文本
        return r.text
    return r


def save_request_content(request_content):
    """将发出请求后的内容写入到文件，便于后期查错"""
    filename = join(settings.REQUEST_CONTENT_DIRECTORY,
                    datetime.now().strftime('%Y-%m-%d_%H-%M-%S_request_content.txt'))
    with open(filename, 'w', encoding='utf-8') as fp:
        fp.write(request_content)
    
    return


def yield_binary_content(per_yield_size=102400, **kwargs):
    """下载大文件时可以使用流模式"""
    r = get_content(stream=True, **kwargs)  # 流模式获取请求
    for item in r.iter_content(per_yield_size):  # 默认每次下载 102400B，即 100 KB
        yield item


#####################################################################################
# ========================= 时间处理，保证每天只下载一次 =================================
#####################################################################################
@log.start_function('检查是否已经下载过今天的壁纸')
def getStatus(statusFile):
    """
    获取 statusFile 中的内容，判断离上一次下载壁纸的时间是否过了一天
    返回 True 表示下载壁纸
    """
    if not exists(statusFile):  # status 文件不存在，默认还没有下载今天的壁纸
        return True
    with open(statusFile, 'rb') as fp:
        return (datetime.now().date()  # 当前时间
                -
                pickle.load(fp).get('time')  # 上一次下载壁纸的时间
                ).days >= 1
    pass


def setStatus(statusFile):
    with open(statusFile, 'wb') as fp:
        pickle.dump({"time": datetime.now().date()}, fp)
    pass


#####################################################################################
# ========================= 对 HTML 代码解析，获取图片名和下载链接 =======================
#####################################################################################
@log.start_function('开始解析 HTML 获取图片名字')
def extract_img_name_from_html(soup):
    """ 解析 html 代码获取图片名 """
    img_name = soup.select_one('div.musCardCont a.title').text.strip()
    return img_name


@log.start_function('开始解析 HTML 获取图片下载链接')
def extract_img_url_from_html(soup):
    """ 解析 html 代码获取图片下载链接 """
    img_url = soup.select_one('#preloadBg').get('href')
    return img_url


#####################################################################################
# ========================= 图片的文件名处理 ===========================================
#####################################################################################

def be_good_name(name, char='_'):
    """检查文件名是否包含非法字符，如果包含默认用 _ 替换掉"""
    for i in set('\/:*?"<>|'):
        if i in name:
            name = name.replace(i, char)
    return name


#####################################################################################
# ========================== 设置通知和壁纸 ============================================
#####################################################################################


def showNotification(title, showText, duration=5):
    toaster = ToastNotifier()
    toaster.show_toast(title, showText, duration=duration,
                       icon_path=join(settings.BASED_SAVE_DIRECTORY + '\\data\\good.ico'))


@log.start_function('设置壁纸')
def setWallpaper(picture):
    # 我的实验证明，参数 4 设为 1，而不是 0
    windll.user32.SystemParametersInfoW(20, 0, picture, 1)
    pass


if __name__ == '__main__':
    log.info("********** 程序启动 **********")

    # 检查是否已经下载过今天的壁纸
    if not getStatus(settings.STATUS_FILE):
        log.info('今天已经下载过壁纸啦！')
        showNotification('Hi', '主人, 今天的任务已经完成了哟, 一起努力吧!')
        log.info('完美退出')
        exit()

    log.info("开始下载壁纸")

    showNotification('Time For Wallpaper', '主人, 开始下载壁纸啦!')

    try:
        r = get_content(url=settings.BIYING_HOME, headers=settings.DEFAULT_HEADERS)
        save_request_content(r.text)  # 无论有没有出错，只要访问了网络就保存
        r.raise_for_status()
    except Exception as e:
        log.error(e)
        showNotification(settings.BAD_TITLE, '获取网页源代码出错!请主人查看日志', duration=15)
        exit()

    try:
        soup = BeautifulSoup(r.text, 'html.parser')
    except Exception as e:
        log.error(e)
        showNotification(settings.BAD_TITLE, f'解析网页源代码出错!请主人查看日志', duration=15)
        exit()

    ################################################
    # 解析 html，因为网页改动，这两个函数会随时修改逻辑
    ################################################
    try:
        imgName = extract_img_name_from_html(soup)
        imgUrl = extract_img_url_from_html(soup)
        if not imgName or not imgUrl:
            log.error(f'没有提取到图片名字、下载链接，以下是网页 HTML:\n{r.text}')
            showNotification(settings.BAD_TITLE, f'没有提取到图片名字、下载链接', duration=15)
            exit()
            
        # 2022/11/28 改动，图片下载连接发生了改变
        # imgUrl = settings.BIYING_HOME + imgUrl  # 2022/12/06 取消改动
    except Exception as e:
        log.error(e)
        showNotification(settings.BAD_TITLE, f'获取图片名字、链接出错!快查看日志解决问题吧', duration=15)
        exit()

    showNotification('下载中 . . .', '图片名: {}'.format(imgName))

    log.info('开始下载图片')

    # 保存图片时所用完整路径
    filename = join(settings.WALLPAPER_DIRECTORY, be_good_name(imgName) + '.jpg')
    try:
        with open(filename, 'wb') as fp:
            for chunk in yield_binary_content(url=imgUrl, headers=settings.DEFAULT_HEADERS):
                fp.write(chunk)
    except Exception as e:
        log.error(e)
        showNotification(settings.BAD_TITLE, f'下载图片出错!主人, 快查看日志解决问题吧', duration=15)
        exit()

    log.info('下载壁纸成功！')

    # setWallpaper(filename)

    showNotification('Time For GoodBye', '主人, 明天见哟, 我会想你的', duration=8)

    setStatus(settings.STATUS_FILE)  # 已经下载过今天的壁纸，设置状态
    log.info('完美退出')
    exit()
