// ==UserScript==
// @name         视频播放快捷键
// @namespace    http://tampermonkey.net/
// @version      0.1
// @description  观看视频时定义一些快捷键方便操作
// @author       hosinoharu
// @match        *://**/*
// @grant        none
// @run-at       document-start
// ==/UserScript==

/* global window,document,console,setTimeout,location */

/* #cure-按键的绑定逻辑
    注意，视频的快捷键事件实际绑定在 document 上，而不是 <video> 标签上，
    这样不需要聚焦到视频元素上就可以使用快捷键了，使用起来更方便一些。
    
    事件触发时会操作对应的 video 标签。

    在网站中可能有多个 <video> 标签，定义一个上下按钮，用于切换视频控件。
    默认情况下，给第一个 <video> 标签绑定快捷键。
    如果不是该 <video> 标签，则按快捷键，切换到下一个 <video> 标签，并绑定快捷键。
    如此反复，直到找到正确的 <video> 标签。

    也可以用一个快捷键绑定到当前正在播放的 <video> 标签上，这样就不需要切换了。

    总之，这里的问题是【如何定位到目标的 video】，有以下几个策略
    - 当前播放的 video 标签
    - 宽高最大的 video 标签
*/

/**
* 快捷键操作如下

* q: 暂停。属于 keyup
*
* w: 音量增加 10。属于 keydown
*
* s: 音量减少 10。属于 keydown
*
* m: 音量为 0.属于 keyup
*
* a: 后退 5s。属于 keydown，如果同时按下 ctrl，则只后退 1s。
*
* d：前进 5s。属于 keydown，如果同时按下 ctrl，则只前进 1s。
*
* e: 切换全屏。属于 keyup
*
* r: picture in picture，即视频悬窗。属于 keyup
*
* * * * * * 倍速相关  * * * * * *
*
* c: 倍速增加 0.25。属于 keyup
*
* x: 倍速减少 0.25。属于 keyup
*
* z: 倍速直接到 2。属于 keyup
 */
