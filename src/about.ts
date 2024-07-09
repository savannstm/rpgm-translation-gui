import { open as openLink } from "@tauri-apps/api/shell";
import { readTextFile } from "@tauri-apps/api/fs";
import { BaseDirectory, join } from "@tauri-apps/api/path";
import { getVersion } from "@tauri-apps/api/app";

window.addEventListener("DOMContentLoaded", async () => {
    let sheet: CSSStyleSheet;

    for (const styleSheet of document.styleSheets) {
        for (const rule of styleSheet.cssRules) {
            if (rule.selectorText === ".backgroundDark") {
                sheet = styleSheet;
                break;
            }
        }
    }

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

    const { theme, language } = JSON.parse(
        await readTextFile(await join("../res", "settings.json"), { dir: BaseDirectory.Resource })
    ) as Settings;

    let aboutLocalization: aboutLocalization;

    switch (language) {
        case "ru":
            aboutLocalization = JSON.parse(
                await readTextFile(await join("../res", "ru.json"), { dir: BaseDirectory.Resource })
            ).about;
            break;
        default:
        case "en":
            aboutLocalization = JSON.parse(
                await readTextFile(await join("../res", "en.json"), { dir: BaseDirectory.Resource })
            ).about;
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

    version.innerHTML = aboutLocalization.version;
    versionNumber.innerHTML = await getVersion();
    about.innerHTML = aboutLocalization.about;
    socials.innerHTML = aboutLocalization.socials;
    vkLink.innerHTML = aboutLocalization.vkLink;
    tgLink.innerHTML = aboutLocalization.tgLink;
    githubLink.innerHTML = aboutLocalization.githubLink;
    license.innerHTML = aboutLocalization.license;

    const links = new Map([
        [vkLink, "https://vk.com/stivhuis228"],
        [tgLink, "https://t.me/Arsen1337Curduke"],
        [githubLink, "https://github.com/savannstm/fh-termina-json-writer"],
        [licenseLink, "http://www.wtfpl.net/about"],
        [wtpflLink, "http://www.wtfpl.net"],
    ]);

    for (const [id, url] of links) {
        id.addEventListener("click", async () => await openLink(url));
    }
});
