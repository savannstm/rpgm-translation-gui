import { readTextFile } from "@tauri-apps/api/fs";
import { BaseDirectory, join } from "@tauri-apps/api/path";

document.addEventListener("DOMContentLoaded", async () => {
    let sheet: CSSStyleSheet;

    for (const styleSheet of document.styleSheets) {
        for (const rule of styleSheet.cssRules) {
            if (rule.selectorText === ".backgroundDark") {
                sheet = styleSheet;
                break;
            }
        }
    }

    const helpTitle = document.getElementById("help-title") as HTMLDivElement;
    const help = document.getElementById("help") as HTMLDivElement;

    const { theme, language } = JSON.parse(
        await readTextFile(await join("../res", "settings.json"), { dir: BaseDirectory.Resource })
    ) as Settings;

    let helpLocalization: helpLocalization;

    switch (language) {
        case "ru":
            helpLocalization = JSON.parse(
                await readTextFile(await join("../res", "ru.json"), { dir: BaseDirectory.Resource })
            ).help;
            break;
        default:
        case "en":
            helpLocalization = JSON.parse(
                await readTextFile(await join("../res", "en.json"), { dir: BaseDirectory.Resource })
            ).help;
            break;
    }

    const themeObj: Theme = JSON.parse(await readTextFile(await join("../res", "themes.json")))[theme];

    for (const [key, value] of Object.entries(themeObj)) {
        for (const rule of sheet!.cssRules) {
            if (key.endsWith("Focused") && rule.selectorText === `.${key}:focus`) {
                rule.style.setProperty(rule.style[0], value);
            } else if (key.endsWith("Hovered") && rule.selectorText === `.${key}:hover`) {
                rule.style.setProperty(rule.style[0], value);
            } else if (rule.selectorText === `.${key}`) {
                rule.style.setProperty(rule.style[0], value);
            }
        }
    }

    helpTitle.innerHTML = helpLocalization.helpTitle;
    help.innerHTML = helpLocalization.help;
});
