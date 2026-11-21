import { colorScheme, signal } from "nsh";
import { MathRender } from "bibcitex/native";
import type { MathFormula } from "../ui";

/** Typeset formulas by (TeX, point size, appearance); `null` marks a
 * formula the renderer could not typeset, whose source shows instead. */
const formulas = signal<Readonly<Record<string, MathFormula | null>>>({});
const pending = new Set<string>();

function load(key: string, tex: string, fontSize: number, dark: boolean): void {
  if (pending.has(key)) return;
  pending.add(key);
  MathRender.formula(tex, fontSize, dark)
    .then((f) => (f.dataUrl ? { dataUrl: f.dataUrl, width: f.width, height: f.height } : null))
    .catch(() => null)
    .then((value) => {
      pending.delete(key);
      formulas.update((all) => ({ ...all, [key]: value }));
    });
}

/** A `MathText` formula source at `fontSize`, following the appearance:
 * `undefined` while it is typeset, then the artwork or `null`. Reactive. */
export function formulaAt(fontSize: number): (tex: string) => MathFormula | null | undefined {
  return (tex: string) => {
    const dark = colorScheme() === "dark";
    const key = `${fontSize}|${dark ? 1 : 0}|${tex}`;
    const all = formulas();
    if (key in all) return all[key];
    load(key, tex, fontSize, dark);
    return undefined;
  };
}
