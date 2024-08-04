import { Localization } from "./localization";

import { writeTextFile } from "@tauri-apps/api/fs";
import { join } from "@tauri-apps/api/path";

import { OrderedSet } from "immutable";
import XRegExp from "xregexp";

export function romanizeString(string: string): string {
    const replacements = {
        "。": ".",
        "、": ",",
        "，": ",",
        "・": "·",
        "゠": "–",
        "＝": "—",
        ー: "—",
        "…": "...",
        "‥": "...",
        "「": "'",
        "」": "'",
        "〈": "'",
        "〉": "'",
        "『": '"',
        "』": '"',
        "《": '"',
        "》": '"',
        "（": "(",
        "〔": "(",
        "｟": "(",
        "〘": "(",
        "）": ")",
        "〕": ")",
        "｠": ")",
        "〙": ")",
        "｛": "{",
        "｝": "}",
        "［": "[",
        "【": "[",
        "〖": "[",
        "〚": "[",
        "］": "]",
        "】": "]",
        "〗": "]",
        "〛": "]",
        "〜": "~",
        "？": "?",
        "：": ":",
        "！": "!",
        "※": "·",
        "　": " ",
        Ⅰ: "I",
        ⅰ: "i",
        Ⅱ: "II",
        ⅱ: "ii",
        Ⅲ: "III",
        ⅲ: "iii",
        Ⅳ: "IV",
        ⅳ: "iv",
        Ⅴ: "V",
        ⅴ: "v",
        Ⅵ: "VI",
        ⅵ: "vi",
        Ⅶ: "VII",
        ⅶ: "vii",
        Ⅷ: "VIII",
        ⅷ: "viii",
        Ⅸ: "IX",
        ⅸ: "ix",
        Ⅹ: "X",
        ⅹ: "x",
        Ⅺ: "XI",
        ⅺ: "xi",
        Ⅻ: "XII",
        ⅻ: "xii",
        Ⅼ: "L",
        ⅼ: "l",
        Ⅽ: "C",
        ⅽ: "c",
        Ⅾ: "D",
        ⅾ: "d",
        Ⅿ: "M",
        ⅿ: "m",
    } as Record<string, string>;

    const result = [];

    for (const char of string) {
        if (char in replacements) {
            result.push(replacements[char]);
        } else {
            result.push(char);
        }
    }

    return result.join("");
}

export function extractStrings(rubyCode: string, mode = false): string[] | [string[], number[]] {
    function isEscaped(index: number) {
        let backslashCount = 0;

        for (let i = index; i >= 0; i--) {
            if (rubyCode[i] !== "\\") {
                break;
            }

            backslashCount++;
        }

        return backslashCount % 2 === 1;
    }

    const strings: string[] | OrderedSet<string> = mode ? [] : OrderedSet<string>().asMutable();
    const indices: number[] = [];
    let insideComment = false;
    let insideMultilineComment = false;
    let insideString = false;
    let stringStartIndex = 0;
    let currentQuoteType = "";

    for (let i = 0; i < rubyCode.length; i++) {
        const char = rubyCode[i];

        if (!insideString && !insideComment && rubyCode.slice(i, i + 6) === "=begin") {
            insideMultilineComment = true;
            i += 5;
            continue;
        }

        if (insideMultilineComment) {
            if (rubyCode.slice(i, i + 4) === "=end") {
                insideMultilineComment = false;
                i += 3;
            }
            continue;
        }

        if (!insideString && char === "#") {
            insideComment = true;
            continue;
        }
        if (insideComment) {
            if (char === "\n") {
                insideComment = false;
            }
            continue;
        }

        if (!insideString && ['"', "'"].includes(char)) {
            insideString = true;
            stringStartIndex = i;
            currentQuoteType = char;
        } else if (insideString && char === currentQuoteType && !isEscaped(i - 1)) {
            if (mode) {
                (strings as string[]).push(rubyCode.slice(stringStartIndex + 1, i).replaceAll(/\r?\n/g, "\\#"));
                indices.push(stringStartIndex + 1);
            } else {
                (strings as OrderedSet<string>).add(
                    rubyCode.slice(stringStartIndex + 1, i).replaceAll(/\r?\n/g, "\\#"),
                );
            }

            insideString = false;
            currentQuoteType = "";
        }
    }

    const stringsArray = mode ? null : Array.from(strings);
    return mode ? [strings as string[], indices] : (stringsArray as string[]);
}

