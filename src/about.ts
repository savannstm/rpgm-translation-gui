import { open as openLink } from "@tauri-apps/api/shell";
import { readTextFile } from "@tauri-apps/api/fs";
import { BaseDirectory } from "@tauri-apps/api/path";
import { getVersion } from "@tauri-apps/api/app";
import { AboutWindowLocalization } from "./extensions/localization";
const { Resource } = BaseDirectory;

window.addEventListener("DOMContentLoaded", async () => {
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

    const { theme, language } = JSON.parse(await readTextFile("res/settings.json", { dir: Resource })) as Settings;

    const windowLocalization = new AboutWindowLocalization(language);
    const themeObj: Theme = JSON.parse(await readTextFile("res/themes.json", { dir: Resource }))[theme];

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
