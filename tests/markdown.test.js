// Run: node --test tests/
const test = require("node:test");
const assert = require("node:assert/strict");
const { render, safeUrl } = require("../js/markdown.js");

const BASE = "https://github.com/o/r/blob/main/";

test("headings, paragraphs and heading offset", () => {
  assert.equal(render("# Title\n\nHello\nworld"), "<h1>Title</h1>\n<p>Hello world</p>");
  assert.equal(render("## Sub", { headingOffset: 1 }), "<h3>Sub</h3>");
  assert.equal(render("# Title\n## Sub", { dropFirstH1: true }), "<h2>Sub</h2>");
});

test("inline formatting", () => {
  assert.equal(render("**b** *i* `c`"), "<p><strong>b</strong> <em>i</em> <code>c</code></p>");
  assert.equal(render("`**not bold**`"), "<p><code>**not bold**</code></p>");
});

test("lists, including nesting and ordered lists", () => {
  assert.equal(render("- a\n- b"), "<ul><li>a</li><li>b</li></ul>");
  assert.equal(render("1. a\n2. b"), "<ol><li>a</li><li>b</li></ol>");
  assert.equal(render("- a\n  - a1\n- b"), "<ul><li>a<ul><li>a1</li></ul></li><li>b</li></ul>");
});

test("tables, code fences, blockquotes, rules", () => {
  const table = render("| A | B |\n|---|---|\n| 1 | 2 |");
  assert.match(table, /<th>A<\/th><th>B<\/th>/);
  assert.match(table, /<td>1<\/td><td>2<\/td>/);
  assert.equal(render("```js\nconst a = '<b>';\n```"), "<pre><code>const a = &#39;&lt;b&gt;&#39;;</code></pre>");
  assert.equal(render("> quoted"), "<blockquote><p>quoted</p></blockquote>");
  assert.equal(render("---"), "<hr>");
});

test("raw HTML is always rendered as text", () => {
  const payloads = [
    "<script>alert(1)</script>",
    '<img src=x onerror="alert(1)">',
    '<a href="javascript:alert(1)">x</a>',
    "| <svg onload=alert(1)> | b |\n|---|---|\n| c | d |",
    "- <iframe src=//evil></iframe>",
    "> <script>alert(1)</script>"
  ];
  for (const p of payloads) {
    const out = render(p);
    assert.doesNotMatch(out, /<(script|img|svg|iframe)\b/i, p);
    assert.doesNotMatch(out, /<a href="javascript/i, p);
  }
});

test("links: safe schemes kept, dangerous schemes dropped", () => {
  assert.equal(render("[x](https://a.com)", { linkBase: BASE }),
    '<p><a href="https://a.com/" target="_blank" rel="noopener noreferrer">x</a></p>');
  for (const bad of ["javascript:alert(1)", "JaVaScRiPt:alert(1)", "java\tscript:alert(1)",
                     "data:text/html,<b>", "vbscript:x", "jav&#x61;script:alert(1)"]) {
    const out = render(`[x](${bad})`, { linkBase: BASE });
    assert.doesNotMatch(out, /href="(javascript|data|vbscript)/i, bad);
  }
  assert.equal(safeUrl(" javascript:alert(1)"), null);
});

test("relative links resolve against linkBase; quotes cannot break the attribute", () => {
  assert.match(render("[d](docs/a.md)", { linkBase: BASE }), /href="https:\/\/github\.com\/o\/r\/blob\/main\/docs\/a\.md"/);
  const out = render('[x](https://a.com/"onmouseover="alert(1))');
  assert.doesNotMatch(out, /"\s*onmouseover=/);
});

test("links to the site itself open in the same tab", () => {
  const out = render("[d](https://site.example/docs.html)", { siteOrigin: "https://site.example" });
  assert.doesNotMatch(out, /target=/);
});