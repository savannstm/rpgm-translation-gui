import {
    animateProgressText,
    applyLocalization,
    applyTheme,
    getThemeStyleSheet,
    readScripts,
} from "./extensions/functions";
import { invokeRead } from "./extensions/invokes";
import { ReadWindowLocalization } from "./extensions/localization";
import { EngineType, ProcessingMode } from "./types/enums";

import { emit, once } from "@tauri-apps/api/event";
import { BaseDirectory, exists, readTextFile } from "@tauri-apps/api/fs";
import { join } from "@tauri-apps/api/path";
import { appWindow } from "@tauri-apps/api/window";
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

    const { projectPath, theme, language, engineType } = settings;

    applyTheme(
        getThemeStyleSheet() as CSSStyleSheet,
        JSON.parse(await readTextFile("res/themes.json", { dir: Resource }))[theme],
    );

    const windowLocalization = new ReadWindowLocalization(language);
    applyLocalization(windowLocalization);

    const settingsContainer = document.getElementById("settings-container") as HTMLDivElement;
    const readingModeSelect = document.getElementById("reading-mode-select") as HTMLSelectElement;
    const modeDescription = document.getElementById("mode-description") as HTMLDivElement;
    const loggingCheckbox = document.getElementById("logging-checkbox") as HTMLSpanElement;
    const romanizeCheckbox = document.getElementById("romanize-checkbox") as HTMLSpanElement;
    const customProcessingCheckbox = document.getElementById("custom-processing-checkbox") as HTMLSpanElement;
    const disableProcessingCheckbox = document.getElementById("disable-processing-checkbox") as HTMLSpanElement;
    const disableProcessingSettings = document.getElementById("disable-processing-settings") as HTMLDivElement;
    const doNotAskAgainCheckbox = document.getElementById("dont-ask-again-checkbox") as HTMLSpanElement;
    const disableMapsProcessingCheckbox = document.getElementById(
        "disable-maps-processing-checkbox",
    ) as HTMLSpanElement;
    const disableOtherProcessingCheckbox = document.getElementById(
        "disable-other-processing-checkbox",
    ) as HTMLSpanElement;
    const disableSystemProcessingCheckbox = document.getElementById(
        "disable-system-processing-checkbox",
    ) as HTMLSpanElement;
    const disablePluginsProcessingCheckbox = document.getElementById(
        "disable-plugins-processing-checkbox",
    ) as HTMLSpanElement;
    const readButton = document.getElementById("read-button") as HTMLButtonElement;

    readingModeSelect.addEventListener("change", () => {
        if (readingModeSelect.value === "append") {
            modeDescription.innerHTML = windowLocalization.appendModeDescription;
        } else if (readingModeSelect.value === "force") {
            modeDescription.innerHTML = windowLocalization.forceModeDescription;
        } else {
            modeDescription.innerHTML = "";
        }
    });

    settingsContainer.addEventListener("click", async (event) => {
        const target = event.target as HTMLElement;

        switch (target.id) {
            case loggingCheckbox.id:
                if (!loggingCheckbox.textContent) {
                    loggingCheckbox.innerHTML = "check";
                } else {
                    loggingCheckbox.innerHTML = "";
                }
                break;
            case romanizeCheckbox.id:
                if (!romanizeCheckbox.textContent) {
                    romanizeCheckbox.innerHTML = "check";
                } else {
                    romanizeCheckbox.innerHTML = "";
                }
                break;
            case customProcessingCheckbox.id:
                if (!customProcessingCheckbox.textContent) {
                    customProcessingCheckbox.innerHTML = "check";
                } else {
                    customProcessingCheckbox.innerHTML = "";
                }
                break;
            case disableProcessingCheckbox.id:
                if (!disableProcessingCheckbox.textContent) {
                    disableProcessingSettings.classList.replace("hidden", "flex");

                    requestAnimationFrame(() =>
                        disableProcessingSettings.classList.replace("-translate-y-full", "translate-y-0"),
                    );

                    disableProcessingCheckbox.innerHTML = "check";
                } else {
                    disableProcessingSettings.classList.replace("translate-y-0", "-translate-y-full");

                    disableProcessingSettings.addEventListener(
                        "transitionend",
                        () => disableProcessingSettings.classList.replace("flex", "hidden"),
                        {
                            once: true,
                        },
                    );

                    disableProcessingCheckbox.innerHTML = "";
                }
                break;
            case disableMapsProcessingCheckbox.id:
                if (!disableMapsProcessingCheckbox.textContent) {
                    disableMapsProcessingCheckbox.innerHTML = "check";
                } else {
                    disableMapsProcessingCheckbox.innerHTML = "";
                }
                break;
            case disableOtherProcessingCheckbox.id:
                if (!disableOtherProcessingCheckbox.textContent) {
                    disableOtherProcessingCheckbox.innerHTML = "check";
                } else {
                    disableOtherProcessingCheckbox.innerHTML = "";
                }
                break;
            case disableSystemProcessingCheckbox.id:
                if (!disableSystemProcessingCheckbox.textContent) {
                    disableSystemProcessingCheckbox.innerHTML = "check";
                } else {
                    disableSystemProcessingCheckbox.innerHTML = "";
                }
                break;
            case disablePluginsProcessingCheckbox.id:
                if (!disablePluginsProcessingCheckbox.textContent) {
                    disablePluginsProcessingCheckbox.innerHTML = "check";
                } else {
                    disablePluginsProcessingCheckbox.innerHTML = "";
                }
                break;
            case doNotAskAgainCheckbox.id:
                if (!doNotAskAgainCheckbox.textContent) {
                    doNotAskAgainCheckbox.innerHTML = "check";
                } else {
                    doNotAskAgainCheckbox.innerHTML = "";
                }
                break;
        }
    });

    let reading = false;

    readButton.addEventListener("click", async () => {
        if (!readingModeSelect.value) {
            alert("Select reading mode");
            return;
        }

        const disableProcessings: Record<string, boolean> = {
            maps: disableMapsProcessingCheckbox.textContent ? true : false,
            other: disableOtherProcessingCheckbox.textContent ? true : false,
            system: disableSystemProcessingCheckbox.textContent ? true : false,
            plugins: disablePluginsProcessingCheckbox.textContent ? true : false,
        };

        await once<string[]>("metadata", async (data) => {
            const gameTitle = data.payload[0];

            let originalDir: string;

            if (engineType === EngineType.New) {
                originalDir = "data";

                if (await exists(await join(projectPath, "original"))) {
                    originalDir = "original";
                }
            } else {
                originalDir = ".rpgm-translation-gui/json-data";
            }

            const progressText = document.getElementById("progress-text") as HTMLDivElement;

            progressText.innerHTML =
                readingModeSelect.value === "append"
                    ? windowLocalization.readingInAppendMode
                    : windowLocalization.readingInForceMode;

            const progressWindow = document.getElementById("progress-window") as HTMLDivElement;
            progressWindow.classList.remove("hidden");

            animateProgressText(progressText);

            reading = true;

            await invokeRead({
                projectPath,
                originalDir: originalDir,
                gameTitle,
                romanize: romanizeCheckbox.textContent ? true : false,
                disableCustomProcessing: customProcessingCheckbox.textContent ? true : false,
                disableProcessing: Object.values(disableProcessings).slice(0, -1),
                logging: false,
                processingMode: readingModeSelect.value === "append" ? ProcessingMode.Append : ProcessingMode.Force,
                engineType: engineType,
            });

            if (engineType !== EngineType.New) {
                if (!disableProcessings.plugins) {
                    await readScripts(
                        await readTextFile(await join(projectPath, originalDir, "Scripts.txt")),
                        await join(projectPath, ".rpgm-translation-gui/translation/other"),
                        romanizeCheckbox.textContent ? true : false,
                    );
                }
            }

            await emit("restart");
            appWindow.close();
        });

        await emit("fetch");
    });

    await appWindow.onCloseRequested((event) => {
        if (reading) {
            event.preventDefault();
        }
    });
});
