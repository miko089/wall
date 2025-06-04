(() => {
    "use strict";

    const API     = Object.freeze({ FETCH: "/get_msgs", POST: "/send_msg", LAST: "/last_msg" });
    const CHAR_LIMIT    = 250;
    const PAGE_SIZE     = 20;
    const POLL_INTERVAL = 5_000;

    const qs   = obj => Object.entries(obj)
        .filter(([,v]) => v !== null && v !== undefined)
        .map(([k,v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)
        .join("&");

    const fmtDate = iso =>
        new Date(iso*1000).toLocaleString("ru-RU", {
            year: "numeric", month: "2-digit", day: "2-digit",
            hour: "2-digit", minute: "2-digit", second: "2-digit",
        });

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

    /* ========= UI ========= */
    class Toast {
        #el;
        #onClick;
        constructor(el, onClick) {
            this.#el = el;
            this.#onClick = onClick;
            this.#el.addEventListener("click", () => {
                this.hide();
                this.#onClick?.();
            });
            this.#el.addEventListener("click", () => this.hide()); }
        show(msg, isErr = false) { this.#el.textContent = msg; this.#el.classList.toggle("error", isErr); this.#el.hidden = false; }
        hide() { this.#el.hidden = true; }
    }

    class Renderer {
        #list;
        constructor(listEl) { this.#list = listEl; }
        prepend(msg) { this.#list.prepend(this.#tpl(msg)); }
        append (msg) { this.#list.append (this.#tpl(msg)); }
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
            this.#toast    = new Toast($("toast"), () => this.#fetchNewest());
            this.#author   = $("author");
            this.#content  = $("content");
            this.#counter  = $("counter");

            $("msg-form").addEventListener("submit", e => this.#onSubmit(e));
            this.#content.addEventListener("input", () => this.#updateCounter());
            this.#updateCounter();

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
        }

        async #pollLast() {
            try {
                if ((await ChatAPI.lastId()) > (this.#state.newest ?? 0))
                    this.#toast.show("Новое сообщение! Кликни, чтобы обновить.");
            } catch {/* silent */ }
        }
    }

    /* ========= GO ========= */
    window.addEventListener("DOMContentLoaded", () => new ChatApp());
})();
