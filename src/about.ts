import { applyTheme, getThemeStyleSheet } from "./extensions/functions";
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

    while (settings === undefined) {
        await new Promise((resolve) => setTimeout(resolve, 100));
    }

    const { theme, language } = settings;

    applyTheme(
        getThemeStyleSheet() as CSSStyleSheet,
        JSON.parse(await readTextFile("res/themes.json", { dir: Resource }))[theme],
    );

    const windowLocalization = new AboutWindowLocalization(language);

    const version = document.getElementById("version") as HTMLSpanElement;
    const versionNumber = document.getElementById("version-number") as HTMLSpanElement;
    const about = document.getElementById("about") as HTMLDivElement;
    const socials = document.getElementById("socials") as HTMLDivElement;
    const vkLink = document.getElementById("vk-link") as HTMLAnchorElement;
    const tgLink = document.getElementById("tg-link") as HTMLAnchorElement;
    const githubLink = document.getElementById("github-link") as HTMLAnchorElement;
    const license = document.getElementById("license") as HTMLSpanElement;
    const licenseLink = document.getElementById("license-link") as HTMLAnchorElement;
    const wtpflLink = document.getElementById("wtfpl-link") as HTMLAnchorElement;

    version.innerHTML = windowLocalization.version;
    versionNumber.innerHTML = await getVersion();
    about.innerHTML = windowLocalization.about;
    socials.innerHTML = windowLocalization.socials;
    vkLink.innerHTML = windowLocalization.vkLink;
    tgLink.innerHTML = windowLocalization.tgLink;
    githubLink.innerHTML = windowLocalization.githubLink;
    license.innerHTML = windowLocalization.license;

    const links = new Map([
        [vkLink, "https://vk.com/stivhuis228"],
        [tgLink, "https://t.me/Arsen1337Curduke"],
        [githubLink, "https://github.com/savannstm/rpgm-translation-gui"],
        [licenseLink, "http://www.wtfpl.net/about"],
        [wtpflLink, "http://www.wtfpl.net"],
    ]);

    for (const [id, url] of links) {
        id.addEventListener("click", async () => await openLink(url));
    }
});
