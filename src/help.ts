import { readTextFile } from "@tauri-apps/api/fs";
import { BaseDirectory, join } from "@tauri-apps/api/path";
import { HelpWindowLocalization } from "./extensions/localization";
const { Resource } = BaseDirectory;

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

    const sheet = getThemeStyleSheet() as CSSStyleSheet;

    const helpTitle = document.getElementById("help-title") as HTMLDivElement;
    const help = document.getElementById("help") as HTMLDivElement;
    const hotkeysTitle = document.getElementById("hotkeys-title") as HTMLDivElement;
    const hotkeys = document.getElementById("hotkeys") as HTMLDivElement;

    const { theme, language } = JSON.parse(
        await readTextFile(await join("res", "settings.json"), { dir: Resource }),
    ) as Settings;

    const windowLocalization = new HelpWindowLocalization(language);
    const themeObj: Theme = JSON.parse(await readTextFile(await join("res", "themes.json")))[theme];

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

    helpTitle.innerHTML = windowLocalization.helpTitle;
    help.innerHTML = windowLocalization.help;
    hotkeysTitle.innerHTML = windowLocalization.hotkeysTitle;
    hotkeys.innerHTML = windowLocalization.hotkeys;
});
