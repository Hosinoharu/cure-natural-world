"""
保存全局设置，相比较使用 .ini 文件来说更方便
"""

from os import makedirs
from sys import argv
from os.path import realpath, dirname, join, exists

######################################################
# 网络访问的设置
######################################################
BIYING_HOME = 'https://cn.bing.com'
# 默认请求头能写完整的就写完整的，不要省略啦！
DEFAULT_HEADERS = {'User-Agent' : 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.5112.81 Safari/537.36 Edg/104.0.1293.47'}


######################################################
# 弹窗提示时使用
######################################################
BAD_TITLE = 'ooops! Something Wrong'


# 获取本 py 文件所在的目录
BASED_SAVE_DIRECTORY = dirname(realpath(__file__))

# 保存下载的必应壁纸的目录
# WALLPAPER_DIRECTORY = join(BASED_SAVE_DIRECTORY, 'wallpapers')
# 使用原来的那个
WALLPAPER_DIRECTORY = r'F:\BiYingWallpaper（已经废弃）\test'

# 保存日志的目录。日志的名字都是以日期来命名的，如【2022-07-17】等
LOGGING_DIRECTORY = join(BASED_SAVE_DIRECTORY, 'logging')

# 保存 request 的内容的目录，文件名使用日期来命名，如【2022-08-18】等
REQUEST_CONTENT_DIRECTORY = join(BASED_SAVE_DIRECTORY, 'request_content')

STATUS_FILE = join(BASED_SAVE_DIRECTORY, 'status.pickle')







def init_all_directory():
    """ 保证设置里面的目录都已存在 """

    def crate_directory(directory):
        if not exists(directory):
            makedirs(directory)
        pass

    crate_directory(BASED_SAVE_DIRECTORY)
    crate_directory(WALLPAPER_DIRECTORY)
    crate_directory(LOGGING_DIRECTORY)
    crate_directory(REQUEST_CONTENT_DIRECTORY)


init_all_directory()