// this function just takes forever to execute in rust
export async function readScripts(string: string, otherPath: string, romanize: boolean) {
    const strings = [];

    const extractedStrings = extractStrings(string) as string[];
    for (const extracted of extractedStrings) {
        let trimmed = extracted.replaceAll("　", " ").trim();

        if (trimmed.length === 0) {
            continue;
        }

        if (
            /(Graphics|Data|Audio|Movies|System)\/.*\/?/.test(trimmed) ||
            /r[xv]data2?$/.test(trimmed) ||
            XRegExp(
                String.raw`^[.()+\-:;\[\]^~%&!№$@\`*\/→×？?ｘ％▼|♥♪！：〜『』「」〽。…‥＝゠、，【】［］｛｝（）〔〕｟｠〘〙〈〉《》・\\#'"<>=_ー※▶ⅠⅰⅡⅱⅢⅲⅣⅳⅤⅴⅥⅵⅦⅶⅧⅷⅨⅸⅩⅹⅪⅺⅫⅻⅬⅼⅭⅽⅮⅾⅯⅿ\s0-9]+$`,
            ).test(trimmed) ||
            /@window/.test(trimmed) ||
            /\$game/.test(trimmed) ||
            /_/.test(trimmed) ||
            /^\\e/.test(trimmed) ||
            /.*\(/.test(trimmed) ||
            XRegExp(String.raw`^([d\d\p{P}+-]*|[d\p{P}+-]*)$`).test(trimmed) ||
            /ALPHAC/.test(trimmed) ||
            /^(Actor<id>|ExtraDropItem|EquipLearnSkill|GameOver|Iconset|Window|true|false|MActor%d|[wr]b|\\f|\\n|\[[A-Z]*\])$/.test(
                trimmed,
            )
        ) {
            continue;
        }

        if (romanize) {
            trimmed = romanizeString(trimmed);
        }

        strings.push(trimmed);
    }

    await writeTextFile(await join(otherPath, "scripts.txt"), strings.join("\n"));
    await writeTextFile(await join(otherPath, "scripts_trans.txt"), "\n".repeat(strings.length));
}

export function applyTheme(sheet: CSSStyleSheet, theme: Theme | [string, string]) {
    if (Array.isArray(theme)) {
        const [id, value] = theme;

        for (const rule of sheet.cssRules) {
            if (id.endsWith("Focused") && rule.selectorText === `.${id}:focus`) {
                rule.style.setProperty(rule.style[0], value);
            } else if (id.endsWith("Hovered") && rule.selectorText === `.${id}:hover`) {
                rule.style.setProperty(rule.style[0], value);
            } else if (rule.selectorText === `.${id}`) {
                const styleLength = rule.style.length;
                if (styleLength > 1) {
                    for (let i = 0; i < styleLength; i++) {
                        rule.style.setProperty(rule.style[i], value);
                    }
                    continue;
                }

                rule.style.setProperty(rule.style[0], value);
            }
        }
    } else {
        for (const [key, value] of Object.entries(theme)) {
            for (const rule of sheet.cssRules) {
                if (key.endsWith("Focused") && rule.selectorText === `.${key}:focus`) {
                    rule.style.setProperty(rule.style[0], value);
                } else if (key.endsWith("Hovered") && rule.selectorText === `.${key}:hover`) {
                    rule.style.setProperty(rule.style[0], value);
                } else if (rule.selectorText === `.${key}`) {
                    const styleLength = rule.style.length;
                    if (styleLength > 1) {
                        for (let i = 0; i < styleLength; i++) {
                            rule.style.setProperty(rule.style[i], value);
                        }
                        continue;
                    }

                    rule.style.setProperty(rule.style[0], value);
                }
            }
        }
    }
}

export function applyLocalization(localization: Localization, theme: Theme | null = null) {
    for (const [key, value] of Object.entries(localization)) {
        if (theme) {
            if (key in theme) {
                continue;
            }
        }

        const element = document.querySelectorAll(`.${key}`) as NodeListOf<HTMLElement> | null;
        if (!element) {
            continue;
        }

        if (key.endsWith("Title")) {
            for (const elem of element) {
                elem.title = value;
            }
        } else {
            for (const elem of element) {
                elem.innerHTML = value;
            }
        }
    }
}

export function getThemeStyleSheet(): CSSStyleSheet | undefined {
    for (const styleSheet of document.styleSheets) {
        for (const rule of styleSheet.cssRules) {
            if (rule.selectorText === ".backgroundDark") {
                return styleSheet;
            }
        }
    }
}

export function animateProgressText(progressText: HTMLElement, interval = 500) {
    const baseText = progressText.textContent?.replace(/\.+$/, "");
    let dots = 0;

    function updateText() {
        progressText.textContent = baseText + ".".repeat(dots);
        dots = (dots + 1) % 4;
    }

    return setInterval(updateText, interval);
}
