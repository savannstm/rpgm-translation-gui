import { applyLocalization, applyTheme, getThemeStyleSheet } from "./extensions/functions";
import { SettingsWindowLocalization } from "./extensions/localization";
import "./extensions/math-extensions";

import { convertFileSrc } from "@tauri-apps/api/core";
import { emit, once } from "@tauri-apps/api/event";
import { BaseDirectory } from "@tauri-apps/api/path";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { readDir, readTextFile } from "@tauri-apps/plugin-fs";
import { platform as getPlatform } from "@tauri-apps/plugin-os";
const appWindow = getCurrentWebviewWindow();
const { Resource } = BaseDirectory;

interface FontObject extends Record<string, string> {
    font: string;
    name: string;
}

document.addEventListener("DOMContentLoaded", async () => {
    let settings!: Settings;

    await once<Settings>("settings", (data) => {
        settings = data.payload;
    });

    await emit("fetch-settings");

    while (!settings) {
        await new Promise((resolve) => setTimeout(resolve, 100));
    }

    applyTheme(
        getThemeStyleSheet() as CSSStyleSheet,
        JSON.parse(await readTextFile("res/themes.json", { baseDir: Resource }))[settings.theme],
    );

    applyLocalization(new SettingsWindowLocalization(settings.language));

    const backupCheck = document.getElementById("backup-check") as HTMLSpanElement;
    const backupSettings = document.getElementById("backup-settings") as HTMLDivElement;
    const backupMaxInput = document.getElementById("backup-max-input") as HTMLInputElement;
    const backupPeriodInput = document.getElementById("backup-period-input") as HTMLInputElement;
    const fontSelect = document.getElementById("font-select") as HTMLSelectElement;

    async function fetchFonts(): Promise<FontObject | undefined> {
        const fontsObject = {} as FontObject;
        const platform = await getPlatform();
        let fontPath: string;

        switch (platform) {
            case "windows":
                fontPath = "C:/Windows/Fonts";
                break;
            case "linux":
                fontPath = "/usr/share/fonts";
                break;
            default:
                return;
        }

        for (const entry of await readDir(fontPath)) {
            const name = entry.name as string;
            const extension = name.slice(-3);

            if (["ttf", "otf"].includes(extension)) {
                fontsObject[join(fontPath, name)] = name;
            }
        }

        return fontsObject;
    }

    backupMaxInput.value = settings.backup.max.toString();
    backupPeriodInput.value = settings.backup.period.toString();
    backupCheck.innerHTML = settings.backup.enabled ? "check" : "";

    if (!backupCheck.textContent) {
        backupSettings.classList.add("hidden");
        backupSettings.classList.add("-translate-y-full");
    } else {
        backupSettings.classList.add("flex");
        backupSettings.classList.add("translate-y-0");
    }

    for (const [path, name] of Object.entries((await fetchFonts()) as object)) {
        const optionElement = document.createElement("option");

        optionElement.id = path;
        optionElement.value = name;
        optionElement.innerHTML = name;

        fontSelect.appendChild(optionElement);
    }

    fontSelect.addEventListener("change", async () => {
        for (const element of fontSelect.children as HTMLCollectionOf<HTMLOptionElement>) {
            if (element.value === fontSelect.value) {
                const font = new FontFace("font", `url(${convertFileSrc(element.id)})`);
                document.fonts.add(await font.load());
                document.body.style.fontFamily = "font";
            }
        }
    });

    backupCheck.addEventListener("click", () => {
        if (!backupCheck.textContent) {
            backupSettings.classList.replace("hidden", "flex");

            requestAnimationFrame(() => backupSettings.classList.replace("-translate-y-full", "translate-y-0"));

            backupCheck.innerHTML = "check";
        } else {
            backupSettings.classList.replace("translate-y-0", "-translate-y-full");

            backupSettings.addEventListener("transitionend", () => backupSettings.classList.replace("flex", "hidden"), {
                once: true,
            });

            backupCheck.innerHTML = "";
        }
    });

    backupMaxInput.addEventListener("input", () => {
        backupMaxInput.value = backupMaxInput.value.replaceAll(/[^0-9]/g, "");
        backupMaxInput.value = Math.clamp(Number.parseInt(backupMaxInput.value), 1, 99).toString();
    });

    backupPeriodInput.addEventListener("input", () => {
        backupPeriodInput.value = backupPeriodInput.value.replaceAll(/[^0-9]/g, "");
        backupPeriodInput.value = Math.clamp(Number.parseInt(backupPeriodInput.value), 60, 3600).toString();
    });

    appWindow.onCloseRequested(async () => {
        await emit("backup-settings", [
            Boolean(backupCheck.textContent),
            Number.parseInt(backupMaxInput.value),
            Number.parseInt(backupPeriodInput.value),
        ]);
    });
});
