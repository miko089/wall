(() => {
    "use strict";

    const API = Object.freeze({
        FETCH: "/get_msgs",
        POST: "/send_msg",
        LAST: "/last_msg",
        GIT_INFO: "/git_info"
    });
    const CHAR_LIMIT    = 250;
    const PAGE_SIZE     = 20;
    const POLL_INTERVAL = 5_000;

    const qs   = obj => Object.entries(obj)
        .filter(([,v]) => v !== null && v !== undefined)
        .map(([k,v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)
        .join("&");

    const fmtDate = iso => {
        const userTimeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
        return new Date(iso * 1000).toLocaleString("ru-RU", {
            timeZone: userTimeZone,
            year: "numeric", month: "2-digit", day: "2-digit",
            hour: "2-digit", minute: "2-digit", second: "2-digit",
        });
    };

    const escapeHtml = str => str
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");

    class ChatAPI {
        static async fetchBatch({ after = null, before = null, limit = PAGE_SIZE } = {}) {
            const url = `${API.FETCH}?${qs({ after, before, limit })}`;
            const res = await fetch(url);
            if (!res.ok) throw new Error("Не могу загрузить сообщения");
            return res.json();
        }
        static async postMessage(body) {
            const res = await fetch(API.POST, {
                method : "POST",
                headers: { "Content-Type": "application/json" },
                body   : JSON.stringify(body),
            });
            if (!res.ok) {
                const { error } = await res.json();
                throw new Error(error || "Ошибка отправки");
            }
            return res.json();
        }
        static async lastId() {
            const res = await fetch(API.LAST);
            if (!res.ok) throw new Error("Не могу проверить новые сообщения");
            const { id } = await res.json();
            return id;
        }
    }

    class GitInfo {
        static async fetch() {
            const res = await fetch(API.GIT_INFO);
            if (!res.ok) throw new Error("Не могу полуичть информацию о репо");
            return res.json();
        }

        static async init() {
            const { commit_hash, repo_url } = await GitInfo.fetch();
            document.getElementById("commit-hash").textContent = `Commit: ${commit_hash.slice(0, 7)}`;
            const repoLink = document.getElementById("repo-link");
            repoLink.href = repo_url;
        }
    }

    /* ========= UI ========= */    class Toast {
        #container;
        #onClick;
        #newMessageToast = null;
        
        constructor(container, onClick) {
            this.#container = container;
            this.#onClick = onClick;
        }
        
        show(msg, isErr = false) {
            // Для сообщений о новых сообщениях используем специальный тост
            if (msg.includes("новых сообщений") || msg.includes("Новое сообщение")) {
                if (this.#newMessageToast) {
                    this.#newMessageToast.textContent = msg;
                    return;
                }

                const toast = document.createElement('div');
                toast.className = 'toast';
                toast.textContent = msg;
                
                toast.addEventListener('click', () => {
                    this.hide(toast);
                    this.#newMessageToast = null;
                    this.#onClick?.();
                });
                
                this.#container.appendChild(toast);
                this.#newMessageToast = toast;
                return;
            }

            // Для ошибок и других сообщений
            const toast = document.createElement('div');
            toast.className = 'toast' + (isErr ? ' error' : '');
            toast.textContent = msg;
            
            toast.addEventListener('click', () => {
                this.hide(toast);
            });
            
            this.#container.appendChild(toast);
            
            if (isErr) {
                setTimeout(() => this.hide(toast), 5000);
            }
        }
        
        hide(toast) {
            toast.classList.add('hiding');
            toast.addEventListener('animationend', () => {
                toast.remove();
                if (this.#newMessageToast === toast) {
                    this.#newMessageToast = null;
                }
            }, { once: true });
        }
    }    class Renderer {
        #list;
        
        constructor(listEl) { 
            this.#list = listEl;
            const messages = Array.from(this.#list.children);
            messages.forEach(msg => {
                if (!msg.parentElement.classList.contains('message-wrapper')) {
                    const wrapper = document.createElement('div');
                    wrapper.className = 'message-wrapper';
                    msg.parentNode.insertBefore(wrapper, msg);
                    wrapper.appendChild(msg);
                }
            });
        }
        
        prepend(msg) {
            const el = this.#tpl(msg);
            const wrapper = document.createElement('div');
            wrapper.className = 'message-wrapper';
            wrapper.appendChild(el);
            this.#list.prepend(wrapper);
        }
        
        append(msg) {
            const el = this.#tpl(msg);
            const wrapper = document.createElement('div');
            wrapper.className = 'message-wrapper';
            wrapper.appendChild(el);
            this.#list.append(wrapper);
        }

        #tpl({ id, author, content, timestamp }) {
            const div = document.createElement("div");
            div.className = "message";
            div.innerHTML = `
        <div class="head">${escapeHtml(author)}</div>
        <div class="body">${escapeHtml(content)}</div>
        <time datetime="${timestamp}">${fmtDate(timestamp)}</time>
      `;
            return div;
        }
    }

    /* ========= App ========= */
    class ChatApp {
        #state     = { newest: null, oldest: null };
        #renderer; #toast;
        #author; #content; #counter;
        #observer;

        constructor() {
            const $ = id => document.getElementById(id);
            this.#renderer = new Renderer($("messages"));
            this.#toast = new Toast(document.querySelector(".toast-container"), () => this.#fetchNewest());
            this.#author = $("author");
            this.#content = $("content");
            this.#counter = $("counter");

            // auto-resize for textarea
            const autoResize = () => {
                this.#content.style.height = 'auto';
                this.#content.style.height = this.#content.scrollHeight + 'px';
            };
            
            this.#content.addEventListener("input", () => {
                this.#updateCounter();
                autoResize();
            });
            this.#updateCounter();
            autoResize();
            GitInfo.init();
            $("msg-form").addEventListener("submit", e => this.#onSubmit(e));
            this.#observer = new IntersectionObserver(e => this.#onIntersect(e[0]));
            this.#observer.observe($("sentinel"));
            this.#boot().catch(console.error);
        }

        async #boot() {
            await this.#fetchNewest();
            setInterval(() => this.#pollLast(), POLL_INTERVAL);
        }

        /* ----- Handlers ----- */
        async #onSubmit(e) {
            e.preventDefault();
            const author  = this.#author.value.trim();
            const content = this.#content.value.trim();
            if (!author || !content) return;

            try {
                await ChatAPI.postMessage({ author, content });
                this.#content.value = "";
                this.#content.style.height = "auto";
                this.#updateCounter();
                await this.#fetchNewest();
            } catch (err) { this.#toast.show(err.message, true); }
        }

        async #onIntersect(entry) {
            if (!entry.isIntersecting || this.#state.oldest === null) return;
            try {
                const msgs = await ChatAPI.fetchBatch({ before: this.#state.oldest });
                if (!msgs.length) return;

                msgs.sort((a, b) => b.id - a.id)        // DESC
                    .forEach(m => {
                        this.#renderer.append(m);
                        this.#state.oldest = m.id;
                    });
            } catch (err) { this.#toast.show(err.message, true); }
        }

        /* ----- Misc ----- */
        #updateCounter() {
            const left = CHAR_LIMIT - this.#content.value.length;
            this.#counter.textContent = `Осталось ${left}`;
        }

        async #fetchNewest() {
            try {
                const msgs = await ChatAPI.fetchBatch({ after: this.#state.newest ?? 0 });
                msgs.sort((a, b) => a.id - b.id)        // ASC
                    .forEach(m => this.#renderer.prepend(m));

                if (msgs.length) {
                    this.#state.newest = Math.max(this.#state.newest ?? 0, msgs[msgs.length - 1].id);
                    this.#state.oldest ??= msgs[0].id;
                }
            } catch (err) { this.#toast.show(err.message, true); }
        }        async #pollLast() {
            try {
                const lastId = await ChatAPI.lastId();
                if (lastId > (this.#state.newest ?? 0)) {
                    const diff = lastId - (this.#state.newest ?? 0);
                    const message = diff === 1 
                        ? "Новое сообщение! Кликни, чтобы обновить."
                        : `${diff} новых сообщений! Кликни, чтобы обновить.`;
                    this.#toast.show(message);
                }
            } catch {/* silent */ }
        }
    }

    /* ========= GO ========= */
    window.addEventListener("DOMContentLoaded", () => new ChatApp());
})();