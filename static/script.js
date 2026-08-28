document.addEventListener("DOMContentLoaded", () => {
  const STORAGE_KEY = "sidebar-collapsed";

  const toggle = document.getElementById("sidebar-toggle");
  const sidebar = document.getElementById("sidebar");

  if (toggle && sidebar) {
    const saved = localStorage.getItem(STORAGE_KEY);
    const collapsed =
      saved !== null
        ? saved === "true"
        : window.matchMedia("(max-width: 767px)").matches; // mobile: start collapsed
    if (collapsed) {
      document.body.classList.add("sidebar-collapsed");
    }

    toggle.addEventListener("click", () => {
      document.body.classList.toggle("sidebar-collapsed");
      localStorage.setItem(STORAGE_KEY, document.body.classList.contains("sidebar-collapsed"));
    });
  }

  // Tool search: filter the sidebar list (and home grid tiles) as you type.
  const searchEl = document.getElementById("tool-search");
  if (searchEl) {
    const items = Array.from(document.querySelectorAll(".sidebar .nav-item"));
    const tiles = Array.from(document.querySelectorAll(".home-grid .tool-tile"));
    const noMatch = document.getElementById("sidebar-no-match");
    const noMatchQuery = document.getElementById("sidebar-no-match-query");

    const filter = (q) => {
      q = q.trim().toLowerCase();
      let visible = 0;
      for (const el of items) {
        const match = !q || el.textContent.toLowerCase().includes(q);
        el.hidden = !match;
        if (match) visible++;
      }
      if (noMatch) {
        noMatch.hidden = visible !== 0 || q === "";
        if (noMatchQuery) noMatchQuery.textContent = `“${q}”`;
      }
      for (const tile of tiles) {
        const name = (tile.dataset.name || "").toLowerCase();
        const slug = (tile.dataset.slug || "").toLowerCase();
        tile.hidden = q && !name.includes(q) && !slug.includes(q);
      }
    };

    searchEl.addEventListener("input", () => filter(searchEl.value));
    // Keyboard: press Enter on the first match (or a partial slug match) to jump to it.
    searchEl.addEventListener("keydown", (e) => {
      if (e.key !== "Enter") return;
      const q = searchEl.value.trim().toLowerCase();
      if (!q) return;
      const first = items.find((el) => !el.hidden && el.textContent.toLowerCase().includes(q));
      const exactSlug = items.find((el) => {
        const href = el.querySelector("a")?.getAttribute("href") || "";
        return href === `/tool/${q}`;
      });
      const target = exactSlug || first;
      if (target) {
        location.assign(target.querySelector("a").getAttribute("href"));
      }
    });
    // "/" focuses the search when no input has focus and there's sidebar search.
    document.addEventListener("keydown", (e) => {
      if (e.key !== "/" || e.ctrlKey || e.metaKey || e.altKey) return;
      const tag = document.activeElement?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;
      e.preventDefault();
      searchEl.focus();
      searchEl.select();
    });
  }

  // Highlight the sidebar link for the current page on load.
  document.querySelectorAll(".sidebar .nav-link").forEach((link) => {
    if (link.getAttribute("href") === location.pathname) {
      link.classList.add("active");
    }
    link.addEventListener("click", () => {
      document.querySelectorAll(".sidebar .nav-link.active").forEach((active) => {
        active.classList.remove("active");
      });
      link.classList.add("active");
    });
  });

  const runBtn = document.getElementById("run-btn");
  if (runBtn) {
    const card = runBtn.closest("[data-slug]");
    const slug = card ? card.dataset.slug : null;
    const inputEl = document.getElementById("tool-input");
    const resultEl = document.getElementById("result");
    const findEl = document.getElementById("tool-find");
    const replaceEl = document.getElementById("tool-replace");
    const actionEl = document.getElementById("tool-action");
    const copyBtn = document.getElementById("copy-btn");
    const clearBtn = document.getElementById("clear-btn");

    const setOutput = (text, isError) => {
      if (resultEl) {
        resultEl.textContent = text;
        resultEl.classList.toggle("tool-result-error", Boolean(isError));
      }
    };

    const run = async () => {
      if (!slug) return;
      const params = new URLSearchParams();
      params.set("input", inputEl ? inputEl.value : "");
      if (findEl) params.set("find", findEl.value);
      if (replaceEl) params.set("replace", replaceEl.value);
      if (actionEl) params.set("action", actionEl.value);
      try {
        const res = await fetch(`/api/${slug}?${params.toString()}`);
        setOutput(await res.text(), !res.ok);
      } catch (err) {
        setOutput(`Error: ${err}`, true);
      }
    };

    runBtn.addEventListener("click", (e) => {
      e.preventDefault();
      run();
    });

    // Live re-run while typing (debounced); also runs on find/replace/action.
    let timer = null;
    const scheduleRun = () => {
      clearTimeout(timer);
      timer = setTimeout(run, 250);
    };
    if (inputEl) {
      inputEl.addEventListener("input", scheduleRun);
      inputEl.addEventListener("keydown", (e) => {
        if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
          clearTimeout(timer);
          run();
        }
      });
    }
    for (const el of [findEl, replaceEl]) {
      if (el) el.addEventListener("input", scheduleRun);
    }
    if (actionEl) actionEl.addEventListener("change", run);

    if (copyBtn) {
      copyBtn.addEventListener("click", async () => {
        const text = resultEl ? resultEl.textContent : "";
        if (!text) return;
        try {
          await navigator.clipboard.writeText(text);
        } catch {
          // Fallback for contexts where the async clipboard API is blocked.
          const ta = document.createElement("textarea");
          ta.value = text;
          document.body.appendChild(ta);
          ta.select();
          document.execCommand("copy");
          ta.remove();
        }
        const original = copyBtn.textContent;
        copyBtn.textContent = "Copied!";
        setTimeout(() => {
          copyBtn.textContent = original;
        }, 1200);
      });
    }

    if (clearBtn) {
      clearBtn.addEventListener("click", () => {
        if (inputEl) inputEl.value = "";
        if (findEl) findEl.value = "";
        if (replaceEl) replaceEl.value = "";
        if (actionEl) actionEl.selectedIndex = 0;
        setOutput("", false);
        if (inputEl) inputEl.focus();
      });
    }
  }
});
