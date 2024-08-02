import { applyTheme, getThemeStyleSheet } from "./extensions/functions";
import { SettingsWindowLocalization } from "./extensions/localization";

import { readTextFile, writeTextFile } from "@tauri-apps/api/fs";
import { BaseDirectory } from "@tauri-apps/api/path";
import { appWindow } from "@tauri-apps/api/window";
const { Resource } = BaseDirectory;

document.addEventListener("DOMContentLoaded", async () => {
    const sheet = getThemeStyleSheet() as CSSStyleSheet;

    const settings: Settings = JSON.parse(await readTextFile("res/settings.json", { dir: Resource }));

    applyTheme(sheet, JSON.parse(await readTextFile("res/themes.json", { dir: Resource }))[settings.theme]);
    const windowLocalization = new SettingsWindowLocalization(settings.language);

    const backupPeriodLabel = document.getElementById("backup-period-label") as HTMLSpanElement;
    const backupPeriodNote = document.getElementById("backup-period-note") as HTMLSpanElement;
    const backupMaxLabel = document.getElementById("backup-max-label") as HTMLSpanElement;
    const backupMaxNote = document.getElementById("backup-max-note") as HTMLSpanElement;
    const backup = document.getElementById("backup") as HTMLSpanElement;
    const backupCheck = document.getElementById("backup-check") as HTMLSpanElement;
    const backupSettings = document.getElementById("backup-settings") as HTMLDivElement;
    const backupMaxInput = document.getElementById("backup-max-input") as HTMLInputElement;
    const backupPeriodInput = document.getElementById("backup-period-input") as HTMLInputElement;

    backupPeriodLabel.innerHTML = windowLocalization.backupPeriodLabel;
    backupPeriodNote.innerHTML = windowLocalization.backupPeriodNote;
    backupMaxLabel.innerHTML = windowLocalization.backupMaxLabel;
    backupMaxNote.innerHTML = windowLocalization.backupMaxNote;
    backup.innerHTML = windowLocalization.backup;

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
        const backupMaxValue = Number.parseInt(backupMaxInput.value);

        backupMaxInput.value = (backupMaxValue < 1 ? 1 : backupMaxValue > 99 ? 99 : backupMaxValue).toString();
    });

    backupPeriodInput.addEventListener("input", () => {
        backupPeriodInput.value = backupPeriodInput.value.replaceAll(/[^0-9]/g, "");
        const backupPeriodValue = Number.parseInt(backupPeriodInput.value);

        backupPeriodInput.value = (
            backupPeriodValue < 60 ? 60 : backupPeriodValue > 3600 ? 3600 : backupPeriodValue
        ).toString();
    });

    appWindow.onCloseRequested(async () => {
        await writeTextFile(
            "res/settings.json",
            JSON.stringify({
                ...settings,
                backup: {
                    enabled: backupCheck.textContent ? true : false,
                    max: backupMaxInput.value,
                    period: backupPeriodInput.value,
                },
            }),
            { dir: Resource }
        );
    });
});
