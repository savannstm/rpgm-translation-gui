import { applyLocalization, applyTheme, getThemeStyleSheet } from "./extensions/functions";
import { CompileWindowLocalization } from "./extensions/localization";

import { open as openPath } from "@tauri-apps/api/dialog";
import { emit } from "@tauri-apps/api/event";
import { BaseDirectory, readTextFile, writeTextFile } from "@tauri-apps/api/fs";
import { join } from "@tauri-apps/api/path";
import { appWindow } from "@tauri-apps/api/window";
const { Resource } = BaseDirectory;

document.addEventListener("DOMContentLoaded", async () => {
    const sheet = getThemeStyleSheet() as CSSStyleSheet;

    const { projectPath, theme, language } = JSON.parse(
        await readTextFile("res/settings.json", { dir: Resource }),
    ) as Settings;

    const windowLocalization = new CompileWindowLocalization(language);
    const themeObj: Theme = JSON.parse(await readTextFile("res/themes.json", { dir: Resource }))[theme];

    applyTheme(sheet, themeObj);
    applyLocalization(windowLocalization);

    const settingsContainer = document.getElementById("settings-container") as HTMLDivElement;
    const loggingCheckbox = document.getElementById("logging-checkbox") as HTMLSpanElement;
    const romanizeCheckbox = document.getElementById("romanize-checkbox") as HTMLSpanElement;
    const shuffleCheckbox = document.getElementById("shuffle-checkbox") as HTMLSpanElement;
    const shuffleSettings = document.getElementById("shuffle-settings") as HTMLDivElement;
    const customParsingCheckbox = document.getElementById("custom-parsing-checkbox") as HTMLSpanElement;
    const customOutputPathCheckbox = document.getElementById("custom-output-path-checkbox") as HTMLSpanElement;
    const customOutputPathSettings = document.getElementById("custom-output-path-settings") as HTMLDivElement;
    const disableProcessingCheckbox = document.getElementById("disable-processing-checkbox") as HTMLSpanElement;
    const disableProcessingSettings = document.getElementById("disable-processing-settings") as HTMLDivElement;
    const doNotAskAgainCheckbox = document.getElementById("dont-ask-again-checkbox") as HTMLSpanElement;
    const shuffleSelect = document.getElementById("shuffle-select") as HTMLSelectElement;
    const outputPath = document.getElementById("output-path") as HTMLInputElement;
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
    const compileButton = document.getElementById("compile-button") as HTMLButtonElement;

    const compileSettings: CompileSettings = JSON.parse(
        await readTextFile(await join(projectPath, ".rpgm-translation-gui", "compile-settings.json")),
    );

    loggingCheckbox.innerHTML = compileSettings.logging ? "check" : "";
    romanizeCheckbox.innerHTML = compileSettings.romanize ? "check" : "";

    if (compileSettings.shuffle.enabled) {
        shuffleCheckbox.innerHTML = "check";
        shuffleSettings.classList.add("flex", "translate-y-0");
    } else {
        shuffleCheckbox.innerHTML = "";
        shuffleSettings.classList.add("hidden", "-translate-y-full");
    }

    customParsingCheckbox.innerHTML = compileSettings.disableCustomProcessing ? "check" : "";

    if (compileSettings.customOutputPath.enabled) {
        customOutputPathCheckbox.innerHTML = "check";
        customOutputPathSettings.classList.add("flex", "translate-y-0");
    } else {
        customOutputPathCheckbox.innerHTML = "";
        customOutputPathSettings.classList.add("hidden", "-translate-y-full");
    }

    if (compileSettings.disableProcessing.enabled) {
        disableProcessingCheckbox.innerHTML = "check";
        disableProcessingSettings.classList.add("flex", "translate-y-0");
    } else {
        disableProcessingCheckbox.innerHTML = "";
        disableProcessingSettings.classList.add("hidden", "-translate-y-full");
    }

    doNotAskAgainCheckbox.innerHTML = compileSettings.doNotAskAgain ? "check" : "";

    shuffleSelect.value = compileSettings.shuffle.level.toString();

    if (compileSettings.customOutputPath.path !== "") {
        outputPath.value = compileSettings.customOutputPath.path;
    } else {
        const path = await join(await join(".rpgm-translation-gui", projectPath));
        outputPath.value = path;
        compileSettings.customOutputPath.path = path;
    }

    disableMapsProcessingCheckbox.innerHTML = compileSettings.disableProcessing.of.maps ? "check" : "";
    disableOtherProcessingCheckbox.innerHTML = compileSettings.disableProcessing.of.other ? "check" : "";
    disableSystemProcessingCheckbox.innerHTML = compileSettings.disableProcessing.of.system ? "check" : "";
    disablePluginsProcessingCheckbox.innerHTML = compileSettings.disableProcessing.of.plugins ? "check" : "";

    settingsContainer.addEventListener("click", async (event) => {
        const target = event.target as HTMLElement;

        switch (target.id) {
            case loggingCheckbox.id:
                if (!loggingCheckbox.textContent) {
                    loggingCheckbox.innerHTML = "check";
                    compileSettings.logging = true;
                } else {
                    loggingCheckbox.innerHTML = "";
                    compileSettings.logging = false;
                }
                break;
            case romanizeCheckbox.id:
                if (!romanizeCheckbox.textContent) {
                    romanizeCheckbox.innerHTML = "check";
                    compileSettings.romanize = true;
                } else {
                    romanizeCheckbox.innerHTML = "";
                    compileSettings.romanize = false;
                }
                break;
            case shuffleCheckbox.id:
                if (!shuffleCheckbox.textContent) {
                    shuffleSettings.classList.replace("hidden", "flex");

                    requestAnimationFrame(() =>
                        shuffleSettings.classList.replace("-translate-y-full", "translate-y-0"),
                    );

                    shuffleCheckbox.innerHTML = "check";
                    compileSettings.shuffle.enabled = true;
                } else {
                    shuffleSettings.classList.replace("translate-y-0", "-translate-y-full");

                    shuffleSettings.addEventListener(
                        "transitionend",
                        () => shuffleSettings.classList.replace("flex", "hidden"),
                        {
                            once: true,
                        },
                    );

                    shuffleCheckbox.innerHTML = "";
                    compileSettings.shuffle.enabled = false;
                }
                break;
            case customParsingCheckbox.id:
                if (!customParsingCheckbox.textContent) {
                    customParsingCheckbox.innerHTML = "check";
                    compileSettings.disableCustomProcessing = true;
                } else {
                    customParsingCheckbox.innerHTML = "";
                    compileSettings.disableCustomProcessing = false;
                }
                break;
            case customOutputPathCheckbox.id:
                if (!customOutputPathCheckbox.textContent) {
                    customOutputPathSettings.classList.replace("hidden", "flex");

                    requestAnimationFrame(() =>
                        customOutputPathSettings.classList.replace("-translate-y-full", "translate-y-0"),
                    );

                    customOutputPathCheckbox.innerHTML = "check";
                    compileSettings.customOutputPath.enabled = true;
                } else {
                    customOutputPathSettings.classList.replace("translate-y-0", "-translate-y-full");

                    customOutputPathSettings.addEventListener(
                        "transitionend",
                        () => customOutputPathSettings.classList.replace("flex", "hidden"),
                        {
                            once: true,
                        },
                    );

                    customOutputPathCheckbox.innerHTML = "";
                    compileSettings.customOutputPath.enabled = false;
                }
                break;
            case disableProcessingCheckbox.id:
                if (!disableProcessingCheckbox.textContent) {
                    disableProcessingSettings.classList.replace("hidden", "flex");

                    requestAnimationFrame(() =>
                        disableProcessingSettings.classList.replace("-translate-y-full", "translate-y-0"),
                    );

                    disableProcessingCheckbox.innerHTML = "check";
                    compileSettings.disableProcessing.enabled = true;
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
                    compileSettings.disableProcessing.enabled = false;
                }
                break;
            case disableMapsProcessingCheckbox.id:
                if (!disableMapsProcessingCheckbox.textContent) {
                    disableMapsProcessingCheckbox.innerHTML = "check";
                    compileSettings.disableProcessing.of.maps = true;
                } else {
                    disableMapsProcessingCheckbox.innerHTML = "";
                    compileSettings.disableProcessing.of.maps = false;
                }
                break;
            case disableOtherProcessingCheckbox.id:
                if (!disableOtherProcessingCheckbox.textContent) {
                    disableOtherProcessingCheckbox.innerHTML = "check";
                    compileSettings.disableProcessing.of.other = true;
                } else {
                    disableOtherProcessingCheckbox.innerHTML = "";
                    compileSettings.disableProcessing.of.other = false;
                }
                break;
            case disableSystemProcessingCheckbox.id:
                if (!disableSystemProcessingCheckbox.textContent) {
                    disableSystemProcessingCheckbox.innerHTML = "check";
                    compileSettings.disableProcessing.of.system = true;
                } else {
                    disableSystemProcessingCheckbox.innerHTML = "";
                    compileSettings.disableProcessing.of.system = false;
                }
                break;
            case disablePluginsProcessingCheckbox.id:
                if (!disablePluginsProcessingCheckbox.textContent) {
                    disablePluginsProcessingCheckbox.innerHTML = "check";
                    compileSettings.disableProcessing.of.plugins = true;
                } else {
                    disablePluginsProcessingCheckbox.innerHTML = "";
                    compileSettings.disableProcessing.of.plugins = false;
                }
                break;
            case doNotAskAgainCheckbox.id:
                if (!doNotAskAgainCheckbox.textContent) {
                    doNotAskAgainCheckbox.innerHTML = "check";
                    compileSettings.doNotAskAgain = true;
                } else {
                    doNotAskAgainCheckbox.innerHTML = "";
                    compileSettings.doNotAskAgain = false;
                }
                break;
            case "select-output-path": {
                const directory = (await openPath({ directory: true, multiple: false })) as string;

                if (directory) {
                    outputPath.value = directory;
                }
                break;
            }
        }
    });

    let compile = false;

    async function closeWindow() {
        compileSettings.initialized = true;

        await writeTextFile(
            await join(projectPath, ".rpgm-translation-gui", "compile-settings.json"),
            JSON.stringify(compileSettings),
        );

        if (compile) {
            await emit("compile");
        }

        appWindow.close();
    }

    compileButton.addEventListener("click", async () => {
        compile = true;
        await closeWindow();
    });

    await appWindow.onCloseRequested(async () => {
        await closeWindow();
    });
});
