import { readTextFile } from "@tauri-apps/api/fs";
import { BaseDirectory, join } from "@tauri-apps/api/path";
import { HotkeysLocalization } from "./localization";

document.addEventListener("DOMContentLoaded", async () => {
    function getThemeStyleSheet(): CSSStyleSheet | undefined {
        for (const styleSheet of document.styleSheets) {
            for (const rule of styleSheet.cssRules) {
                if (rule.selectorText === ".backgroundDark") {
                    return styleSheet;
                }
            }
        }
    }

    const sheet: CSSStyleSheet = getThemeStyleSheet()!;

    const hotkeysTitle = document.getElementById("hotkeys-title") as HTMLDivElement;
    const hotkeys = document.getElementById("hotkeys") as HTMLDivElement;

    const { theme, language } = JSON.parse(
        await readTextFile(await join("../res", "settings.json"), { dir: BaseDirectory.Resource })
    ) as Settings;

    const hotkeysLocalization: HotkeysLocalization = new HotkeysLocalization(language);
    const themeObj: Theme = JSON.parse(await readTextFile(await join("../res", "themes.json")))[theme];

    for (const [key, value] of Object.entries(themeObj)) {
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

    hotkeysTitle.innerHTML = hotkeysLocalization.hotkeysTitle;
    hotkeys.innerHTML = hotkeysLocalization.hotkeys;
});
