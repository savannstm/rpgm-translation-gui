import { BaseDirectory, readTextFile, writeTextFile } from "@tauri-apps/api/fs";
import { join } from "@tauri-apps/api/path";
import { appWindow } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event";
import { CompileWindowLocalization } from "./extensions/localization";
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

    const sheet: CSSStyleSheet = getThemeStyleSheet() as CSSStyleSheet;

    const settings = JSON.parse(await readTextFile("res/settings.json", { dir: Resource })) as Settings;

    const projectPath = settings.project as string;
    const theme = settings.theme;
    const language = settings.language;

    const windowLocalization = new CompileWindowLocalization(language);
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
    const loggingCheckbox = document.getElementById("logging-checkbox") as HTMLSpanElement;
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
        await readTextFile(await join(projectPath, "compile-settings.json")),
    );

    loggingCheckbox.innerHTML = compileSettings.logging ? "check" : "";

    if (compileSettings.shuffle.enabled) {
        shuffleCheckbox.innerHTML = "check";
        shuffleSettings.classList.add("flex", "translate-y-0");
    } else {
        shuffleCheckbox.innerHTML = "";
        shuffleSettings.classList.add("hidden", "-translate-y-full");
    }

    customParsingCheckbox.innerHTML = compileSettings.disableCustomParsing ? "check" : "";

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
        outputPath.value = projectPath;
        compileSettings.customOutputPath.path = projectPath;
    }

    disableMapsProcessingCheckbox.innerHTML = compileSettings.disableProcessing.of.maps ? "check" : "";
    disableOtherProcessingCheckbox.innerHTML = compileSettings.disableProcessing.of.other ? "check" : "";
    disableSystemProcessingCheckbox.innerHTML = compileSettings.disableProcessing.of.system ? "check" : "";
    disablePluginsProcessingCheckbox.innerHTML = compileSettings.disableProcessing.of.plugins ? "check" : "";

    settingsContainer.addEventListener("click", async (event) => {
        const target = event.target as HTMLElement;

        switch (target.id) {
            case "logging-checkbox":
                if (!loggingCheckbox.textContent) {
                    loggingCheckbox.innerHTML = "check";
                    compileSettings.logging = true;
                } else {
                    loggingCheckbox.innerHTML = "";
                    compileSettings.logging = false;
                }

                break;
            case "shuffle-checkbox":
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
            case "custom-parsing-checkbox":
                if (!customParsingCheckbox.textContent) {
                    customParsingCheckbox.innerHTML = "check";
                    compileSettings.disableCustomParsing = true;
                } else {
                    customParsingCheckbox.innerHTML = "";
                    compileSettings.disableCustomParsing = false;
                }
                break;
            case "custom-output-path-checkbox":
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
            case "disable-processing-checkbox":
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
            case "disable-maps-processing-checkbox":
                if (!disableMapsProcessingCheckbox.textContent) {
                    disableMapsProcessingCheckbox.innerHTML = "check";
                    compileSettings.disableProcessing.of.maps = true;
                } else {
                    disableMapsProcessingCheckbox.innerHTML = "";
                    compileSettings.disableProcessing.of.maps = false;
                }
                break;
            case "disable-other-processing-checkbox":
                if (!disableOtherProcessingCheckbox.textContent) {
                    disableOtherProcessingCheckbox.innerHTML = "check";
                    compileSettings.disableProcessing.of.other = true;
                } else {
                    disableOtherProcessingCheckbox.innerHTML = "";
                    compileSettings.disableProcessing.of.other = false;
                }
                break;

            case "disable-system-processing-checkbox":
                if (!disableSystemProcessingCheckbox.textContent) {
                    disableSystemProcessingCheckbox.innerHTML = "check";
                    compileSettings.disableProcessing.of.system = true;
                } else {
                    disableSystemProcessingCheckbox.innerHTML = "";
                    compileSettings.disableProcessing.of.system = false;
                }
                break;
            case "disable-plugins-processing-checkbox":
                if (!disablePluginsProcessingCheckbox.textContent) {
                    disablePluginsProcessingCheckbox.innerHTML = "check";
                    compileSettings.disableProcessing.of.plugins = true;
                } else {
                    disablePluginsProcessingCheckbox.innerHTML = "";
                    compileSettings.disableProcessing.of.plugins = false;
                }
                break;
            case "dont-ask-again-checkbox":
                if (!doNotAskAgainCheckbox.textContent) {
                    doNotAskAgainCheckbox.innerHTML = "check";
                    compileSettings.doNotAskAgain = true;
                } else {
                    doNotAskAgainCheckbox.innerHTML = "";
                    compileSettings.doNotAskAgain = false;
                }
                break;
        }
    });

    let compile = false;

    async function closeWindow() {
        compileSettings.initialized = true;
        await writeTextFile(await join(projectPath, "compile-settings.json"), JSON.stringify(compileSettings));

        if (compile) await emit("compile");
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
