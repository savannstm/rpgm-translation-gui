import { Localization } from "./localization";

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
