import { exists, readTextFile } from "@tauri-apps/api/fs";
import { ReadWindowLocalization } from "./extensions/localization";
import { BaseDirectory } from "@tauri-apps/api/fs";
import { Command } from "@tauri-apps/api/shell";
import { platform as getPlatform } from "@tauri-apps/api/os";
import { appWindow } from "@tauri-apps/api/window";
import { emit, once } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { ProcessingMode } from "./types/enums";
import { join } from "@tauri-apps/api/path";
const { Resource } = BaseDirectory;

document.addEventListener("DOMContentLoaded", async () => {
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

    const settings = JSON.parse(await readTextFile("res/settings.json", { dir: Resource })) as Settings;

    const projectPath = settings.projectPath as string;
    const theme = settings.theme;
    const language = settings.language;

    const windowLocalization = new ReadWindowLocalization(language);
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

    for (const [key, value] of Object.entries(windowLocalization)) {
        const element = document.querySelectorAll(`.${key}`) as NodeListOf<HTMLElement> | null;
        if (!element) {
            continue;
        }

        if (key.endsWith("Title")) {
            for (const elem of element) {
                elem.title = value;
            }
        } else {
            for (const elem of element) {
                elem.innerHTML = value;
            }
        }
    }

    const settingsContainer = document.getElementById("settings-container") as HTMLDivElement;
    const readingModeSelect = document.getElementById("reading-mode-select") as HTMLSelectElement;
    const modeDescription = document.getElementById("mode-description") as HTMLDivElement;
    const loggingCheckbox = document.getElementById("logging-checkbox") as HTMLSpanElement;
    const romanizeCheckbox = document.getElementById("romanize-checkbox") as HTMLSpanElement;
    const customParsingCheckbox = document.getElementById("custom-parsing-checkbox") as HTMLSpanElement;
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
            case customParsingCheckbox.id:
                if (!customParsingCheckbox.textContent) {
                    customParsingCheckbox.innerHTML = "check";
                } else {
                    customParsingCheckbox.innerHTML = "";
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
            const title = data.payload[0];
            const engine = data.payload[1];

            let originalDir = await join(projectPath, "Data");

            if (!(await exists(originalDir))) {
                originalDir = await join(projectPath, "data");

                if (!(await exists(originalDir))) {
                    originalDir = await join(projectPath, "original");
                }
            }

            const progressText = document.getElementById("progress-text") as HTMLDivElement;

            function animateProgressText() {
                if (!progressText.innerHTML.endsWith("...")) {
                    setTimeout(() => {
                        progressText.innerHTML += ".";
                        animateProgressText();
                    }, 500);
                } else {
                    progressText.innerHTML = progressText.innerHTML.slice(0, -3);
                    animateProgressText();
                }
            }

            progressText.innerHTML =
                readingModeSelect.value === "append"
                    ? windowLocalization.readingInAppendMode
                    : windowLocalization.readingInForceMode;

            const progressWindow = document.getElementById("progress-window") as HTMLDivElement;
            progressWindow.classList.remove("hidden");

            animateProgressText();

            if (engine === "new") {
                await invoke<string>("read", {
                    projectPath: projectPath,
                    originalPath: originalDir,
                    gameTitle: title,
                    romanize: romanizeCheckbox.textContent ? true : false,
                    disableCustomParsing: customParsingCheckbox.textContent ? true : false,
                    disableProcessing: Object.values(disableProcessings).slice(0, -1),
                    logging: false,
                    processingMode: readingModeSelect.value === "append" ? ProcessingMode.Append : ProcessingMode.Force,
                });
            } else {
                const platform = await getPlatform();

                await new Command(platform === "win32" ? "rvpacker-txt-win-read" : "rvpacker-txt-linux-read", [
                    "read",
                    "--input-dir",
                    projectPath,
                    romanizeCheckbox.textContent ? "--romanize" : "",
                    customParsingCheckbox.textContent ? "--disable-custom-parsing" : "",
                    "--disable-processing",
                    Object.keys(disableProcessings)
                        .filter((key) => disableProcessings[key])
                        .join(","),
                    `--${readingModeSelect.value}`,
                    "--silent",
                ]).execute();
            }

            await emit("restart");
            appWindow.close();
        });

        await emit("fetch");
    });
});
