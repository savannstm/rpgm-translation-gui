import { applyTheme, getThemeStyleSheet } from "./extensions/functions";
import { HelpWindowLocalization } from "./extensions/localization";

import { readTextFile } from "@tauri-apps/api/fs";
import { BaseDirectory, join } from "@tauri-apps/api/path";
const { Resource } = BaseDirectory;

document.addEventListener("DOMContentLoaded", async () => {
    const sheet = getThemeStyleSheet() as CSSStyleSheet;

    const { theme, language } = JSON.parse(
        await readTextFile(await join("res", "settings.json"), { dir: Resource }),
    ) as Settings;

    const windowLocalization = new HelpWindowLocalization(language);
    const themeObj: Theme = JSON.parse(await readTextFile("res/themes.json", { dir: Resource }))[theme];

    applyTheme(sheet, themeObj);

    const helpTitle = document.getElementById("help-title") as HTMLDivElement;
    const help = document.getElementById("help") as HTMLDivElement;
    const hotkeysTitle = document.getElementById("hotkeys-title") as HTMLDivElement;
    const hotkeys = document.getElementById("hotkeys") as HTMLDivElement;

    helpTitle.innerHTML = windowLocalization.helpTitle;
    help.innerHTML = windowLocalization.help;
    hotkeysTitle.innerHTML = windowLocalization.hotkeysTitle;
    hotkeys.innerHTML = windowLocalization.hotkeys;
});
