// ==UserScript==
// @name         Steam 折扣与价格过滤
// @namespace    http://tampermonkey.net/
// @version      2026-04-17
// @description  在 Steam 所有商品选择页面中，根据最大折扣与价格筛选游戏
// @author       You
// @match        https://store.steampowered.com/search/*
// @icon         https://www.google.com/s2/favicons?sz=64&domain=steampowered.com
// @grant        none
// @run-at        document-start
// ==/UserScript==

(function () {
  "use strict";

  const DEBUG = false;
  const MIN_DISCOUNT = 70;
  /** 只显示比它小的价格 */
  const MAX_PRICE = 10.6;

  /** 根据折扣是否过滤该 node
   *
   * @param {HTMLElement} node
   */
  function is_fileter_one_node_by_discount(node) {
    const discount = node.querySelector(".discount_pct")?.textContent;
    if (!discount) {
      return false;
    }

    // 折扣的构成为 -75% 这样的形式，只要取中间的数字即可
    const discount_num = parseInt(discount.slice(1, -1));
    const ok = discount_num < MIN_DISCOUNT;
    if (ok && DEBUG) {
      const ganme_name = node.querySelector(".title")?.textContent;
      console.log("discount:", discount, ". delete game:", ganme_name);
    }
    return ok;
  }

  /** 根据价格是否过滤该 node
   *
   * @param {HTMLElement} node
   */
  function is_fileter_one_node_by_price(node) {
    const price = node.querySelector(".discount_final_price")?.textContent;
    if (!price) {
      return false;
    }

    // 价格的构成为 $10.99 这样的形式，只要取中间的数字即可
    const price_num = parseFloat(price.slice(1));
    const ok = price_num > MAX_PRICE;
    if (ok && DEBUG) {
      const ganme_name = node.querySelector(".title")?.textContent;
      console.log("price:", price, ". delete game:", ganme_name);
    }
    return ok;
  }

  /** 添加一个元素，告知已经启用了该功能 */
  function add_tip() {
    const header = document.querySelector(".pageheader");
    if (header) {
      header.textContent += ` - 只显示折扣大于 ${MIN_DISCOUNT}%、价格低于 ${MAX_PRICE} 的游戏（由脚本提供）`;
    }
  }

  function observe_search_result() {
    const search_result_container = document.querySelector("#search_results");
    if (!search_result_container) {
      setTimeout(observe_search_result, 300);
      return;
    }

    const observer = new MutationObserver(mutations => {
      mutations.forEach(mutation => {
        const nodes = mutation.addedNodes;
        for (const node of nodes) {
          // 这是触发了右侧的筛选逻辑
          if (node.id === "search_result_container") {
            const games = node.querySelectorAll("#search_resultsRows > a");
            for (const game of games) {
              if (
                is_fileter_one_node_by_discount(game) ||
                is_fileter_one_node_by_price(game)
              ) {
                game.remove();
              }
            }
          }

          // 这是触发了底部加载
          if (
            node.nodeName === "A" &&
            (is_fileter_one_node_by_discount(node) ||
              is_fileter_one_node_by_price(node))
          ) {
            node.remove();
          }
        }
      });
    });

    observer.observe(search_result_container, {
      childList: true,
      subtree: true,
    });
  }

  // 第一次加载页面时，也进行一次过滤
  function filter_on_loaded() {
    const games = document.querySelectorAll("#search_resultsRows > a");
    for (const game of games) {
      if (
        is_fileter_one_node_by_discount(game) ||
        is_fileter_one_node_by_price(game)
      ) {
        game.remove();
      }
    }
  }

  document.addEventListener("DOMContentLoaded", () => {
    observe_search_result();
    filter_on_loaded();
    add_tip();
  });
})();
