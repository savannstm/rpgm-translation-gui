import { emit, once } from "@tauri-apps/api/event";
import { applyTheme, getThemeStyleSheet } from "./extensions/functions";
import { HelpWindowLocalization } from "./extensions/localization";

import { readTextFile } from "@tauri-apps/api/fs";
import { BaseDirectory } from "@tauri-apps/api/path";
const { Resource } = BaseDirectory;

document.addEventListener("DOMContentLoaded", async () => {
    let settings!: Settings;

    await once<Settings>("settings", (data) => {
        settings = data.payload;
    });

    await emit("fetch-settings");

    while (settings === undefined) {
        await new Promise((resolve) => setTimeout(resolve, 100));
    }

    const { theme, language } = settings;

    applyTheme(
        getThemeStyleSheet() as CSSStyleSheet,
        JSON.parse(await readTextFile("res/themes.json", { dir: Resource }))[theme],
    );

    const windowLocalization = new HelpWindowLocalization(language);

    const helpTitle = document.getElementById("help-title") as HTMLDivElement;
    const help = document.getElementById("help") as HTMLDivElement;
    const hotkeysTitle = document.getElementById("hotkeys-title") as HTMLDivElement;
    const hotkeys = document.getElementById("hotkeys") as HTMLDivElement;

    helpTitle.innerHTML = windowLocalization.helpTitle;
    help.innerHTML = windowLocalization.help;
    hotkeysTitle.innerHTML = windowLocalization.hotkeysTitle;
    hotkeys.innerHTML = windowLocalization.hotkeys;
});
