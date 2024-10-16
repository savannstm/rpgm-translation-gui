import { applyLocalization, applyTheme, getThemeStyleSheet } from "./extensions/functions";
import { AboutWindowLocalization } from "./extensions/localization";

import { getVersion } from "@tauri-apps/api/app";
import { emit, once } from "@tauri-apps/api/event";
import { open as openLink } from "@tauri-apps/plugin-shell";

document.addEventListener("DOMContentLoaded", async () => {
    let settings!: Settings;
    let theme!: Theme;

    await once<[Settings, Theme]>("settings", (data) => {
        [settings, theme] = data.payload;
    });

    await emit("fetch-settings");

    await new Promise((resolve) => setTimeout(resolve, 100));

    const { language } = settings;

    applyTheme(getThemeStyleSheet()!, theme);
    applyLocalization(new AboutWindowLocalization(language));

    (document.getElementById("version-number") as HTMLSpanElement).innerHTML = await getVersion();

    const links = new Map([
        [document.getElementById("vk-link") as HTMLAnchorElement, "https://vk.com/stivhuis228"],
        [document.getElementById("tg-link") as HTMLAnchorElement, "https://t.me/Arsen1337Curduke"],
        [document.getElementById("repo-link") as HTMLAnchorElement, "https://github.com/savannstm/rpgmtranslate"],
        [document.getElementById("license-link") as HTMLAnchorElement, "http://www.wtfpl.net/about"],
        [document.getElementById("wtfpl-link") as HTMLAnchorElement, "http://www.wtfpl.net"],
    ]);

    for (const [id, url] of links) {
        id.addEventListener("click", async () => {
            await openLink(url);
        });
    }
});
