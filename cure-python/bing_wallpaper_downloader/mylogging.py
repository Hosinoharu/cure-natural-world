"""
我的日志处理
统一保存到 settings.LOGGING_PATH 中，日志名字为当天日期，如 2022-07-17.txt
"""
import logging
from datetime import datetime
from os.path import join


class MyLogging:

    def __init__(self, save_directory, level='DEBUG'):
        """ 默认日志级别是 DEBUG """
        # 当前日志输出的文件路径
        self.log_filename = join(save_directory, MyLogging._generate_filename())

        self.logger = logging.getLogger('default')  # 名为 default 的记录器
        self.logger.setLevel(level)                 # 设置当前记录器的日志等级
        self.level = self.logger.getEffectiveLevel()
        self.default_handler = None
        self.default_formatter = None

        self._add_default_things()
        self.info = self.logger.info
        self.error = self.logger.error
        pass

    def _add_default_things(self):
        """ 添加默认的处理器、格式器。过滤器倒是不用了 """
        if self.default_handler and self.default_handler:  # 已经设置过了
            return

        # 记录日志默认的处理器（输出到文件），默认打开模式是 a，encoding 指定为 utf-8
        self.default_handler = logging.FileHandler(self.log_filename, encoding='utf-8')
        self.default_handler.setLevel(self.level)
        # 给记录器添加处理器
        self.logger.addHandler(self.default_handler)

        # 默认的格式器
        self.default_formatter = logging.Formatter('[%(asctime)s] - %(levelname)s - %(message)s',
                                                   datefmt='%Y-%m-%d %H:%M:%S')
        # 给处理器添加格式器
        self.default_handler.setFormatter(self.default_formatter)
        pass

    def start_function(self, msg):
        """ 函数修饰器，在函数执行时输出指定的信息 """
        def outer(func):
            def inner(*args, **kwargs):
                self.logger.debug(msg)
                return func(*args, **kwargs)
            return inner
        return outer

    @staticmethod
    def _generate_filename():
        """ 根据当前的时间生成日志文件的名字 """
        return datetime.now().strftime('%Y-%m-%d_log.txt')

