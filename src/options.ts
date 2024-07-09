import { readTextFile, writeTextFile } from "@tauri-apps/api/fs";
import { BaseDirectory, join } from "@tauri-apps/api/path";
import { appWindow } from "@tauri-apps/api/window";

document.addEventListener("DOMContentLoaded", async () => {
    let sheet: CSSStyleSheet;

    for (const styleSheet of document.styleSheets) {
        for (const rule of styleSheet.cssRules) {
            if (rule.selectorText === ".backgroundDark") {
                sheet = styleSheet;
                break;
            }
        }
    }

    const backupPeriodLabel = document.getElementById("backup-period-label") as HTMLSpanElement;
    const backupPeriodNote = document.getElementById("backup-period-note") as HTMLSpanElement;
    const backupMaxLabel = document.getElementById("backup-max-label") as HTMLSpanElement;
    const backupMaxNote = document.getElementById("backup-max-note") as HTMLSpanElement;
    const backup = document.getElementById("backup") as HTMLSpanElement;
    const backupCheck = document.getElementById("backup-check") as HTMLSpanElement;
    const backupSettings = document.getElementById("backup-settings") as HTMLDivElement;
    const backupMaxInput = document.getElementById("backup-max-input") as HTMLInputElement;
    const backupPeriodInput = document.getElementById("backup-period-input") as HTMLInputElement;

    const settings: Settings = JSON.parse(
        await readTextFile(await join("../res", "settings.json"), { dir: BaseDirectory.Resource })
    );

    const language = settings.language;
    const theme = settings.theme;

    let optionsLocalization: optionsLocalization;

    switch (language) {
        case "ru":
            optionsLocalization = JSON.parse(
                await readTextFile(await join("../res", "ru.json"), { dir: BaseDirectory.Resource })
            ).options;
            break;
        default:
        case "en":
            optionsLocalization = JSON.parse(
                await readTextFile(await join("../res", "en.json"), { dir: BaseDirectory.Resource })
            ).options;
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

    backupPeriodLabel.innerHTML = optionsLocalization.backupPeriodLabel;
    backupPeriodNote.innerHTML = optionsLocalization.backupPeriodNote;
    backupMaxLabel.innerHTML = optionsLocalization.backupMaxLabel;
    backupMaxNote.innerHTML = optionsLocalization.backupMaxNote;
    backup.innerHTML = optionsLocalization.backup;

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
        } else {
            backupSettings.classList.replace("translate-y-0", "-translate-y-full");
            backupSettings.addEventListener("transitionend", () => backupSettings.classList.replace("flex", "hidden"), {
                once: true,
            });
        }
        backupCheck.innerHTML = !backupCheck.textContent ? "check" : "";
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

    appWindow.onCloseRequested(async (): Promise<void> => {
        writeTextFile(
            await join("../res", "settings.json"),
            JSON.stringify({
                ...settings,
                backup: {
                    enabled: backupCheck.textContent ? true : false,
                    max: backupMaxInput.value,
                    period: backupPeriodInput.value,
                },
            }),
            { dir: BaseDirectory.Resource }
        );
    });
});
