// Loads Markdown documents from the GitHub repo into elements marked with data-doc:
//   <div data-doc data-branch="Source_Code" data-path="docs/overview.md"
//        data-heading-offset="1" data-drop-title></div>
// Files are read from raw.githubusercontent.com, so the repo is the single source of truth.
// On localhost only, ?source=<base-url> reads <base-url>/<branch>/<path> instead, so docs can
// be previewed before they are pushed (the folder names match the branch names).
(function () {
  "use strict";

  var REPO = "SpaceSquare640/Minecraft_Auto_Fishing";
  var RAW = "https://raw.githubusercontent.com/" + REPO + "/";
  var BLOB = "https://github.com/" + REPO + "/blob/";
  var LOCAL_HOSTS = ["localhost", "127.0.0.1", "[::1]"];

  function sourceBase() {
    var override = new URLSearchParams(location.search).get("source");
    if (override && LOCAL_HOSTS.indexOf(location.hostname) !== -1) return override.replace(/\/?$/, "/");
    return RAW;
  }

  function t(key) { return window.I18n.t(document.documentElement.lang, key); }

  function showError(el, githubUrl) {
    el.innerHTML = "";
    var p = document.createElement("p");
    p.className = "doc-status doc-status--error";
    p.textContent = t("loader.error") + " ";
    var a = document.createElement("a");
    a.href = githubUrl;
    a.target = "_blank";
    a.rel = "noopener noreferrer";
    a.textContent = t("loader.viewOnGithub");
    p.appendChild(a);
    el.appendChild(p);
  }

  function load(el) {
    var branch = el.getAttribute("data-branch");
    var path = el.getAttribute("data-path");
    var encodedPath = path.split("/").map(encodeURIComponent).join("/");
    var githubUrl = BLOB + branch + "/" + encodedPath;
    var dir = githubUrl.slice(0, githubUrl.lastIndexOf("/") + 1);

    el.setAttribute("aria-busy", "true");
    el.innerHTML = '<p class="doc-status"></p>';
    el.firstChild.textContent = t("loader.loading");

    return fetch(sourceBase() + branch + "/" + encodedPath)
      .then(function (res) {
        if (!res.ok) throw new Error("HTTP " + res.status);
        return res.text();
      })
      .then(function (text) {
        // Safe: Markdown.render escapes all source text and emits only whitelisted tags.
        el.innerHTML = window.Markdown.render(text, {
          linkBase: dir,
          siteOrigin: location.origin,
          headingOffset: Number(el.getAttribute("data-heading-offset") || 0),
          dropFirstH1: el.hasAttribute("data-drop-title")
        });
      })
      .catch(function (err) {
        console.warn("[doc-loader]", path, err);
        showError(el, githubUrl);
      })
      .then(function () { el.removeAttribute("aria-busy"); });
  }

  document.querySelectorAll("[data-doc]").forEach(load);
})();