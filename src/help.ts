import { applyLocalization, applyTheme, getThemeStyleSheet } from "./extensions/functions";
import { HelpWindowLocalization } from "./extensions/localization";

import { emit, once } from "@tauri-apps/api/event";

document.addEventListener("DOMContentLoaded", async () => {
    let settings!: Settings;
    let theme!: Theme;

    await once<[Settings, Theme]>("settings", (data) => {
        [settings, theme] = data.payload;
    });

    await emit("fetch-settings");

    while (!settings) {
        await new Promise((resolve) => setTimeout(resolve, 100));
    }

    const { language } = settings;

    applyTheme(getThemeStyleSheet() as CSSStyleSheet, theme);
    applyLocalization(new HelpWindowLocalization(language));
});
