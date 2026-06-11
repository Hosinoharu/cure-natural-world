// ==UserScript==
// @name         显示 Github Issue 最新评论的时间
// @namespace    http://tampermonkey.net/
// @version      2026-06-11
// @description  显示 Github Issue 最新评论的时间
// @author       You
// @match        https://github.com/*
// @match        https://github.com/*/*/issues
// @match        https://github.com/*/*/issues?*
// @icon         https://www.google.com/s2/favicons?sz=64&domain=github.com
// @grant        none
// @run-at       document-start
// ==/UserScript==

/*
    # 原理简述

    ## 最简单
    访问 Github Issue API 就可以。但是需要提供 Personal Access Token，否则会有速率限制。
    很显然，这不行，太麻烦了，所以另寻方法 —— 利用网站本身的请求信息了。

    ## 登录账号的情况
    `API https://github.com/_graphql` 发送请求后，返回 issue 相关数据，
    其中就包含了 `"createdAt": "xx", "updatedAt": "xx"` 时间信息。

    考虑到将来要取出它们，所以 `hook Respons.parse` —— 可不是 `JSON.parse` 哟，分析网页得出的结论。
    找到 `#issue-id <--> updatedAt` 的映射关系，然后添加到页面中。

    ## 未登录账号的情况
    似乎直接保存在网页 html 中。

    # 注入的网页
    因为 github 使用 turbo navigation，所以会注入到所有 github 页面，
    然后定时检查是否为 issue 页面。
*/

(function () {
    "use strict";

    /** issue-id <--> updatedAt
     * @type {Map<number, string>}
     */
    const issues = new Map();
    /** 定位到 issue 整体的 css selector */
    const selector = ".ListView-module__ul__uMK30 > div > div > li";
    /** 插入到网页中的 html 元素的 clas name */
    const class_name = "my-issue-updated-time";

    const DEBUG = false;
    const raw_log = console.log;
    function log(...args) {
        DEBUG && raw_log("[show-github-issue-updated-time]", ...args);
    }

    const raw_json = Response.prototype.json;

    /** 用于 hook 的 方法 */
    async function hooked_method() {
        const value = await Reflect.apply(raw_json, this, []);

        const issues_info = value?.data?.repository?.search?.edges;
        if (issues_info && Array.isArray(issues_info)) {
            for (const issue of issues_info) {
                const id = issue.node?.number;
                const updated = issue.node?.updatedAt;
                if (id && updated) {
                    // 注意时区转换
                    const local_time = new Date(updated).toLocaleString("zh-CN", {
                        hour12: false,
                    });
                    issues.set(id, local_time);
                    log(`add #issue-${id} updated: ${updated}`);
                }
            }
        }

        return value;
    }

    /** hook 特定 API 监听 issue 相关的信息 */
    function hook_api() {
        Response.prototype.json = hooked_method;
    }

    /** 给网页中的 issue 添加更新时间 */
    function add_updated_time() {
        const elements = document.querySelectorAll(selector);
        for (const element of elements) {
            const issue_id = element.ariaLabel?.match(/#(\d+)/)?.[1];
            if (!issue_id) {
                console.warn("issue_id not found");
                return;
            }

            const updated = issues.get(parseInt(issue_id));
            if (!updated) {
                log(`#issue-${issue_id} updated time not found`);
                continue;
            }

            // insert location
            const target = element.querySelector(
                '[data-testid="list-row-repo-name-and-number"]',
            );
            if (!target) {
                console.warn("insert location not found");
                return;
            }

            const insert_text = ` · updated ${updated}`;
            const exist_span = target.querySelector(`:scope > .${class_name}`);

            if (exist_span) {
                exist_span.textContent = insert_text;
            } else {
                const span = document.createElement("span");
                span.style.className = class_name;
                span.textContent = insert_text;
                target.append(span);
            }
        }
    }

    // ==================================================================

    /** 是否启动了定时器 */
    let initialized = false;

    function main() {
        if (initialized || !location.href.includes("/issues")) return;

        hook_api();

        /** 因为 github 使用 turbo navigation
         * 所以使用定时器检查记录的 issues 中是否有数据
         * - 有数据则添加，然后清空咯
         * - 没有则继续等待
         */
        const id = setInterval(() => {
            if (issues.size > 0) {
                try {
                    add_updated_time();
                } catch (e) {
                    clearInterval(id);
                    console.error("add updated time error:", e?.message);
                } finally {
                    issues.clear();
                }
            }
        }, 1000);

        initialized = true;
        log("initialized");
    }

    main();
    window.addEventListener("turbo:load", async () => main());
})();
