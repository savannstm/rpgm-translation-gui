import { applyLocalization, applyTheme, getThemeStyleSheet } from "./extensions/functions";
import { AboutWindowLocalization } from "./extensions/localization";

import { getVersion } from "@tauri-apps/api/app";
import { emit, once } from "@tauri-apps/api/event";
import { readTextFile } from "@tauri-apps/api/fs";
import { BaseDirectory } from "@tauri-apps/api/path";
import { open as openLink } from "@tauri-apps/api/shell";
const { Resource } = BaseDirectory;

window.addEventListener("DOMContentLoaded", async () => {
    let settings!: Settings;

    await once<Settings>("settings", (data) => {
        settings = data.payload;
    });

    await emit("fetch-settings");

    while (!settings) {
        await new Promise((resolve) => setTimeout(resolve, 100));
    }

    const { theme, language } = settings;

    applyTheme(
        getThemeStyleSheet() as CSSStyleSheet,
        JSON.parse(await readTextFile("res/themes.json", { dir: Resource }))[theme],
    );

    applyLocalization(new AboutWindowLocalization(language));

    (document.getElementById("version-number") as HTMLSpanElement).innerHTML = await getVersion();

    const links = new Map([
        [document.getElementById("vk-link") as HTMLAnchorElement, "https://vk.com/stivhuis228"],
        [document.getElementById("tg-link") as HTMLAnchorElement, "https://t.me/Arsen1337Curduke"],
        [
            document.getElementById("github-link") as HTMLAnchorElement,
            "https://github.com/savannstm/rpgm-translation-gui",
        ],
        [document.getElementById("license-link") as HTMLAnchorElement, "http://www.wtfpl.net/about"],
        [document.getElementById("wtfpl-link") as HTMLAnchorElement, "http://www.wtfpl.net"],
    ]);

    for (const [id, url] of links) {
        id.addEventListener("click", async () => await openLink(url));
    }
});