(() => {
    "use strict";

    const log = console.log;
    /** 在获取 video 控件失败时，会进行循环操作的最大次数 */
    const MAX_QUERY_COUNT = 10;
    /** document 触发 key 事件时，需要忽略这些目标元素，其实就是可以输入文本的元素 */
    const SKIP_ELEMS = ["input", "textarea"];
    /** 最大放大倍数 */
    const MaxScale = 10;

    /** 记录当前网页的所有 video 标签 */
    let videos = [];
    /** 快捷键事件实际要处理的 <video> 标签哟 */
    let current_video = null;
    /** 记录当前是切换到控制第几个 video 标签了 */
    let current_index = 0;
    let my_html_policy;
    /** 是否启用快捷键。当输入内容时可能需要禁用快捷键啦 */
    let enable = true;

    /** 部分网站可能无法注入 innerHTML，只好这样了 */
    function createHTML(s) {
        if (my_html_policy === undefined) {
            my_html_policy = window.trustedTypes?.createPolicy(
                "my_html_policy",
                {
                    createHTML: (s) => s,
                },
            );
        }
        return my_html_policy ? my_html_policy.createHTML(s) : s;
    }

    /** 在页面的右下角创建对应的按钮元素，用于切换不同的 <video> 标签 */
    async function init_html_elem() {
        const div = document.createElement("div");
        div.id = "video-controller-container";
        div.innerHTML = createHTML(`
            <p id="video-controller">video shortcuts</p>
            <div id="video-controller-control">
                <button type="button" id="my_prev_video_btn" class="vbtn"><</button>
                <label id="my_video_info" title="点击重新获取 video 元素，双击定位到当前正在播放的 video">click</label>
                <button type="button" id="my_next_video_btn" class="vbtn">></button>
            </div>
        `);

        const style = document.createElement("style");
        style.textContent = `
            #video-controller-container {
                position: fixed;
                bottom: 10px;
                right: 0;

                font-size: 20px;
                color: #ccc;
                width: 220px;
                background-color: #333;
                border: 2px solid orange;
                border-radius: 10px;
                padding: 5px 0;

                display: none;
                z-index: 999999;
            }

            #video-controller-container * {
                padding: 0;
                margin: 0;
                text-align: center;
            }

            #video-controller-container .vbtn {
                width: 50px;
                margin: 0 8px;
                color: yellow;
                background-color: transparent;
                border: 1px solid #888;
            }

            #video-controller {
                margin-bottom: 5px;
            }
        `;

        document.body.appendChild(style);
        document.body.appendChild(div);

        // 默认情况下，给第一个 video 标签绑定快捷键
        await update_video_info(current_index);

        // 绑定事件
        document
            .querySelector("#my_prev_video_btn")
            .addEventListener("click", async () => {
                // 已经是第一个了，则来到最后一个
                if (current_index <= 0) {
                    current_index = videos.length - 1;
                } else {
                    current_index--;
                }
                await update_video_info(current_index);
            });
        document
            .querySelector("#my_next_video_btn")
            .addEventListener("click", async () => {
                // 已经是最后一个了，则来到第一个
                if (current_index >= videos.length - 1) {
                    current_index = 0;
                } else {
                    current_index++;
                }
                await update_video_info(current_index);
            });

        // #cure-tip 点击中间的 label 标签重新获取页面的 video 元素
        // 如果是双击触发，则定位到当前正在播放的 video
        document
            .querySelector("#my_video_info")
            .addEventListener("click", async (e) => {
                e.preventDefault();
                // 双击
                if (e.detail >= 2) {
                    const i = videos.findIndex((v) => !v.paused);
                    if (i !== -1) {
                        current_index = i;
                        await update_video_info(current_index);
                    }
                } else {
                    videos = Array.from(document.querySelectorAll("video"));
                    current_index = 0;
                    await update_video_info(current_index);
                }
            });

        await init_little_pannel();
    }

    /** 更新切换的视频信息，比如共有 3 个 video 标签，现在切换到了 2/3，即第二个 video 标签啦 */
    async function update_video_info(current_index) {
        current_video = videos[current_index];
        zoomer(current_video);
        // 更新文本
        document.querySelector("#my_video_info").innerText =
            `${current_index + 1} / ${videos.length}`;
        // 高亮一下视频元素吧
        const v = current_video;
        const raw_zindex = v.style.zIndex;
        const raw_border = v.style.border;
        v.style.border = "5px solid red";
        v.style.zIndex = 99999;
        // 2s 后取消高亮
        setTimeout(() => {
            v.style.border = raw_border;
            v.style.zIndex = raw_zindex;
        }, 2000);
    }

    /** 默认情况下，这个视频选择窗口隐藏，如果发现了视频则仅展示一个小 pannel。
     *
     * 鼠标放到小 pannel 上面时才显示真正的控制界面。
     *
     * 鼠标离开控制界面时又回到小 pannel 状态。
     */
    async function init_little_pannel() {
        // 如果有视频时，才显示这个小弹窗哟
        if (videos.length <= 0) {
            return;
        }

        // 当点击 label 时，隐藏该弹窗，显示一个小弹窗，当点击小弹窗时才显现整个大弹窗
        const little_div = document.createElement("div");
        little_div.id = "side-pannel";
        little_div.textContent = "<";
        little_div.style.cssText = `
            position: fixed;
            bottom: 10px;
            right: -5px;
            color: white;
            font-size: 16px;
            height: 40px;
            line-height: 40px;
            background-color: orange;
            border-radius: 5px;
            display: none;
            padding: 0 5px;
            display: block;
            z-index: 999998;
        `;
        document.body.appendChild(little_div);

        const video_shortcuts_container = document.querySelector(
            "#video-controller-container",
        );
        video_shortcuts_container.addEventListener("mouseleave", () => {
            video_shortcuts_container.style.display = "none";
            little_div.style.display = "block";
        });

        little_div.addEventListener("mousemove", () => {
            video_shortcuts_container.style.display = "block";
            little_div.style.display = "none";
        });
    }

    /** 给 document 绑定快捷键，控制当前视频元素的播放。
     * @param {HTMLVideoElement} video
     */
    async function init_document_event() {
        /** 如果当前焦点在 input 等标签中（输入文本），则不处理事件 */
        function should_skip_elem() {
            const node_name = document.activeElement.nodeName.toLowerCase();
            return SKIP_ELEMS.indexOf(node_name) !== -1;
        }

        /** keydown 事件处理函数
         * @param {KeyboardEvent} event
         */
        function keydown_handler(event) {
            if (!current_video) {
                return;
            }
            if (
                !enable ||
                event.ctrlKey ||
                event.altKey ||
                event.shiftKey ||
                event.metaKey
            ) {
                return;
            }
            if (should_skip_elem()) {
                return;
            }

            // 记录这个按键是否经过了处理，如果经过了处理则要避免和网站的快捷键冲突
            let is_worked = true;
            // 默认调整播放时间时，以 5s 为单位
            const time_step = 5;
            switch (event.key.toLowerCase()) {
                // 音量增加 10%
                case "w":
                    if (current_video.volume > 0.9) {
                        current_video.volume = 1;
                    } else {
                        current_video.volume += 0.1;
                    }
                    break;

                // 音量减少 10%
                case "s":
                    if (current_video.volume < 0.1) {
                        current_video.volume = 0;
                    } else {
                        current_video.volume -= 0.1;
                    }
                    break;

                // 后退 5s
                case "a":
                    if (current_video.currentTime < time_step) {
                        current_video.currentTime = 0;
                    } else {
                        let step = time_step;
                        if (event.ctrlKey) {
                            step = 1;
                        }
                        current_video.currentTime -= step;
                    }
                    break;

                // 前进 5s
                case "d":
                    if (
                        current_video.currentTime >
                        current_video.duration - time_step
                    ) {
                        current_video.currentTime = current_video.duration;
                    } else {
                        let step = time_step;
                        if (event.ctrlKey) {
                            step = 1;
                        }
                        current_video.currentTime += step;
                    }
                    break;
                default:
                    is_worked = false;
                    break;
            }

            // 避免和网站的快捷键功能冲突啦
            if (is_worked) {
                event.preventDefault();
                event.stopImmediatePropagation();
            }
        }

        /** keyup 事件处理函数
         * @param {KeyboardEvent} event
         */
        function keyup_handler(event) {
            if (!current_video) {
                return;
            }
            if (
                !enable ||
                event.ctrlKey ||
                event.altKey ||
                event.shiftKey ||
                event.metaKey
            ) {
                return;
            }
            if (should_skip_elem()) {
                return;
            }

            let is_worked = true;
            switch (event.key.toLocaleLowerCase()) {
                // 暂停与播放
                case "q":
                    current_video.paused
                        ? current_video.play()
                        : current_video.pause();
                    break;

                // 切换全屏
                case "e":
                    if (document.fullscreenElement) {
                        document.exitFullscreen();
                        current_video.controls = false;
                    } else {
                        current_video.requestFullscreen({
                            navigationUI: "show",
                        });
                        current_video.controls = true;
                    }
                    break;

                // 倍速增加 0.25，但不能超过 3 倍速
                case "c":
                    if (current_video.playbackRate > 2.75) {
                        current_video.playbackRate = 3;
                    } else {
                        current_video.playbackRate += 0.25;
                    }
                    break;

                // 倍速减少 0.25，但不能低于 0.5
                case "x":
                    if (current_video.playbackRate < 0.75) {
                        current_video.playbackRate = 0.5;
                    } else {
                        current_video.playbackRate -= 0.25;
                    }
                    break;

                case "z":
                    current_video.playbackRate = 2;
                    break;

                // 触发视频悬窗或退出
                // 接口文档: https://developer.mozilla.org/zh-CN/docs/Web/API/Picture-in-Picture_API
                case "r":
                    current_video.disablePictureInPicture = false;
                    // 以下代码取自官方文档
                    if (document.pictureInPictureElement) {
                        document.exitPictureInPicture();
                    } else if (document.pictureInPictureEnabled) {
                        current_video.requestPictureInPicture();
                    }
                    break;
                case "m":
                    // 不是静音，而是将音量降为 0。因为静音状态下无法调整声音大小了。
                    // 因为一些网站用了第三方播放器，其中出现了快捷键冲突。按 m 的时候会静音
                    // 所以我直接将它取值为 false！
                    current_video.muted = false;
                    if (current_video.volume !== 0) {
                        current_video.volume = 0;
                    } else {
                        current_video.volume = 0.5;
                    }
                    break;
                default:
                    is_worked = false;
                    break;
            }

            // 避免和网站的快捷键功能冲突啦
            if (is_worked) {
                event.preventDefault();
                event.stopImmediatePropagation();
            }
        }

        // 第三个参数为 true 表示在事件传递阶段（而不是冒泡阶段）触发哟
        // 这是为了尽可能避免与网站定义的快捷键冲突，干脆禁止网站的快捷键了
        document.addEventListener("keyup", keyup_handler, true);
        document.addEventListener("keydown", keydown_handler, true);
    }

    /** 为 elem 元素实现鼠标绘制区域的放大功能 */
    function zoomer(elem) {
        /** 记录鼠标框选区域的元素 */
        let selectedRect = null;

        /** 初始化显示的矩形框 */
        function init_highlight_rect() {
            if (selectedRect) {
                return;
            }
            selectedRect = document.createElement("div");
            selectedRect.id = "highlight-rect";
            selectedRect.style.padding = "0";
            selectedRect.style.margin = "0";
            selectedRect.style.position = "absolute";
            selectedRect.style.border = "2px solid red";
            selectedRect.style.backgroundColor = "transparent";
            selectedRect.style.pointerEvents = "none";
            selectedRect.style.zIndex = 99999999;
            selectedRect.style.display = "none";
            document.body.appendChild(selectedRect);
        }

        /** 重置显示的矩形框 */
        function reset_highlight_rect() {
            if (!selectedRect) {
                return;
            }
            selectedRect.style.display = "none";
            selectedRect.style.width = "0";
            selectedRect.style.height = "0";
        }

        /** 获取元素 elem 的中心位置，基于 Page 页面坐标 */
        function get_elem_center(elem) {
            // 先获取元素基于视口的位置
            const rect = elem.getBoundingClientRect();
            // 然后计算元素的中心位置，基于视口的
            const centerX = rect.left + rect.width / 2;
            const centerY = rect.top + rect.height / 2;
            // 然后加上滚动条的偏移量，得到基于页面的坐标
            const pageX = centerX + document.documentElement.scrollLeft;
            const pageY = centerY + document.documentElement.scrollTop;
            return { x: pageX, y: pageY };
        }

        /** 放大元素的某个区域，这里的 startX、startY 是基于 Page 页面的 */
        function zoom_in(elem, startX, startY, width, height) {
            // 计算缩放比列
            const scaleX = elem.offsetWidth / width;
            const scaleY = elem.offsetHeight / height;
            const scale = Math.min(
                Math.min(scaleX, scaleY).toFixed(2),
                MaxScale,
            );

            // 获取要放大区域的中心位置，基于 Page 页面的
            const left = startX + width / 2;
            const top = startY + height / 2;

            // 计算 elem 元素的中心，也是基于 Page 页面的
            const { x: elemLeft, y: elemTop } = get_elem_center(elem);

            // 计算把该区域中心平移到 elem 元素中心的距离
            const translateX = elemLeft - left;
            const translateY = elemTop - top;
            // 设置元素的 transform-origin 属性，使其缩放的中心点在矩形框的中心位置
            elem.style.transformOrigin = `${left - elem.offsetLeft}px, ${top - elem.offsetTop}px`;
            // 然后，设置元素的 transform 属性，先缩放，再平移
            elem.style.transform = `scale(${scale}) translate(${translateX}px, ${translateY}px)`;
        }

        /** 重置元素的缩放 */
        function zoom_reset(elem) {
            elem.style.transformOrigin = "";
            elem.style.transform = "";
        }

        /** 为元素 elem 绑定鼠标事。
         * - 当鼠标中键被按下、移动时可以显示矩形框
         * - 当鼠标中键松开时，可以放大或缩小元素
         */
        function bind_events(elem) {
            let isMouseDown = false;

            // 记录鼠标按下时的坐标，基于页面
            let startX = 0;
            let startY = 0;
            // 记录鼠标抬起时的坐标，基于页面
            let endX = 0;
            let endY = 0;
            /** 记录是否已经缩放过 */
            let isZoomed = false;

            /** 为 "" 表示还没有进行任何鼠标操作。
             *
             * 为 "zomin" 表示鼠标往右下方向移动，需要放大图片大小
             *
             * 为 "reset" 表示鼠标往左上方向移动，需要还原图片大小
             */
            let operation = "";

            init_highlight_rect();

            // 鼠标中键快速双击也能取消缩放
            let lastClickTime = Date.now();
            elem.addEventListener("mousedown", (e) => {
                // 只有鼠标中键点击才行
                if (e.button !== 1) {
                    return;
                }
                // 防止默认行为 —— 出现向下的滚动
                e.preventDefault();

                // 判断是否为双击
                const now = Date.now();
                if (now - lastClickTime < 300) {
                    zoom_reset(elem);
                    isZoomed = false;
                    isMouseDown = false;
                    operation = "";
                    reset_highlight_rect();
                    return;
                }
                lastClickTime = now;

                isMouseDown = true;
                // 基于页面的坐标
                startX = e.pageX;
                startY = e.pageY;

                // 设置矩形框的起点，注意了，这个起点坐标是基于 Page 的
                selectedRect.style.left = `${startX}px`;
                selectedRect.style.top = `${startY}px`;
                selectedRect.style.display = "block";
            });

            elem.addEventListener("mousemove", (e) => {
                if (!isMouseDown) {
                    return;
                }
                // 计算鼠标移动的距离，基于 Page 坐标系
                endX = e.pageX;
                endY = e.pageY;
                const x = endX - startX;
                const y = endY - startY;

                // 鼠标是往右下方向移动，需要放大图片大小
                // 如果已经放大了，那就不能再放大了
                if (!isZoomed && x > 0 && y > 0) {
                    if (operation !== "zoomin") {
                        operation = "zoomin";
                        selectedRect.style.transformOrigin = "";
                        selectedRect.style.transform = "";
                    }
                }
                // 鼠标是往左上方向移动
                else if (x < 0 && y < 0) {
                    if (operation !== "reset") {
                        operation = "reset";
                        // rect 绕着起使位置旋转 180 度
                        selectedRect.style.transformOrigin = "left top";
                        selectedRect.style.transform = "rotate(180deg)";
                    }
                }
                // 其它情况不进行处理
                else {
                }

                // 然后绘制矩形框的宽高
                if (operation !== "") {
                    selectedRect.style.width = `${Math.abs(x)}px`;
                    selectedRect.style.height = `${Math.abs(y)}px`;
                }
            });

            // 鼠标松开时，隐藏矩形框，然后放大或缩小元素
            elem.addEventListener("mouseup", () => {
                if (!isMouseDown) {
                    return;
                }
                isMouseDown = false;
                reset_highlight_rect();

                if (operation === "zoomin") {
                    zoom_in(
                        elem,
                        startX,
                        startY,
                        Math.abs(endX - startX),
                        Math.abs(endY - startY),
                    );
                    isZoomed = true;
                } else if (operation === "reset") {
                    zoom_reset(elem);
                    isZoomed = false;
                }

                operation = "";
            });

            // 鼠标移出元素时，隐藏矩形框，然后什么都不做
            elem.addEventListener("mouseout", () => {
                isMouseDown = false;
                operation = "";
                reset_highlight_rect();
            });

            // 添加属性，表示该视频已经添加了事件
            elem.setAttribute("data-zoomed", "true");
        }

        /** main code */
        !elem.getAttribute("data-zoomed") && bind_events(elem);
    }

    function main(cur_query_count = 1) {
        if (cur_query_count > MAX_QUERY_COUNT) {
            return;
        }

        videos = Array.from(document.querySelectorAll("video"));
        if (videos.length > 0) {
            log(`%c在 ${location.host} 中找到 video 控件！`, "color: yellow");
            init_document_event();
            init_html_elem();
        } else {
            setTimeout(main, 1000, cur_query_count + 1);
        }
    }

    document.addEventListener("DOMContentLoaded", () => main());
})();
