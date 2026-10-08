// Fills elements marked with data-i18n (text) and data-i18n-attr ("attr:key;attr:key")
// from window.LOCALES[lang]. Missing keys fall back to English, then to the key itself,
// so a gap in a translation shows up visibly instead of as blank space.
(function () {
  "use strict";

  var FALLBACK = "en";

  function lookup(dict, key) {
    return key.split(".").reduce(function (node, part) {
      return node && typeof node === "object" ? node[part] : undefined;
    }, dict);
  }

  function translate(lang, key) {
    var locales = window.LOCALES || {};
    var value = lookup(locales[lang], key);
    if (value === undefined && lang !== FALLBACK) value = lookup(locales[FALLBACK], key);
    if (value === undefined) {
      console.warn("[i18n] missing key:", key);
      return key;
    }
    return value;
  }

  function apply(lang, root) {
    var scope = root || document;
    var locales = window.LOCALES || {};
    var active = locales[lang] ? lang : FALLBACK;

    scope.querySelectorAll("[data-i18n]").forEach(function (el) {
      el.textContent = translate(active, el.getAttribute("data-i18n"));
    });

    scope.querySelectorAll("[data-i18n-attr]").forEach(function (el) {
      el.getAttribute("data-i18n-attr").split(";").forEach(function (pair) {
        var parts = pair.split(":");
        if (parts.length === 2) el.setAttribute(parts[0].trim(), translate(active, parts[1].trim()));
      });
    });

    document.documentElement.lang = active;
    return active;
  }

  window.I18n = { apply: apply, t: translate };
})();
