// Minimal Markdown renderer for the project's own docs.
// Safety model: every piece of source text is HTML-escaped first, and only a fixed set of
// tags is generated afterwards, so raw HTML in a document is always shown as text.
// Supported: headings, paragraphs, lists (nested by indent), fenced code, inline code,
// bold, italic, links, tables, blockquotes, horizontal rules.
(function (root, factory) {
  if (typeof module === "object" && module.exports) module.exports = factory();
  else root.Markdown = factory();
})(typeof self !== "undefined" ? self : this, function () {
  "use strict";

  var ALLOWED_PROTOCOLS = ["http:", "https:", "mailto:"];

  function escapeHtml(text) {
    return String(text)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }

  // Returns a safe absolute URL, or null. Control characters and whitespace are removed
  // first because browsers ignore them inside a scheme ("java\tscript:" is javascript:).
  function safeUrl(raw, base) {
    var cleaned = String(raw).replace(/[\u0000-\u0020\u007f]+/g, "");
    if (!cleaned) return null;
    if (cleaned.charAt(0) === "#") return cleaned;
    var url;
    try { url = base ? new URL(cleaned, base) : new URL(cleaned); } catch (e) { return null; }
    return ALLOWED_PROTOCOLS.indexOf(url.protocol) === -1 ? null : url.href;
  }

  function unescapeHtml(text) {
    return text.replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, '"')
      .replace(/&#39;/g, "'").replace(/&amp;/g, "&");
  }

  // `text` is raw (unescaped) source.
  function inline(text, opts) {
    var codes = [];
    var out = escapeHtml(text).replace(/`([^`]+)`/g, function (_, code) {
      codes.push("<code>" + code + "</code>");
      return "\u0000" + (codes.length - 1) + "\u0000";
    });
    out = out.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, function (whole, label, href) {
      // href is escaped text: undo the escaping to validate, re-escape for the attribute.
      var url = safeUrl(unescapeHtml(href), opts.linkBase);
      if (!url) return label;
      var external = /^https?:/.test(url) && !(opts.siteOrigin && url.indexOf(opts.siteOrigin) === 0);
      return '<a href="' + escapeHtml(url) + '"' +
        (external ? ' target="_blank" rel="noopener noreferrer"' : "") + ">" + label + "</a>";
    });
    out = out.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
      .replace(/(^|[^*])\*([^*\s][^*]*)\*/g, "$1<em>$2</em>");
    return out.replace(/\u0000(\d+)\u0000/g, function (_, n) { return codes[+n]; });
  }

  var RE = {
    fence: /^```/,
    heading: /^(#{1,6})\s+(.*?)\s*#*\s*$/,
    hr: /^\s{0,3}([-*_])(\s*\1){2,}\s*$/,
    quote: /^\s{0,3}>\s?/,
    listItem: /^(\s*)([-*+]|\d+[.)])\s+(.*)$/,
    tableSep: /^\s*\|?\s*:?-{3,}:?\s*(\|\s*:?-{3,}:?\s*)*\|?\s*$/
  };

  function splitRow(line) {
    var s = line.trim();
    if (s.charAt(0) === "|") s = s.slice(1);
    if (s.charAt(s.length - 1) === "|") s = s.slice(0, -1);
    return s.split("|").map(function (c) { return c.trim(); });
  }

  // items: [{indent, ordered, text}] -> nested lists by indentation.
  function renderList(items, opts) {
    var html = "", stack = [];
    items.forEach(function (item) {
      while (stack.length && item.indent < stack[stack.length - 1].indent) {
        html += "</li></" + stack.pop().tag + ">";
      }
      var top = stack[stack.length - 1];
      if (!top || item.indent > top.indent) {
        var tag = item.ordered ? "ol" : "ul";
        stack.push({ indent: item.indent, tag: tag });
        html += "<" + tag + "><li>";
      } else {
        html += "</li><li>";
      }
      html += inline(item.text, opts);
    });
    while (stack.length) html += "</li></" + stack.pop().tag + ">";
    return html;
  }

  function render(source, options) {
    var opts = options || {};
    var offset = opts.headingOffset || 0;
    var lines = String(source).replace(/\u0000/g, "").replace(/\r\n?/g, "\n").split("\n");
    var html = [], i = 0, droppedTitle = false, para = [];

    function flushPara() {
      if (para.length) { html.push("<p>" + inline(para.join(" "), opts) + "</p>"); para = []; }
    }

    while (i < lines.length) {
      var line = lines[i], m;

      if (!line.trim()) { flushPara(); i++; continue; }

      if (RE.fence.test(line.trim())) {
        flushPara();
        var code = []; i++;
        while (i < lines.length && !RE.fence.test(lines[i].trim())) code.push(lines[i++]);
        i++; // closing fence
        html.push("<pre><code>" + escapeHtml(code.join("\n")) + "</code></pre>");
        continue;
      }

      if ((m = RE.heading.exec(line))) {
        flushPara(); i++;
        if (opts.dropFirstH1 && !droppedTitle && m[1].length === 1) { droppedTitle = true; continue; }
        var level = Math.min(6, m[1].length + offset);
        html.push("<h" + level + ">" + inline(m[2], opts) + "</h" + level + ">");
        continue;
      }

      if (RE.hr.test(line)) { flushPara(); html.push("<hr>"); i++; continue; }

      if (RE.quote.test(line)) {
        flushPara();
        var quoted = [];
        while (i < lines.length && RE.quote.test(lines[i])) quoted.push(lines[i++].replace(RE.quote, ""));
        html.push("<blockquote>" + render(quoted.join("\n"), { linkBase: opts.linkBase, siteOrigin: opts.siteOrigin }) + "</blockquote>");
        continue;
      }

      if (line.indexOf("|") !== -1 && i + 1 < lines.length && RE.tableSep.test(lines[i + 1])) {
        flushPara();
        var head = splitRow(line); i += 2;
        var rows = [];
        while (i < lines.length && lines[i].indexOf("|") !== -1 && lines[i].trim()) rows.push(splitRow(lines[i++]));
        html.push('<div class="table-wrap"><table><thead><tr>' +
          head.map(function (c) { return "<th>" + inline(c, opts) + "</th>"; }).join("") +
          "</tr></thead><tbody>" +
          rows.map(function (r) {
            return "<tr>" + head.map(function (_, k) { return "<td>" + inline(r[k] || "", opts) + "</td>"; }).join("") + "</tr>";
          }).join("") + "</tbody></table></div>");
        continue;
      }

      if (RE.listItem.test(line)) {
        flushPara();
        var items = [];
        while (i < lines.length && (m = RE.listItem.exec(lines[i]))) {
          items.push({ indent: m[1].replace(/\t/g, "    ").length, ordered: /\d/.test(m[2]), text: m[3] });
          i++;
          // indented continuation lines belong to the previous item
          while (i < lines.length && /^\s{2,}\S/.test(lines[i]) && !RE.listItem.test(lines[i])) {
            items[items.length - 1].text += " " + lines[i++].trim();
          }
        }
        html.push(renderList(items, opts));
        continue;
      }

      para.push(line.trim()); i++;
    }
    flushPara();
    return html.join("\n");
  }

  return { render: render, escapeHtml: escapeHtml, safeUrl: safeUrl };
});