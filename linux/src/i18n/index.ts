import { effect, locale, localeSettings, signal } from "nsh";
import { availableLocales, setLocale, t } from "nsh/i18n";

/** The languages a user can pick in Settings. */
export type LanguagePreference = "system" | "zh-Hans" | "en";
export type Language = "zh-Hans" | "en";
const preference = signal<LanguagePreference>("system");
const systemLanguageTag = signal("en");

/** Keep system-language detection independent from the locale override. */
export function resolveLanguage(selection: LanguagePreference, systemLocale: string = systemLanguageTag()): Language {
  if (selection !== "system") return selection;
  const first = systemLocale.split(/[.@]/)[0].toLowerCase().replace(/_/g, "-");
  return first === "zh" || first.startsWith("zh-hans") || first === "zh-cn" || first === "zh-sg" ? "zh-Hans" : "en";
}

export const language = (): Language => resolveLanguage(preference());
export const languagePreference = (): LanguagePreference => preference();

/** Called by the app root so following the system remains reactive. */
export function installLanguage(readSystemLocale: () => string): void {
  effect(() => {
    // NSH prepends its override to localeSettings.languages, so read the raw
    // OS tag through the small native getter, never infer it from that list.
    localeSettings();
    const raw = readSystemLocale();
    const resolved = resolveLanguage(preference(), raw);
    systemLanguageTag.set(raw);
    if (locale() !== resolved) setLocale(availableLocales().includes(resolved) ? resolved : "en");
  });
}

export function setLanguagePreference(value: LanguagePreference): void {
  preference.set(value);
}

/** Mirrors locale_key.rs: fixed-width UTF-16 encoding is reversible. */
function encodedKey(key: string): string {
  let result = "k";
  for (let i = 0; i < key.length; i++) result += key.charCodeAt(i).toString(16).padStart(4, "0");
  return result;
}

const placeholder = /\{([0-9]+)\}/g;
export function format(template: string, args: readonly unknown[]): string {
  return template.replace(placeholder, (match, index: string) => Number(index) < args.length ? String(args[Number(index)]) : match);
}

export function translate(key: string, lang: Language, args: readonly unknown[] = []): string {
  const encoded = encodedKey(key);
  const template = t(encoded, { locale: lang });
  if (template === `${lang}.${encoded}`) return format(key, args);
  const options: { locale: string; [name: string]: unknown } = { locale: lang, pct: "%" };
  // Supply missing positional values before interpolation, leaving user args untouched.
  for (const match of template.matchAll(/%\{p([0-9]+)\}/g)) options[`p${match[1]}`] = `{${match[1]}}`;
  args.forEach((value, index) => { options[`p${index}`] = String(value); });
  return t(encoded, options);
}

export function tr(key: string, ...args: unknown[]): string {
  locale();
  return translate(key, language(), args);
}

export function trReferences(count: number): string {
  return count === 1 ? tr("1 条文献") : tr("{0} 条文献", count);
}

export type Message = { literal: string } | { key: string; args?: readonly Message[] | readonly unknown[] };
export function resolveMessage(message: Message): string {
  if ("literal" in message) return message.literal;
  const args = (message.args ?? []).map((arg) =>
    typeof arg === "object" && arg !== null && ("literal" in arg || "key" in arg) ? resolveMessage(arg as Message) : arg,
  );
  return tr(message.key, ...args);
}
