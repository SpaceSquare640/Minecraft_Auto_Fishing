// Tiny i18n: elements carry data-i18n="section.key"; code calls t("section.key", { name }).
import { en, type Strings } from "./locales/en";

const locales: Record<string, Strings> = { en };
let current: Strings = en;

export function setLanguage(lang: string): void {
  current = locales[lang] ?? locales[lang.split("-")[0]] ?? en;
  document.documentElement.lang = lang in locales ? lang : "en";
  document.querySelectorAll<HTMLElement>("[data-i18n]").forEach((el) => {
    el.textContent = t(el.dataset.i18n ?? "");
  });
}

export function t(key: string, params: Record<string, string | number> = {}): string {
  const lookup = (table: unknown): unknown =>
    key.split(".").reduce<unknown>((node, part) => (node as Record<string, unknown> | undefined)?.[part], table);
  const value = lookup(current) ?? lookup(en);
  const text = typeof value === "string" ? value : key;
  return text.replace(/\{(\w+)\}/g, (_, name: string) => String(params[name] ?? `{${name}}`));
}
