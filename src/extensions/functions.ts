import { Localization } from "./localization";

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
                    const styleLength = rule.style.length;
                    if (styleLength > 1) {
                        for (let i = 0; i < styleLength; i++) {
                            rule.style.setProperty(rule.style[i], value);
                        }
                        continue;
                    }

                    rule.style.setProperty(rule.style[0], value);
                } else if (key.endsWith("Hovered") && rule.selectorText === `.${key}:hover`) {
                    const styleLength = rule.style.length;
                    if (styleLength > 1) {
                        for (let i = 0; i < styleLength; i++) {
                            rule.style.setProperty(rule.style[i], value);
                        }
                        continue;
                    }

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

export const join = (...strings: string[]): string => strings.join("/");

export class CompileSettings {
    initialized: boolean;
    logging: boolean;
    romanize: boolean;
    mapsProcessingMode: number;
    disableCustomProcessing: boolean;
    customOutputPath: {
        enabled: boolean;
        path: string;
    };
    disableProcessing: {
        enabled: boolean;
        of: {
            maps: boolean;
            other: boolean;
            system: boolean;
            plugins: boolean;
        };
    };
    doNotAskAgain: boolean;

    constructor() {
        this.initialized = false;
        this.logging = false;
        this.romanize = false;
        this.mapsProcessingMode = 0;
        this.disableCustomProcessing = false;
        this.customOutputPath = { enabled: false, path: "" };
        this.disableProcessing = { enabled: false, of: { maps: false, other: false, system: false, plugins: false } };
        this.doNotAskAgain = true;
    }
}
