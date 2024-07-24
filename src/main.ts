/* eslint-disable @typescript-eslint/no-dynamic-delete */
import "./extensions/string-extensions";
import "./extensions/htmlelement-extensions";
import { EngineType, Language, ProcessingType, State } from "./types/enums";
import { MainWindowLocalization } from "./extensions/localization";

import { createDir, exists, FileEntry, readDir, readTextFile, removeFile, writeTextFile } from "@tauri-apps/api/fs";
import { BaseDirectory, join } from "@tauri-apps/api/path";
import { ask, message, open as openPath } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";
import { exit } from "@tauri-apps/api/process";
import { WebviewWindow, appWindow } from "@tauri-apps/api/window";
import { locale as getLocale, platform as getPlatform } from "@tauri-apps/api/os";
import { Command } from "@tauri-apps/api/shell";
const { Resource } = BaseDirectory;

import XRegExp from "xregexp";

document.addEventListener("DOMContentLoaded", async () => {
    const sheet = getThemeStyleSheet() as CSSStyleSheet;

    let clickTimer: null | number = null;
    let projectPath = "";
    let gameEngineType: EngineType;

    const resDir = "res";
    const translationDir = "translation";
    const backupDir = "backups";

    const mapsDir = "maps";
    const otherDir = "other";
    const pluginsDir = "plugins";

    const logFile = "replacement-log.json";

    const themesPath = await join(resDir, "themes.json");
    const settingsPath = await join(resDir, "settings.json");

    const contentContainer = document.getElementById("content-container") as HTMLDivElement;
    const searchInput = document.getElementById("search-input") as HTMLTextAreaElement;
    const replaceInput = document.getElementById("replace-input") as HTMLTextAreaElement;
    const leftPanel = document.getElementById("left-panel") as HTMLDivElement;
    const searchPanel = document.getElementById("search-results") as HTMLDivElement;
    const searchPanelFound = document.getElementById("search-content") as HTMLDivElement;
    const searchPanelReplaced = document.getElementById("replace-content") as HTMLDivElement;
    const searchCurrentPage = document.getElementById("search-current-page") as HTMLSpanElement;
    const searchTotalPages = document.getElementById("search-total-pages") as HTMLSpanElement;
    const topPanel = document.getElementById("top-panel") as HTMLDivElement;
    const topPanelButtonsDiv = document.getElementById("top-panel-buttons") as HTMLDivElement;
    const saveButton = document.getElementById("save-button") as HTMLButtonElement;
    const compileButton = document.getElementById("compile-button") as HTMLButtonElement;
    const themeButton = document.getElementById("theme-button") as HTMLButtonElement;
    const themeMenu = document.getElementById("theme-menu") as HTMLDivElement;
    const searchCaseButton = document.getElementById("case-button") as HTMLButtonElement;
    const searchWholeButton = document.getElementById("whole-button") as HTMLButtonElement;
    const searchRegexButton = document.getElementById("regex-button") as HTMLButtonElement;
    const searchTranslationButton = document.getElementById("translation-button") as HTMLButtonElement;
    const searchLocationButton = document.getElementById("location-button") as HTMLButtonElement;
    const goToRowInput = document.getElementById("goto-row-input") as HTMLInputElement;
    const menuBar = document.getElementById("menu-bar") as HTMLDivElement;
    const fileMenuButton = document.getElementById("file-menu-button") as HTMLButtonElement;
    const helpMenuButton = document.getElementById("help-menu-button") as HTMLButtonElement;
    const languageMenuButton = document.getElementById("language-menu-button") as HTMLButtonElement;
    const fileMenu = document.getElementById("file-menu") as HTMLDivElement;
    const helpMenu = document.getElementById("help-menu") as HTMLDivElement;
    const languageMenu = document.getElementById("language-menu") as HTMLDivElement;
    const currentState = document.getElementById("current-state") as HTMLDivElement;
    const themeWindow = document.getElementById("theme-window") as HTMLDivElement;
    const createThemeMenuButton = document.getElementById("create-theme-menu-button") as HTMLButtonElement;
    const searchMenu = document.getElementById("search-menu") as HTMLDivElement;
    const searchButton = document.getElementById("search-button") as HTMLButtonElement;

    const replaced = new Map<string, Record<string, string>>();
    const activeGhostLines: HTMLDivElement[] = [];

    let settings: Settings | null = (await exists(settingsPath, { dir: Resource }))
        ? JSON.parse(await readTextFile(settingsPath, { dir: Resource }))
        : null;

    let language: Language = await determineLanguage();
    let windowLocalization: MainWindowLocalization;

    const themes = JSON.parse(await readTextFile(themesPath, { dir: Resource })) as ThemeObject;

    let theme: Theme = settings?.theme ? themes[settings.theme] : themes["cool-zinc"];
    let currentTheme: null | string = null;

    await changeLanguage(language);

    if (!settings) {
        settings = (await createSettings()) as Settings;
    }

    const { enabled: backupEnabled, period: backupPeriod, max: backupMax }: BackupSetting = settings.backup;

    await setTheme(theme);
    if (settings.firstLaunch) await initializeFirstLaunch();
    await initializeProject(settings.project);

    initializeLanguage();
    initializeThemes();

    if (projectPath) {
        await createLogFile();
        await createCompileSettings();
    }

    let searchRegex = false;
    let searchWhole = false;
    let searchCase = false;
    let searchTranslation = false;
    let searchLocation = false;

    let state: State | null = null;
    let statePrevious: State | null = null;

    let saved = true;
    let saving = false;
    let currentFocusedElement: [string, string] | [] = [];

    let shiftPressed = false;

    let selectedMultiple = false;
    const selectedTextareas = new Map<string, string>();
    const replacedTextareas = new Map<string, string>();

    leftPanel.style.height = `${window.innerHeight - topPanel.clientHeight - menuBar.clientHeight}px`;

    let nextBackupNumber: number;

    const observerMain = new IntersectionObserver(
        (entries) => {
            for (const entry of entries) {
                entry.target.firstElementChild?.classList.toggle("hidden", !entry.isIntersecting);
            }
        },
        {
            root: document,
            rootMargin: "384px",
            threshold: 0,
        },
    );

    const observerFound = new IntersectionObserver(
        (entries) => {
            for (const entry of entries) {
                entry.target.firstElementChild?.classList.toggle("hidden", !entry.isIntersecting);
            }
        },
        { root: searchPanelFound, threshold: 0 },
    );

    const observerReplaced = new IntersectionObserver(
        (entries) => {
            for (const entry of entries) {
                entry.target.firstElementChild?.classList.toggle("hidden", !entry.isIntersecting);
            }
        },
        { root: searchPanelReplaced, threshold: 0 },
    );

    async function determineLanguage(): Promise<Language> {
        let locale: string | null = await getLocale();

        if (settings && settings.language) {
            locale = settings.language;
        }

        if (!locale) {
            return Language.English;
        }

        switch (locale) {
            case "ru":
            case "uk":
            case "be":
                return Language.Russian;
            default:
                return Language.English;
        }
    }

    function getThemeStyleSheet(): CSSStyleSheet | undefined {
        for (const styleSheet of document.styleSheets) {
            for (const rule of styleSheet.cssRules) {
                if (rule.selectorText === ".backgroundDark") {
                    return styleSheet;
                }
            }
        }
    }

    function handleMousedown(event: MouseEvent): void {
        if (event.button === 0) {
            if (shiftPressed) {
                if (
                    contentContainer.contains(document.activeElement) &&
                    document.activeElement?.tagName === "TEXTAREA"
                ) {
                    event.preventDefault();
                    selectedTextareas.clear();

                    selectedMultiple = true;
                    const target: HTMLTextAreaElement = event.target as HTMLTextAreaElement;

                    const targetId: string[] = target.id.split("-");
                    const targetRow: number = Number.parseInt(targetId.pop() as string);

                    const focusedElementId: string[] = document.activeElement.id.split("-");
                    const focusedElementRow: number = Number.parseInt(focusedElementId.pop() as string);

                    const rowsRange: number = targetRow - focusedElementRow;
                    const rowsToSelect: number = Math.abs(rowsRange);

                    if (rowsRange > 0) {
                        for (let i = 0; i < rowsToSelect + 1; i++) {
                            const line = focusedElementRow + i;

                            const nextElement: HTMLTextAreaElement = document.getElementById(
                                `${targetId.join("-")}-${line}`,
                            ) as HTMLTextAreaElement;

                            nextElement.style.outlineColor = theme.outlineFocused;
                            selectedTextareas.set(nextElement.id, nextElement.value);
                        }
                    } else {
                        for (let i = rowsToSelect; i >= 0; i--) {
                            const line = focusedElementRow - i;

                            const nextElement: HTMLTextAreaElement = document.getElementById(
                                `${targetId.join("-")}-${line}`,
                            ) as HTMLTextAreaElement;

                            nextElement.style.outlineColor = theme.outlineFocused;
                            selectedTextareas.set(nextElement.id, nextElement.value);
                        }
                    }
                }
            } else {
                selectedMultiple = false;

                for (const id of selectedTextareas.keys()) {
                    const element = document.getElementById(id) as HTMLTextAreaElement;
                    element.style.outlineColor = "";
                }
            }
        }
    }

    function showThemeWindow(): void {
        themeWindow.classList.remove("hidden");

        function changeStyle(inputElement: HTMLInputElement) {
            const id = inputElement.id;
            const value = inputElement.value;

            for (const rule of sheet.cssRules) {
                if (id.endsWith("Focused") && rule.selectorText === `.${id}:focus`) {
                    rule.style.setProperty(rule.style[0], value);
                } else if (id.endsWith("Hovered") && rule.selectorText === `.${id}:hover`) {
                    rule.style.setProperty(rule.style[0], value);
                } else if (rule.selectorText === `.${id}`) {
                    rule.style.setProperty(rule.style[0], value);
                }
            }
        }

        async function createTheme(): Promise<void> {
            const themeNameInput = themeWindow.lastElementChild?.firstElementChild
                ?.lastElementChild as HTMLInputElement;
            const themeName = themeNameInput.value;

            const newTheme = { name: themeName } as Theme;

            for (const div of themeWindow.children[1].children as HTMLCollectionOf<HTMLDivElement>) {
                for (const subdiv of div.children) {
                    const inputElement = subdiv.firstElementChild as HTMLInputElement;
                    newTheme[inputElement.id] = inputElement.value;
                }
            }

            if (!/^[a-zA-Z0-9_-]+$/.test(themeName)) {
                await message(
                    `${windowLocalization.invalidThemeName} ${windowLocalization.allowedThemeNameCharacters}`,
                );
                return;
            }

            themes[themeName] = newTheme;

            await writeTextFile(themesPath, JSON.stringify(themes), { dir: Resource });

            const newThemeButton = document.createElement("button");
            newThemeButton.id = themeName;
            newThemeButton.textContent = themeName;

            themeMenu.insertBefore(themeMenu.lastElementChild as HTMLElement, newThemeButton);
        }

        requestAnimationFrame(() => {
            themeWindow.style.left = `${(document.body.clientWidth - themeWindow.clientWidth) / 2}px`;

            let i = 1;
            const themeColors = Object.values(theme);

            for (const div of themeWindow.children[1].children as HTMLCollectionOf<HTMLDivElement>) {
                for (const subdiv of div.children) {
                    const inputElement = subdiv.firstElementChild as HTMLInputElement;

                    inputElement.value = themeColors[i];
                    inputElement.addEventListener("input", () => changeStyle(inputElement));
                    i++;
                }
            }

            const closeButton = themeWindow.firstElementChild?.firstElementChild as HTMLButtonElement;
            const createThemeButton = themeWindow.lastElementChild?.lastElementChild as HTMLButtonElement;
            createThemeButton.addEventListener("click", async () => await createTheme());

            closeButton.addEventListener("click", () => {
                for (const div of themeWindow.children[1].children as HTMLCollectionOf<HTMLDivElement>) {
                    for (const subdiv of div.children) {
                        const inputElement = subdiv.firstElementChild as HTMLInputElement;
                        inputElement.removeEventListener("input", () => changeStyle(inputElement));
                    }
                }

                createThemeButton.removeEventListener("click", async () => await createTheme());
                themeWindow.classList.add("hidden");
            });
        });
    }

    async function ensureProjectIsValid(projectDir: string): Promise<boolean> {
        async function ensureTranslationSubdirsExist(): Promise<boolean> {
            for (const dir of [mapsDir, otherDir]) {
                if (!(await exists(await join(translationPath, dir)))) {
                    await message(windowLocalization.missingTranslationSubdirs);
                    return false;
                }
            }

            return true;
        }

        if (!(await exists(projectDir))) {
            await message(windowLocalization.selectedFolderMissing);
            return false;
        }

        const noProjectSelected = document.getElementById("no-project-selected") as HTMLDivElement;
        noProjectSelected.innerHTML = windowLocalization.loadingProject;

        const translationPath = await join(projectDir, translationDir);
        const parsed = (await exists(translationPath)) ? true : false;

        let originalDir = await join(projectDir, "data");

        if (!(await exists(originalDir))) {
            originalDir = await join(projectDir, "Data");

            if (!(await exists(originalDir))) {
                originalDir = await join(projectDir, "original");

                if (!(await exists(originalDir))) {
                    await message(windowLocalization.missingOriginalDir);
                    return false;
                }
            }
        }

        const currentGameEngine = document.getElementById("current-game-engine") as HTMLDivElement;
        const currentGameTitle = document.getElementById("current-game-title") as HTMLInputElement;

        if (await exists(await join(originalDir, "System.rxdata"))) {
            gameEngineType = EngineType.XP;
            currentGameEngine.innerHTML = "XP";
        } else if (await exists(await join(originalDir, "System.rvdata"))) {
            gameEngineType = EngineType.VX;
            currentGameEngine.innerHTML = "VX";
        } else if (await exists(await join(originalDir, "System.rvdata2"))) {
            gameEngineType = EngineType.VXAce;
            currentGameEngine.innerHTML = "VX Ace";
        } else if (await exists(await join(originalDir, "System.json"))) {
            gameEngineType = EngineType.New;
            currentGameEngine.innerHTML = "MV / MZ";

            const systemOriginalText = await readTextFile(
                await join(projectDir, translationDir, otherDir, "system.txt"),
            );
            const originalGameTitle = systemOriginalText.slice(systemOriginalText.lastIndexOf("\n"));
            const systemTranslatedText = await readTextFile(
                await join(projectDir, translationDir, otherDir, "system_trans.txt"),
            );
            const translatedGameTitle = systemTranslatedText.slice(systemTranslatedText.lastIndexOf("\n"));

            if (originalGameTitle !== "") {
                currentGameTitle.value = originalGameTitle;
            } else {
                currentGameTitle.value = translatedGameTitle;
            }
        } else {
            await message(windowLocalization.cannotDetermineEngine);
            return false;
        }

        if (!parsed) {
            if (gameEngineType === EngineType.New) {
                await invoke<string>("read", {
                    projectDir: projectDir,
                    gameTitle: currentGameTitle.value,
                    logging: false,
                    disableCustomParsing: false,
                    processingType: ProcessingType.Default,
                });
            } else {
                const platform = await getPlatform();

                platform === "win32"
                    ? await new Command("rvpacker-txt-win", ["read", "--input-dir", projectDir]).execute()
                    : await new Command("rvpacker-txt-linux", ["read", "--input-dir", projectDir]).execute();
            }

            return true;
        } else {
            return await ensureTranslationSubdirsExist();
        }
    }

    async function openDirectory(): Promise<void> {
        const folder = await openPath({ directory: true, multiple: false });

        if (folder) {
            const projectIsValid = await ensureProjectIsValid(folder as string);

            if (!projectIsValid) {
                const noProjectSelected = document.getElementById("no-project-selected") as HTMLDivElement;
                noProjectSelected.innerHTML = windowLocalization.noProjectSelected;
                return;
            }

            projectPath = folder as string;

            contentContainer.innerHTML = "";

            settings!.project = projectPath;
            await writeTextFile(settingsPath, JSON.stringify({ ...settings, project: projectPath }), {
                dir: Resource,
            });

            await createDir(await join(projectPath, backupDir), { recursive: true });
            nextBackupNumber = Number.parseInt(await determineLastBackupNumber());
            if (backupEnabled) {
                backup(backupPeriod);
            }

            await createContent();
        }
    }

    async function createSettings(): Promise<Settings | undefined> {
        await message(windowLocalization.cannotGetSettings);
        const askCreateSettings: boolean = await ask(windowLocalization.askCreateSettings);

        if (askCreateSettings) {
            await writeTextFile(
                settingsPath,
                JSON.stringify({
                    backup: { enabled: true, period: 60, max: 99 },
                    language: language,
                    theme: "cool-zinc",
                    firstLaunch: true,
                    project: null,
                }),
                {
                    dir: Resource,
                },
            );

            alert(windowLocalization.createdSettings);
            return JSON.parse(await readTextFile(settingsPath, { dir: Resource }));
        } else {
            await exit();
        }
    }

    async function changeLanguage(lang: Language): Promise<void> {
        language = lang;
        windowLocalization = new MainWindowLocalization(language);

        initializeLanguage();
    }

    async function determineLastBackupNumber(): Promise<string> {
        const backups = await readDir(await join(projectPath, backupDir));
        return backups.length === 0
            ? "00"
            : backups
                  .map((backup) => Number.parseInt((backup.name as string).slice(-2)))
                  .sort((a, b) => b - a)[0]
                  .toString();
    }

    async function createRegExp(text: string): Promise<RegExp | undefined> {
        text = text.trim();
        if (!text) {
            return;
        }

        let regexp = searchRegex
            ? text
            : await invoke<string>("escape_text", {
                  text: text,
              });

        //fuck boundaries, they aren't working with symbols other than from ascii
        regexp = searchWhole ? `(?<!\\p{L})${regexp}(?!\\p{L})` : `${regexp}`;

        const attr: string = searchCase ? "g" : "gi";

        try {
            return XRegExp(regexp, attr);
        } catch (err) {
            await message(`${windowLocalization.invalidRegexp} (${text}), ${err}`);
            return;
        }
    }

    function appendMatch(element: HTMLDivElement, result: string): void {
        const resultContainer: HTMLDivElement = document.createElement("div");

        const resultElement: HTMLDivElement = document.createElement("div");
        resultElement.classList.add("search-result", "textSecond", "borderPrimary", "backgroundSecond");

        const thirdParent: HTMLDivElement = element.parentElement?.parentElement?.parentElement as HTMLDivElement;

        const [counterpartElement, sourceIndex]: [HTMLElement, number] = findCounterpart(element.id);
        const [source, row]: [string, string] = extractInfo(element);

        const mainDiv: HTMLDivElement = document.createElement("div");
        mainDiv.classList.add("text-base");

        const resultDiv: HTMLDivElement = document.createElement("div");
        resultDiv.innerHTML = result;
        mainDiv.appendChild(resultDiv);

        const originalInfo: HTMLDivElement = document.createElement("div");
        originalInfo.classList.add("text-xs", "textThird");

        const secondParent = element.parentElement?.parentElement as HTMLElement;
        const currentFile: string = secondParent.id.slice(0, secondParent.id.lastIndexOf("-")) as string;
        originalInfo.innerHTML = `${currentFile} - ${source} - ${row}`;
        mainDiv.appendChild(originalInfo);

        const arrow: HTMLDivElement = document.createElement("div");
        arrow.classList.add("search-result-arrow", "textSecond");
        arrow.innerHTML = "arrow_downward";
        mainDiv.appendChild(arrow);

        const counterpart: HTMLDivElement = document.createElement("div");
        counterpart.innerHTML =
            counterpartElement.tagName === "TEXTAREA"
                ? (counterpartElement as HTMLTextAreaElement).value.replaceAllMultiple({ "<": "&lt;", ">": "&gt;" })
                : counterpartElement.innerHTML.replaceAllMultiple({ "<": "&lt;", ">": "&gt;" });
        mainDiv.appendChild(counterpart);

        const counterpartInfo: HTMLDivElement = document.createElement("div");
        counterpartInfo.classList.add("text-xs", "textThird");

        counterpartInfo.innerHTML = `${currentFile} - ${sourceIndex === 0 ? "original" : "translation"} - ${row}`;
        mainDiv.appendChild(counterpartInfo);

        resultElement.appendChild(mainDiv);

        resultElement.setAttribute("data", `${thirdParent.id},${element.id},${sourceIndex}`);
        resultContainer.appendChild(resultElement);
        searchPanelFound.appendChild(resultContainer);
    }

    function createMatchesContainer(elementText: string, matches: string[]): string {
        const result: string[] = [];
        let lastIndex = 0;

        for (const match of matches) {
            const start = elementText.indexOf(match, lastIndex);
            const end = start + match.length;

            const beforeDiv = `<span>${elementText.slice(lastIndex, start)}</span>`;
            const matchDiv = `<span class="backgroundThird">${match}</span>`;

            result.push(beforeDiv);
            result.push(matchDiv);

            lastIndex = end;
        }

        const afterDiv = `<span>${elementText.slice(lastIndex)}</span>`;
        result.push(afterDiv);

        return result.join("");
    }

    async function searchText(
        text: string,
        replace: boolean,
    ): Promise<null | undefined | Map<HTMLTextAreaElement, string>> {
        const regexp: RegExp | undefined = await createRegExp(text);
        if (!regexp) {
            return;
        }

        for (const file of await readDir(projectPath)) {
            if (file.name?.startsWith("matches")) {
                await removeFile(file.path);
            }
        }

        let results: null | Map<HTMLTextAreaElement, string> = new Map();
        let objectToWrite: Record<string, string> = {};
        let count = 1;
        let file = 0;

        const searchArray: HTMLElement[] = (
            searchLocation && state
                ? [...(document.getElementById(state)?.children as HTMLCollectionOf<HTMLElement>)]
                : [...(contentContainer.children as HTMLCollectionOf<HTMLElement>)].flatMap((parent: HTMLElement) => [
                      ...parent.children,
                  ])
        ) as HTMLElement[];

        for (const child of searchArray) {
            const node = child.firstElementChild?.children as HTMLCollectionOf<HTMLTextAreaElement>;

            {
                const elementText = node[2].value.replaceAllMultiple({
                    "<": "&lt;",
                    ">": "&gt;",
                });
                const matches = elementText.match(regexp) as RegExpMatchArray;

                if (matches) {
                    const result = createMatchesContainer(elementText, matches);

                    if (replace) {
                        results?.set(node[2], result);
                    } else {
                        objectToWrite[node[2].id] = result;
                        results = null;
                    }

                    count++;
                }
            }

            if (!searchTranslation) {
                const elementText = node[1].innerHTML.replaceAllMultiple({ "<": "&lt;", ">": "&gt;" });
                const matches = elementText.match(regexp) as RegExpMatchArray;

                if (matches) {
                    const result = createMatchesContainer(elementText, matches);

                    if (replace) {
                        results?.set(node[1], result);
                    } else {
                        objectToWrite[node[1].id] = result;
                        results = null;
                    }

                    count++;
                }
            }

            if (count % 1000 === 0) {
                await writeTextFile(await join(projectPath, `matches-${file}.json`), JSON.stringify(objectToWrite));

                objectToWrite = {};
                file++;
            }
        }

        if (file === 0) {
            await writeTextFile(await join(projectPath, "matches-0.json"), JSON.stringify(objectToWrite));
        }

        searchTotalPages.textContent = file.toString();
        searchCurrentPage.textContent = "0";

        for (const [id, result] of Object.entries(
            JSON.parse(await readTextFile(await join(projectPath, "matches-0.json"))),
        )) {
            appendMatch(document.getElementById(id) as HTMLDivElement, result as string);
        }

        return results;
    }

    async function handleReplacedClick(event: MouseEvent): Promise<void> {
        const target = event.target as HTMLElement;

        const element = target.classList.contains("replaced-element") ? target : (target.parentElement as HTMLElement);

        if (element.hasAttribute("reverted") || !searchPanelReplaced.contains(element)) {
            return;
        }

        const clicked = document.getElementById(
            element.firstElementChild?.textContent as string,
        ) as HTMLTextAreaElement;
        const secondParent = element.parentElement?.parentElement as HTMLElement;

        if (event.button === 0) {
            changeState(secondParent.parentElement?.id as State);

            secondParent.scrollIntoView({
                block: "center",
                inline: "center",
            });
        } else if (event.button === 2) {
            clicked.value = element.children[1].textContent as string;

            element.innerHTML = `<span class="text-base"><code>${element.firstElementChild?.textContent}</code>\n${
                windowLocalization.textReverted
            }\n<code>${element.children[1].textContent}</code></span>`;
            element.setAttribute("reverted", "");

            const replacementLogContent: Record<string, { original: string; translation: string }> = JSON.parse(
                await readTextFile(await join(projectPath, logFile)),
            );

            delete replacementLogContent[clicked.id];

            await writeTextFile(await join(projectPath, logFile), JSON.stringify(replacementLogContent));
        }
    }

    function showSearchPanel(hide = true): void {
        if (searchPanel.getAttribute("moving") === "false") {
            hide
                ? searchPanel.toggleMultiple("translate-x-0", "translate-x-full")
                : searchPanel.classList.replace("translate-x-full", "translate-x-0");

            searchPanel.setAttribute("moving", "true");
        }

        let loadingContainer: HTMLDivElement;

        if (searchPanelFound.children.length > 0 && searchPanelFound.firstElementChild?.id !== "no-results") {
            loadingContainer = document.createElement("div");
            loadingContainer.classList.add("flex", "justify-center", "items-center", "h-full", "w-full");
            loadingContainer.innerHTML = searchPanel.classList.contains("translate-x-0")
                ? `<div class="text-4xl animate-spin font-material">refresh</div>`
                : "";

            searchPanelFound.appendChild(loadingContainer);
        }

        if (searchPanel.getAttribute("shown") === "false") {
            searchPanel.addEventListener(
                "transitionend",
                () => {
                    if (loadingContainer) {
                        searchPanelFound.removeChild(loadingContainer);
                    }

                    searchPanel.setAttribute("shown", "true");
                    searchPanel.setAttribute("moving", "false");
                },
                { once: true },
            );
        } else {
            if (searchPanel.classList.contains("translate-x-full")) {
                searchPanel.setAttribute("shown", "false");
                searchPanel.setAttribute("moving", "true");

                searchPanel.addEventListener("transitionend", () => searchPanel.setAttribute("moving", "false"), {
                    once: true,
                });
                return;
            }

            if (loadingContainer! !== undefined) {
                searchPanelFound.removeChild(loadingContainer);
            }

            searchPanel.setAttribute("moving", "false");
        }
    }

    function findCounterpart(id: string): [HTMLElement, number] {
        if (id.includes("original")) {
            return [document.getElementById(id.replace("original", "translation")) as HTMLElement, 1];
        } else {
            return [document.getElementById(id.replace("translation", "original")) as HTMLElement, 0];
        }
    }

    function extractInfo(element: HTMLElement): [string, string] {
        const parts = element.id.split("-");
        const source = parts[1];
        const row = parts[2];
        return [source, row];
    }

    async function handleResultClick(
        button: number,
        currentState: HTMLElement,
        element: HTMLElement,
        resultElement: HTMLDivElement,
        counterpartIndex: number,
    ): Promise<void> {
        if (button === 0) {
            changeState(currentState.id as State);

            element.parentElement?.parentElement?.scrollIntoView({
                block: "center",
                inline: "center",
            });
        } else if (button === 2) {
            if (element.id.includes("original")) {
                alert(windowLocalization.originalTextIrreplacable);
                return;
            } else {
                if (replaceInput.value.trim()) {
                    const newText = await replaceText(element as HTMLTextAreaElement, false);

                    if (newText) {
                        saved = false;
                        const index = counterpartIndex === 1 ? 3 : 0;
                        resultElement.children[index].innerHTML = newText;
                    }
                }
            }
        }
    }

    async function handleResultSelecting(event: MouseEvent): Promise<void> {
        const target: HTMLDivElement = event.target as HTMLDivElement;

        const resultElement: HTMLDivElement = (
            target.parentElement?.hasAttribute("data")
                ? target.parentElement
                : target.parentElement?.parentElement?.hasAttribute("data")
                  ? target.parentElement?.parentElement
                  : target.parentElement?.parentElement?.parentElement
        ) as HTMLDivElement;

        if (!searchPanelFound.contains(resultElement)) {
            return;
        }

        const [thirdParent, element, counterpartIndex] = (resultElement.getAttribute("data") as string).split(",");

        await handleResultClick(
            event.button,
            document.getElementById(thirdParent) as HTMLElement,
            document.getElementById(element) as HTMLElement,
            resultElement,
            Number.parseInt(counterpartIndex),
        );
    }

    async function displaySearchResults(text: string | null = null, hide = true): Promise<void> {
        if (!text) {
            showSearchPanel(hide);
            return;
        }

        text = text.trim();
        if (!text) {
            return;
        }

        const noMatches: null | undefined | Map<HTMLTextAreaElement, string> = await searchText(text, false);

        if (noMatches) {
            searchPanelFound.innerHTML = `<div id="no-results" class="flex justify-center items-center h-full">${windowLocalization.noMatches}</div>`;
            showSearchPanel(false);
            return;
        }

        observerFound.disconnect();
        searchPanelFound.style.height = `${searchPanelFound.scrollHeight}px`;

        for (const container of searchPanelFound.children as HTMLCollectionOf<HTMLDivElement>) {
            container.style.width = `${container.clientWidth}px`;
            container.style.height = `${container.clientHeight}px`;

            observerFound.observe(container);
        }

        for (const container of searchPanelFound.children as HTMLCollectionOf<HTMLDivElement>) {
            container.firstElementChild?.classList.add("hidden");
        }

        showSearchPanel(hide);

        searchPanelFound.removeEventListener("mousedown", async (event) => await handleResultSelecting(event));
        searchPanelFound.addEventListener("mousedown", async (event) => await handleResultSelecting(event));
    }

    async function replaceText(text: string | HTMLTextAreaElement, replaceAll: boolean): Promise<string | undefined> {
        if (!replaceAll && text instanceof HTMLTextAreaElement) {
            const regexp: RegExp | undefined = await createRegExp(searchInput.value);
            if (!regexp) {
                return;
            }

            const replacerText: string = replaceInput.value;

            const highlightedReplacement: HTMLSpanElement = document.createElement("span");
            highlightedReplacement.classList.add("bg-red-600");
            highlightedReplacement.textContent = replacerText;

            const newText: string[] = text.value.split(regexp);
            const newTextParts: (string | HTMLSpanElement)[] = newText.flatMap((part: string, i: number) => [
                part,
                i < newText.length - 1 ? highlightedReplacement : "",
            ]);

            const newValue: string = newText.join(replacerText);

            replaced.set(text.id, { original: text.value, translation: newValue });
            const prevFile: Record<string, Record<string, string>> = JSON.parse(
                await readTextFile(await join(projectPath, logFile)),
            );

            const newObject: Record<string, Record<string, string>> = {
                ...prevFile,
                ...Object.fromEntries([...replaced]),
            };

            await writeTextFile(await join(projectPath, logFile), JSON.stringify(newObject));
            replaced.clear();

            text.value = newValue;
            return newTextParts.join("");
        }

        text = (text as string).trim();
        if (!text) {
            return;
        }

        const results: null | undefined | Map<HTMLTextAreaElement, string> = await searchText(text, true);
        if (!results) {
            return;
        }

        const regexp: RegExp | undefined = await createRegExp(text);
        if (!regexp) {
            return;
        }

        for (const textarea of results.keys()) {
            if (!textarea.id.includes("original")) {
                const newValue = textarea.value.replace(regexp, replaceInput.value);

                replaced.set(textarea.id, {
                    original: textarea.value,
                    translation: newValue,
                });

                textarea.value = newValue;
            }
        }

        const prevFile: Record<string, Record<string, string>> = JSON.parse(
            await readTextFile(await join(projectPath, logFile)),
        );

        const newObject: Record<string, Record<string, string>> = {
            ...prevFile,
            ...Object.fromEntries([...replaced]),
        };

        await writeTextFile(await join(projectPath, logFile), JSON.stringify(newObject));
        replaced.clear();
    }

    async function save(backup = false): Promise<void> {
        if (saving || !projectPath) {
            return;
        }

        saving = true;
        saveButton.firstElementChild?.classList.add("animate-spin");

        let dirName: string = await join(projectPath, translationDir);

        if (backup) {
            const date = new Date();
            const formattedDate: string = [
                date.getFullYear(),
                (date.getMonth() + 1).toString().padStart(2, "0"),
                date.getDate().toString().padStart(2, "0"),
                date.getHours().toString().padStart(2, "0"),
                date.getMinutes().toString().padStart(2, "0"),
                date.getSeconds().toString().padStart(2, "0"),
            ].join("-");

            nextBackupNumber = (nextBackupNumber % backupMax) + 1;

            dirName = await join(
                projectPath,
                backupDir,
                `${formattedDate}_${nextBackupNumber.toString().padStart(2, "0")}`,
            );

            for (const subDir of [mapsDir, otherDir, pluginsDir]) {
                await createDir(await join(dirName, subDir), { recursive: true });
            }
        }

        let i = 0;
        for (const contentElement of contentContainer.children) {
            const outputArray: string[] = [];

            for (const child of contentElement.children) {
                const node: HTMLTextAreaElement = child.firstElementChild?.children[2] as HTMLTextAreaElement;
                outputArray.push(node.value.replaceAll("\n", "\\#"));
            }

            if (contentElement.id === "system") {
                outputArray.push((document.getElementById("current-game-title") as HTMLInputElement).value);
            }

            const dirPath =
                gameEngineType === EngineType.New
                    ? i < 2
                        ? mapsDir
                        : i < 12
                          ? otherDir
                          : pluginsDir
                    : i < 2
                      ? mapsDir
                      : otherDir;

            const filePath = `${dirPath}/${contentElement.id}_trans.txt`;
            await writeTextFile(await join(dirName, filePath), outputArray.join("\n"));

            i++;
        }

        if (!backup) {
            saved = true;
        }

        saveButton.firstElementChild?.classList.remove("animate-spin");
        saving = false;
    }

    function backup(seconds: number): void {
        if (!backupEnabled) {
            return;
        }

        setTimeout(async () => {
            if (backupEnabled) {
                await save(true);
                backup(seconds);
            }
        }, seconds * 1000);
    }

    function updateState(newState: string, slide = true): void {
        const contentParent = document.getElementById(newState) as HTMLDivElement;

        if (!contentParent) {
            alert(windowLocalization.missingFileText);
            return;
        }

        currentState.innerHTML = newState;

        contentParent.classList.replace("hidden", "flex");

        if (statePrevious) {
            const previousStateContainer = document.getElementById(statePrevious) as HTMLDivElement;

            if (previousStateContainer) {
                previousStateContainer.toggleMultiple("flex", "hidden");
            }

            observerMain.disconnect();
        }

        for (const child of contentParent.children) {
            observerMain.observe(child);
        }

        if (slide) {
            leftPanel.toggleMultiple("-translate-x-full", "translate-x-0");
        }
    }

    function changeState(newState: State | null, slide = false): void {
        if (state === newState) {
            return;
        }

        if (newState === null) {
            state = null;
            currentState.innerHTML = "";

            observerMain.disconnect();
            for (const child of contentContainer.children) {
                child.classList.replace("flex", "hidden");
            }
        } else {
            statePrevious = state;
            state = newState;
            updateState(newState, slide);
        }
    }

    function goToRow(): void {
        goToRowInput.classList.remove("hidden");
        goToRowInput.focus();

        const element = document.getElementById(state as string) as HTMLDivElement;
        const lastRow = element.lastElementChild?.id.split("-").at(-1) as string;

        goToRowInput.placeholder = `${windowLocalization.goToRow} ${lastRow}`;
    }

    function jumpToRow(key: string): void {
        const focusedElement = document.activeElement as HTMLElement;
        if (!contentContainer.contains(focusedElement) && focusedElement.tagName !== "TEXTAREA") {
            return;
        }

        const idParts = focusedElement.id.split("-");
        const index = Number.parseInt(idParts.pop() as string);
        const baseId = idParts.join("-");

        if (isNaN(index)) {
            return;
        }

        const step = key === "alt" ? 1 : -1;
        const nextIndex = index + step;
        const nextElementId = `${baseId}-${nextIndex}`;
        const nextElement = document.getElementById(nextElementId) as HTMLTextAreaElement;

        if (!nextElement) {
            return;
        }

        const scrollOffset = nextElement.clientHeight + 8;
        window.scrollBy(0, step * scrollOffset);
        focusedElement.blur();
        nextElement.focus();
        nextElement.setSelectionRange(0, 0);
    }

    async function handleKeypress(event: KeyboardEvent): Promise<void> {
        if (!projectPath) {
            return;
        }

        if (event.key === "Tab") {
            event.preventDefault();
        }

        if (document.activeElement === document.body) {
            switch (event.code) {
                case "Escape":
                    changeState(null);
                    break;
                case "Tab":
                    leftPanel.toggleMultiple("translate-x-0", "-translate-x-full");
                    break;
                case "KeyR":
                    await displaySearchResults();
                    break;
                case "KeyZ":
                    if (event.ctrlKey) {
                        event.preventDefault();

                        for (const key of selectedTextareas.keys()) {
                            const textarea = document.getElementById(key) as HTMLTextAreaElement;
                            textarea.value = selectedTextareas.get(key) as string;
                        }

                        for (const key of replacedTextareas.keys()) {
                            const textarea = document.getElementById(key) as HTMLTextAreaElement;
                            textarea.value = replacedTextareas.get(key) as string;
                            textarea.calculateHeight();
                        }

                        replacedTextareas.clear();
                    }
                    break;
                case "KeyS":
                    if (event.ctrlKey) {
                        await save();
                    }
                    break;
                case "KeyC":
                    if (event.altKey) {
                        await compile(false);
                    }
                    break;
                case "KeyG":
                    if (event.ctrlKey) {
                        event.preventDefault();

                        if (state && goToRowInput.classList.contains("hidden")) {
                            goToRow();
                        } else if (!goToRowInput.classList.contains("hidden")) {
                            goToRowInput.classList.add("hidden");
                        }
                    }
                    break;
                case "KeyF":
                    if (event.ctrlKey) {
                        event.preventDefault();
                        searchInput.focus();
                    }
                    break;
                case "F4":
                    if (event.altKey) {
                        await appWindow.close();
                    }
                    break;
                case "Digit1":
                    changeState(State.Maps);
                    break;
                case "Digit2":
                    changeState(State.Names);
                    break;
                case "Digit3":
                    changeState(State.Actors);
                    break;
                case "Digit4":
                    changeState(State.Armors);
                    break;
                case "Digit5":
                    changeState(State.Classes);
                    break;
                case "Digit6":
                    changeState(State.CommonEvents);
                    break;
                case "Digit7":
                    changeState(State.Enemies);
                    break;
                case "Digit8":
                    changeState(State.Items);
                    break;
                case "Digit9":
                    changeState(State.Skills);
                    break;
                case "Digit0":
                    changeState(State.System);
                    break;
                case "Minus":
                    changeState(State.Troops);
                    break;
                case "Equal":
                    changeState(State.Weapons);
                    break;
            }
        } else {
            switch (event.code) {
                case "Escape":
                    if (document.activeElement) {
                        (document.activeElement as HTMLElement).blur();
                    }
                    break;
                case "Enter":
                    if (event.altKey) {
                        jumpToRow("alt");
                    } else if (event.ctrlKey) {
                        jumpToRow("ctrl");
                    }
                    break;
            }
        }

        if (event.key === "Shift") {
            if (!event.repeat) {
                shiftPressed = true;
            }
        }
    }

    async function handleSearchInputKeypress(event: KeyboardEvent): Promise<void> {
        if (!projectPath) {
            return;
        }

        if (event.code === "Enter") {
            event.preventDefault();

            if (event.ctrlKey) {
                searchInput.value += "\n";
                searchInput.calculateHeight();
            } else {
                if (searchInput.value.trim()) {
                    searchPanelFound.innerHTML = "";
                    await displaySearchResults(searchInput.value, false);
                }
            }
        } else if (event.code === "Backspace") {
            requestAnimationFrame(() => {
                searchInput.calculateHeight();
            });
        } else if (event.altKey) {
            switch (event.code) {
                case "KeyC":
                    switchCase();
                    break;
                case "KeyW":
                    switchWhole();
                    break;
                case "KeyR":
                    switchRegExp();
                    break;
                case "KeyT":
                    switchTranslation();
                    break;
                case "KeyL":
                    switchLocation();
                    break;
            }
        }
    }

    async function handleReplaceInputKeypress(event: KeyboardEvent) {
        if (!projectPath) {
            return;
        }

        if (event.code === "Enter") {
            event.preventDefault();

            if (event.ctrlKey) {
                replaceInput.value += "\n";
                replaceInput.calculateHeight();
            }
        } else if (event.code === "Backspace") {
            requestAnimationFrame(() => {
                replaceInput.calculateHeight();
            });
        }
    }

    async function createContent(): Promise<void> {
        if (!projectPath) {
            return;
        }

        const contentNames: string[] = [];
        const content: string[][] = [];

        for (const entry of await readDir(await join(projectPath, translationDir), {
            recursive: true,
        })) {
            const folder = entry.name as string;

            for (const file of entry.children as FileEntry[]) {
                const name = file.name as string;

                if (!name.endsWith(".txt") || name.includes("plain")) {
                    continue;
                }

                contentNames.push(name.slice(0, -4));
                content.push((await readTextFile(await join(projectPath, translationDir, folder, name))).split("\n"));
            }
        }

        if (contentNames.includes("scripts")) {
            (leftPanel.lastElementChild as HTMLElement).innerHTML = State.Scripts;
        }

        for (let i = 0; i < contentNames.length - 1; i += 2) {
            const contentName = contentNames[i];
            const contentDiv: HTMLDivElement = document.createElement("div");
            contentDiv.id = contentName;
            contentDiv.classList.add("hidden", "flex-col", "h-auto");

            if (contentName === "system") {
                const currentGameTitle = document.getElementById("current-game-title") as HTMLInputElement;

                if (currentGameTitle.value === "") {
                    const originalGameTitle = content[i].pop() as string;
                    const translatedGameTitle = content[i + 1].pop() as string;

                    if (translatedGameTitle === "") {
                        currentGameTitle.value = originalGameTitle;
                    } else {
                        currentGameTitle.value = translatedGameTitle;
                    }
                }
            }

            for (let j = 0; j < content[i].length; j++) {
                const originalText = content[i][j];
                const translationText = content[i + 1][j];

                const textParent: HTMLDivElement = document.createElement("div");
                textParent.id = `${contentName}-${j + 1}`;
                textParent.classList.add("content-parent");

                const textContainer: HTMLDivElement = document.createElement("div");
                textContainer.classList.add("flex", "content-child");

                const originalTextElement: HTMLDivElement = document.createElement("div");
                originalTextElement.id = `${contentName}-original-${j + 1}`;
                originalTextElement.textContent = originalText.replaceAll("\\#", "\n");
                originalTextElement.classList.add("original-text-div", "backgroundPrimary", "outlinePrimary");

                const translationTextElement: HTMLTextAreaElement = document.createElement("textarea");
                const translationTextSplit = translationText.split("\\#");
                translationTextElement.id = `${contentName}-translation-${j + 1}`;
                translationTextElement.rows = translationTextSplit.length;
                translationTextElement.value = translationTextSplit.join("\n");
                translationTextElement.classList.add(
                    "translation-text-input",
                    "outlinePrimary",
                    "backgroundPrimary",
                    "outlineFocused",
                );

                const rowElement: HTMLDivElement = document.createElement("div");
                rowElement.id = `${contentName}-row-${j + 1}`;
                rowElement.textContent = (j + 1).toString();
                rowElement.classList.add("row", "backgroundPrimary");

                textContainer.appendChild(rowElement);
                textContainer.appendChild(originalTextElement);
                textContainer.appendChild(translationTextElement);
                textParent.appendChild(textContainer);
                contentDiv.appendChild(textParent);
            }

            contentContainer.appendChild(contentDiv);
        }

        for (const child of contentContainer.children as HTMLCollectionOf<HTMLDivElement>) {
            child.toggleMultiple("hidden", "flex");

            const heights = new Uint32Array(child.children.length);
            let i = 0;

            for (const node of child.children as HTMLCollectionOf<HTMLDivElement>) {
                heights.set(
                    [((node.firstElementChild as HTMLElement).children[1].innerHTML.count("\n") + 1) * 28 + 8],
                    i,
                );
                i++;
            }

            i = 0;
            for (const node of child.children as HTMLCollectionOf<HTMLDivElement>) {
                node.style.minHeight = `${heights[i] + 8}px`;

                for (const child of node.firstElementChild?.children as HTMLCollectionOf<HTMLDivElement>) {
                    child.style.minHeight = `${heights[i]}px`;
                }

                node.firstElementChild?.classList.add("hidden");
                i++;
            }

            child.style.minHeight = `${child.scrollHeight}px`;
            child.toggleMultiple("hidden", "flex");

            document.body.firstElementChild?.classList.remove("invisible");
        }

        const noProjectSelected = document.getElementById("no-project-selected") as HTMLDivElement;
        if (noProjectSelected) {
            noProjectSelected.innerHTML = "";
        }
    }

    async function compile(silent: boolean): Promise<void> {
        if (!projectPath) {
            return;
        }

        const compileSettings: CompileSettings = JSON.parse(
            await readTextFile(await join(projectPath, "compile-settings.json")),
        );

        if (!compileSettings.initialized || !compileSettings.doNotAskAgain || !silent) {
            const compileWindow = new WebviewWindow("compile", {
                url: "./compile.html",
                title: windowLocalization.compileWindowTitle,
            });

            const compileUnlisten = await compileWindow.once("compile", async () => {
                await compile(true);
            });

            await compileWindow.once("tauri://destroyed", compileUnlisten);
        } else {
            const currentGameTitle = document.getElementById("current-game-title") as HTMLInputElement;
            compileButton.firstElementChild?.classList.add("animate-spin");

            if (gameEngineType === EngineType.New) {
                await appWindow.once("compile-finished", () => {
                    compileButton.firstElementChild?.classList.remove("animate-spin");
                    alert(`${windowLocalization.compileSuccess} ${(performance.now() - startTime) / 1000}`);
                });

                const startTime = performance.now();
                await appWindow.emit("compile", [
                    projectPath,
                    compileSettings.customOutputPath.path,
                    currentGameTitle.value,
                    compileSettings.logging,
                    compileSettings.shuffle.level,
                    Object.values(compileSettings.disableProcessing.of),
                ]);
            } else {
                const startTime = performance.now();
                const platform = await getPlatform();

                console.log(compileSettings.customOutputPath.path);
                if (platform === "win32") {
                    await new Command("rvpacker-txt-win", [
                        "write",
                        "--input-dir",
                        projectPath,
                        "--output-dir",
                        compileSettings.customOutputPath.path,
                        "--shuffle-level",
                        compileSettings.shuffle.level.toString(),
                        compileSettings.disableCustomParsing ? "--disable-custom-parsing" : "",
                        "--disable-processing",
                        Object.keys(compileSettings.disableProcessing.of)
                            // @ts-expect-error object can be indexed by string fuck off
                            .filter((key) => compileSettings.disableProcessing.of[key])
                            .join(","),
                    ]).execute();
                } else {
                    await new Command("rvpacker-txt-linux", [
                        "write",
                        "--input-dir",
                        projectPath,
                        "--output-dir",
                        compileSettings.customOutputPath.path,
                        "--shuffle-level",
                        compileSettings.shuffle.level.toString(),
                        compileSettings.disableCustomParsing ? "--disable-custom-parsing" : "",
                        "--disable-processing",
                        Object.keys(compileSettings.disableProcessing.of)
                            // @ts-expect-error object can be indexed by string fuck off
                            .filter((key) => compileSettings.disableProcessing.of[key])
                            .join(","),
                    ]).execute();
                }

                alert(`${windowLocalization.compileSuccess} ${(performance.now() - startTime) / 1000}`);
                compileButton.firstElementChild?.classList.remove("animate-spin");
            }
        }
    }

    function getNewLinePositions(textarea: HTMLTextAreaElement): { left: number; top: number }[] {
        const positions: { left: number; top: number }[] = [];
        const lines = textarea.value.split("\n");
        const lineHeight = Number.parseFloat(window.getComputedStyle(textarea).lineHeight);

        const offsetTop = textarea.offsetTop;
        const offsetLeft = textarea.offsetLeft;

        const canvas = document.createElement("canvas");
        const context = canvas.getContext("2d") as CanvasRenderingContext2D;
        context.font = '18px "Segoe UI"';

        let top = offsetTop;

        for (let i = 0; i < lines.length - 1; i++) {
            const line = lines[i];
            const textWidth = context.measureText(`${line} `).width;
            const left = offsetLeft + textWidth;

            positions.push({ left, top });
            top += lineHeight;
        }

        return positions;
    }

    function trackFocus(focusedElement: HTMLTextAreaElement): void {
        for (const ghost of activeGhostLines) {
            ghost.remove();
        }

        const result: { left: number; top: number }[] = getNewLinePositions(focusedElement);

        for (const { left, top } of result) {
            const ghostNewLine: HTMLDivElement = document.createElement("div");
            ghostNewLine.classList.add("ghost-new-line", "textThird");
            ghostNewLine.innerHTML = "\\n";
            ghostNewLine.style.left = `${left}px`;
            ghostNewLine.style.top = `${top}px`;

            activeGhostLines.push(ghostNewLine);
            document.body.appendChild(ghostNewLine);
        }
    }

    function handleFocus(event: FocusEvent): void {
        const target = event.target as HTMLTextAreaElement;

        for (const ghost of activeGhostLines) {
            ghost.remove();
        }

        if (
            contentContainer.contains(target) &&
            target.tagName === "TEXTAREA" &&
            target.id !== currentFocusedElement[0]
        ) {
            currentFocusedElement = [target.id, target.value];

            target.addEventListener("keyup", () => target.calculateHeight());
            target.addEventListener("input", () => trackFocus(target));

            trackFocus(target);
        }
    }

    function handleBlur(event: FocusEvent): void {
        const target = event.target as HTMLTextAreaElement;

        for (const ghost of activeGhostLines) {
            ghost.remove();
        }

        if (target.id == currentFocusedElement[0]) {
            if (saved && currentFocusedElement[1] !== target.value) {
                saved = false;
            }

            currentFocusedElement = [];

            if (contentContainer.contains(target) && target.tagName === "TEXTAREA") {
                target.removeEventListener("input", () => trackFocus(target));
                target.removeEventListener("keyup", () => target.calculateHeight());
            }
        }
    }

    function switchCase() {
        searchCase = !searchCase;
        searchCaseButton.classList.toggle("backgroundThird");
    }

    function switchWhole() {
        searchWhole = !searchWhole;
        searchWholeButton.classList.toggle("backgroundThird");
    }

    function switchRegExp() {
        searchRegex = !searchRegex;
        searchRegexButton.classList.toggle("backgroundThird");
    }

    function switchTranslation() {
        searchTranslation = !searchTranslation;
        searchTranslationButton.classList.toggle("backgroundThird");
    }

    function switchLocation() {
        searchLocation = !searchLocation;
        searchLocationButton.classList.toggle("backgroundThird");
    }

    function createOptionsWindow() {
        new WebviewWindow("options", {
            url: "./settings.html",
            title: windowLocalization.optionsButtonTitle,
            width: 800,
            height: 600,
            center: true,
            resizable: false,
        });
    }

    async function exitProgram(): Promise<boolean> {
        let askExitUnsaved: boolean;
        if (saved) {
            askExitUnsaved = true;
        } else {
            askExitUnsaved = await ask(windowLocalization.unsavedChanges);
        }

        let askExit: boolean;
        if (!askExitUnsaved) {
            askExit = await ask(windowLocalization.exit);
        } else {
            if (!saved) {
                await save();
            }
            return true;
        }

        if (!askExit) {
            return false;
        } else {
            return true;
        }
    }

    async function fileMenuClick(target: HTMLElement): Promise<void> {
        fileMenu.classList.replace("flex", "hidden");

        switch (target.id) {
            case "reload-button":
                await awaitSaving();

                if (await exitProgram()) {
                    location.reload();
                }
                break;
        }
    }

    function helpMenuClick(target: HTMLElement) {
        helpMenu.classList.replace("flex", "hidden");

        switch (target.id) {
            case "help-button":
                new WebviewWindow("help", {
                    url: "./help.html",
                    title: windowLocalization.helpButton,
                    width: 640,
                    height: 480,
                    center: true,
                });
                break;
            case "about-button":
                new WebviewWindow("about", {
                    url: "./about.html",
                    title: windowLocalization.aboutButton,
                    width: 640,
                    height: 480,
                    center: true,
                    resizable: false,
                });
                break;
        }
    }

    async function languageMenuClick(target: HTMLElement) {
        languageMenu.classList.replace("flex", "hidden");

        switch (target.id) {
            case "ru-button":
                if (language !== Language.Russian) {
                    await changeLanguage(Language.Russian);
                }
                break;
            case "en-button":
                if (language !== Language.English) {
                    await changeLanguage(Language.English);
                }
                break;
        }
    }

    function handleMenuBarClick(target: HTMLElement): void {
        switch (target.id) {
            case fileMenuButton.id:
                fileMenu.toggleMultiple("hidden", "flex");
                helpMenu.classList.replace("flex", "hidden");
                languageMenu.classList.replace("flex", "hidden");

                fileMenu.style.top = `${fileMenuButton.offsetTop + fileMenuButton.offsetHeight}px`;
                fileMenu.style.left = `${fileMenuButton.offsetLeft}px`;

                fileMenu.addEventListener("click", async (event) => await fileMenuClick(event.target as HTMLElement), {
                    once: true,
                });
                break;
            case helpMenuButton.id:
                helpMenu.toggleMultiple("hidden", "flex");
                fileMenu.classList.replace("flex", "hidden");
                languageMenu.classList.replace("flex", "hidden");

                helpMenu.style.top = `${helpMenuButton.offsetTop + helpMenuButton.offsetHeight}px`;
                helpMenu.style.left = `${helpMenuButton.offsetLeft}px`;

                helpMenu.addEventListener("click", (event) => helpMenuClick(event.target as HTMLElement), {
                    once: true,
                });
                break;
            case languageMenuButton.id:
                languageMenu.toggleMultiple("hidden", "flex");
                helpMenu.classList.replace("flex", "hidden");
                fileMenu.classList.replace("flex", "hidden");

                languageMenu.style.top = `${languageMenuButton.offsetTop + languageMenuButton.offsetHeight}px`;
                languageMenu.style.left = `${languageMenuButton.offsetLeft}px`;

                languageMenu.addEventListener(
                    "click",
                    async (event) => await languageMenuClick(event.target as HTMLElement),
                    {
                        once: true,
                    },
                );
                break;
        }
    }

    async function awaitSaving() {
        if (saving) {
            await new Promise((resolve) => setTimeout(resolve, 2000));
            await awaitSaving();
        }
    }

    async function createCompileSettings() {
        const compileSettingsPath = await join(projectPath, "compile-settings.json");

        if (!(await exists(compileSettingsPath))) {
            await writeTextFile(
                compileSettingsPath,
                JSON.stringify({
                    initialized: false,
                    logging: false,
                    shuffle: { enabled: false, level: 0 },
                    customParsing: false,
                    customOutputPath: { enabled: false, path: "" },
                    disableProcessing: {
                        enabled: false,
                        of: {
                            maps: false,
                            other: false,
                            system: false,
                            plugins: false,
                        },
                    },
                    doNotAskAgain: true,
                }),
            );
        }
    }

    async function createLogFile() {
        const logPath = await join(projectPath, logFile);

        if (!(await exists(logPath))) {
            await writeTextFile(logPath, "{}");
        }
    }

    async function setTheme(newTheme: Theme): Promise<void> {
        if (newTheme.name === currentTheme) {
            return;
        }

        [currentTheme, theme] = [newTheme.name, newTheme];

        for (const [key, value] of Object.entries(newTheme)) {
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

        settings!.theme = newTheme.name;
        await writeTextFile(settingsPath, JSON.stringify({ ...settings, theme: newTheme.name }), {
            dir: Resource,
        });
    }

    function initializeLanguage(): void {
        for (const [key, value] of Object.entries(windowLocalization)) {
            if (key in theme) {
                continue;
            }

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

        for (const div of themeWindow.children[1].children) {
            for (const subdiv of div.children) {
                const label = subdiv.lastElementChild as HTMLLabelElement;
                // @ts-expect-error object can be indexed by string fuck off
                label.innerHTML = windowLocalization[subdiv.firstElementChild?.id];
            }
        }
    }

    async function initializeFirstLaunch() {
        new WebviewWindow("help", {
            url: "./help.html",
            title: windowLocalization.helpButton,
            width: 640,
            height: 480,
            center: true,
            alwaysOnTop: true,
        });

        await writeTextFile(settingsPath, JSON.stringify({ ...settings, firstLaunch: false }), {
            dir: Resource,
        });
    }

    async function initializeProject(project: string | null): Promise<void> {
        if (!project || !(await ensureProjectIsValid(project))) {
            settings!.project = null;
            await writeTextFile(settingsPath, JSON.stringify({ ...settings, project: null }), {
                dir: Resource,
            });

            const noProjectSelected = document.getElementById("no-project-selected") as HTMLDivElement;
            noProjectSelected.innerHTML = windowLocalization.noProjectSelected;
        } else {
            projectPath = project;
            await createContent();
        }
    }

    function initializeThemes() {
        for (const themeName of Object.keys(themes)) {
            const themeButton = document.createElement("button");
            themeButton.id = themeName;
            themeButton.innerHTML = themeName;
            themeButton.classList.add("dropdown-menu-button", "backgroundPrimary", "backgroundPrimaryHovered");

            themeMenu.insertBefore(themeButton, createThemeMenuButton);
        }
    }

    leftPanel.addEventListener("click", (event) => {
        const newState = leftPanel.secondHighestParent(event.target as HTMLElement).textContent as State;
        changeState(newState, true);
    });

    topPanelButtonsDiv.addEventListener("click", async (event) => {
        if (event.target === topPanelButtonsDiv) {
            return;
        }

        const target: HTMLElement = topPanelButtonsDiv.secondHighestParent(event.target as HTMLElement);

        switch (target.id) {
            case "menu-button":
                if (!projectPath) {
                    return;
                }

                leftPanel.toggleMultiple("translate-x-0", "-translate-x-full");
                break;
            case "save-button":
                await save();
                break;
            case "compile-button":
                if (clickTimer == null) {
                    clickTimer = setTimeout(async () => {
                        clickTimer = null;
                        await compile(true);
                    }, 250);
                } else {
                    clearTimeout(clickTimer);
                    clickTimer = null;
                    await compile(false);
                }
                break;
            case "open-button":
                await openDirectory();
                break;
            case "options-button":
                createOptionsWindow();
                break;
            case "theme-button":
                themeMenu.toggleMultiple("hidden", "flex");

                requestAnimationFrame(() => {
                    themeMenu.style.left = `${themeButton.offsetLeft}px`;
                    themeMenu.style.top = `${themeButton.offsetTop + themeButton.clientHeight + 16}px`;

                    themeMenu.addEventListener("click", async (event: MouseEvent) => {
                        const target = event.target as HTMLButtonElement;

                        if (!themeMenu.contains(target)) {
                            return;
                        }

                        if (target.id === "create-theme-menu-button") {
                            showThemeWindow();
                            return;
                        }

                        await setTheme(themes[target.id]);
                    });
                });
                break;
            case "search-button":
                if (!projectPath) {
                    return;
                }

                if (searchMenu.classList.contains("hidden")) {
                    searchMenu.classList.remove("hidden");
                    requestAnimationFrame(() => {
                        searchMenu.style.left = `${searchButton.offsetLeft}px`;
                        searchMenu.style.top = `${searchButton.offsetHeight + 20}px`;
                    });
                } else {
                    searchMenu.classList.add("hidden");
                }

                break;
        }
    });

    searchPanel.addEventListener("click", async (event) => {
        const target = event.target as HTMLElement;

        switch (target.id) {
            case "switch-search-content":
                {
                    searchPanelFound.toggleMultiple("hidden", "flex");
                    searchPanelReplaced.toggleMultiple("hidden", "flex");

                    const searchSwitch: HTMLElement = target;

                    if (searchSwitch.innerHTML.trim() === "search") {
                        searchSwitch.innerHTML = "menu_book";

                        const replacementLogContent: Record<string, { original: string; translation: string }> =
                            JSON.parse(await readTextFile(await join(projectPath, logFile)));

                        for (const [key, value] of Object.entries(replacementLogContent)) {
                            const replacedContainer: HTMLDivElement = document.createElement("div");

                            const replacedElement: HTMLDivElement = document.createElement("div");
                            replacedElement.classList.add(
                                "replaced-element",
                                "textSecond",
                                "borderPrimary",
                                "backgroundSecond",
                            );

                            replacedElement.innerHTML = `<div class="text-base textThird">${key}</div><div class=text-base>${value.original}</div><div class="flex justify-center items-center text-xl textPrimary font-material">arrow_downward</div><div class="text-base">${value.translation}</div>`;

                            replacedContainer.appendChild(replacedElement);
                            searchPanelReplaced.appendChild(replacedContainer);
                        }

                        observerFound.disconnect();
                        searchPanelReplaced.style.height = `${searchPanelReplaced.scrollHeight}px`;

                        for (const container of searchPanelReplaced.children as HTMLCollectionOf<HTMLElement>) {
                            container.style.width = `${container.clientWidth}px`;
                            container.style.height = `${container.clientHeight}px`;

                            observerReplaced.observe(container);
                            container.firstElementChild?.classList.add("hidden");
                        }

                        searchPanelReplaced.addEventListener(
                            "mousedown",
                            async (event) => await handleReplacedClick(event),
                        );
                    } else {
                        searchSwitch.innerHTML = "search";
                        searchPanelReplaced.innerHTML = "";

                        searchPanelReplaced.removeEventListener(
                            "mousedown",
                            async (event) => await handleReplacedClick(event),
                        );
                    }
                }
                break;
            case "previous-page-button": {
                const page = Number.parseInt(searchCurrentPage.textContent as string);

                if (page > 1) {
                    searchCurrentPage.textContent = (page - 1).toString();

                    searchPanelFound.innerHTML = "";

                    for (const [id, result] of Object.entries(
                        JSON.parse(
                            await readTextFile(
                                await join(
                                    projectPath,
                                    `matches-${Number.parseInt(searchCurrentPage.textContent) - 1}.json`,
                                ),
                            ),
                        ),
                    )) {
                        appendMatch(document.getElementById(id) as HTMLDivElement, result as string);
                    }
                }
                break;
            }
            case "next-page-button": {
                const page = Number.parseInt(searchCurrentPage.textContent as string);

                if (page < Number.parseInt(searchTotalPages.textContent as string)) {
                    searchCurrentPage.textContent = (page + 1).toString();
                    searchPanelFound.innerHTML = "";

                    for (const [id, result] of Object.entries(
                        JSON.parse(
                            await readTextFile(
                                await join(
                                    projectPath,
                                    `matches-${Number.parseInt(searchCurrentPage.textContent) + 1}.json`,
                                ),
                            ),
                        ),
                    )) {
                        appendMatch(document.getElementById(id) as HTMLDivElement, result as string);
                    }
                }
                break;
            }
        }
    });

    searchInput.addEventListener("focus", () => {
        searchInput.addEventListener("paste", () => {
            searchInput.calculateHeight();
        });
        searchInput.addEventListener("keydown", async (event) => await handleSearchInputKeypress(event));

        searchInput.addEventListener(
            "blur",
            () => {
                searchInput.value = searchInput.value.trim();
                searchInput.removeEventListener("keydown", async (event) => await handleSearchInputKeypress(event));
                searchInput.removeEventListener("paste", () => {
                    searchInput.calculateHeight();
                });
            },
            { once: true },
        );
    });

    replaceInput.addEventListener("focus", () => {
        replaceInput.addEventListener("keydown", async (event) => await handleReplaceInputKeypress(event));
        replaceInput.addEventListener("paste", () => {
            replaceInput.calculateHeight();
        });

        replaceInput.addEventListener(
            "blur",
            () => {
                replaceInput.value = replaceInput.value.trim();
                replaceInput.removeEventListener("keydown", async (event) => await handleReplaceInputKeypress(event));
                replaceInput.removeEventListener("paste", () => {
                    replaceInput.calculateHeight();
                });
            },
            { once: true },
        );
    });

    menuBar.addEventListener("click", (event) => handleMenuBarClick(event.target as HTMLElement));

    document.addEventListener("keydown", async (event) => await handleKeypress(event));
    document.addEventListener("keyup", (event) => {
        if (event.key === "Shift") shiftPressed = false;
    });

    document.addEventListener("focus", handleFocus, true);
    document.addEventListener("blur", handleBlur, true);

    goToRowInput.addEventListener("keydown", (event) => {
        if (event.code === "Enter") {
            const rowNumber: string = goToRowInput.value;
            const targetRow: HTMLTextAreaElement = document.getElementById(
                `${state}-${rowNumber}`,
            ) as HTMLTextAreaElement;

            if (targetRow) {
                targetRow.scrollIntoView({
                    block: "center",
                    inline: "center",
                });
            }

            goToRowInput.value = "";
            goToRowInput.classList.add("hidden");
        } else if (event.code === "Escape") {
            goToRowInput.value = "";
            goToRowInput.classList.add("hidden");
        }
    });

    searchMenu.addEventListener("click", async (event) => {
        const target = event.target as HTMLElement;

        switch (target.id) {
            case "replace-button":
                if (!projectPath) {
                    return;
                }

                if (searchInput.value.trim() && replaceInput.value.trim()) {
                    await replaceText(searchInput.value, true);
                }
                break;
            case "case-button":
                switchCase();
                break;
            case "whole-button":
                switchWhole();
                break;
            case "regex-button":
                switchRegExp();
                break;
            case "translation-button":
                switchTranslation();
                break;
            case "location-button":
                switchLocation();
                break;
        }
    });

    searchMenu.addEventListener("mousedown", (event) => {
        const target = event.target as HTMLElement;
        if (target.id !== searchMenu.id) {
            return;
        }

        searchMenu.setAttribute("selected", "true");
        searchMenu.setAttribute(
            "offset",
            `${event.clientX - searchMenu.offsetLeft},${event.clientY - searchMenu.offsetTop}`,
        );

        searchMenu.addEventListener(
            "mouseup",
            () => {
                searchMenu.setAttribute("selected", "false");
            },
            { once: true },
        );
    });

    searchMenu.addEventListener("mousemove", (event) => {
        event.preventDefault();

        if (searchMenu.getAttribute("selected") === "true") {
            const offsets: [number, number] = (searchMenu.getAttribute("offset") as string)
                .split(",")
                .map((element) => Number.parseInt(element)) as [number, number];

            const screenWidth = window.innerWidth;
            const screenHeight = window.innerHeight;
            const elementWidth = searchMenu.offsetWidth;
            const elementHeight = searchMenu.offsetHeight;

            const newLeft = Math.max(0, Math.min(event.clientX - offsets[0], screenWidth - elementWidth));
            const newTop = Math.max(0, Math.min(event.clientY - offsets[1], screenHeight - elementHeight));

            searchMenu.style.left = `${newLeft}px`;
            searchMenu.style.top = `${newTop}px`;
        }
    });

    document.addEventListener("mousedown", (event) => handleMousedown(event));

    document.addEventListener(
        "paste",
        (event) => {
            const text = event.clipboardData?.getData("text");

            if (
                contentContainer.contains(document.activeElement) &&
                document.activeElement?.tagName === "TEXTAREA" &&
                text?.includes("\\\\#")
            ) {
                event.preventDefault();
                const clipboardTextSplit = text.split("\\\\#");
                const textRows = clipboardTextSplit.length;

                if (textRows < 1) {
                    return;
                } else {
                    const focusedElement = document.activeElement as HTMLElement;
                    const focusedElementId = focusedElement.id.split("-");
                    const focusedElementNumber = Number.parseInt(focusedElementId.pop() as string);

                    for (let i = 0; i < textRows; i++) {
                        const elementToReplace = document.getElementById(
                            `${focusedElementId.join("-")}-${focusedElementNumber + i}`,
                        ) as HTMLTextAreaElement;

                        replacedTextareas.set(elementToReplace.id, elementToReplace.value.replaceAll(text, ""));
                        elementToReplace.value = clipboardTextSplit[i];
                        elementToReplace.calculateHeight();
                    }

                    saved = false;
                }
            }
        },
        true,
    );

    document.addEventListener(
        "copy",
        (event) => {
            if (
                selectedMultiple &&
                contentContainer.contains(document.activeElement) &&
                document.activeElement?.tagName === "TEXTAREA"
            ) {
                event.preventDefault();
                selectedTextareas.set(document.activeElement.id, (document.activeElement as HTMLTextAreaElement).value);
                event.clipboardData?.setData("text", Array.from(selectedTextareas.values()).join("\\\\#"));
            }
        },
        true,
    );

    document.addEventListener(
        "cut",
        (event) => {
            if (
                selectedMultiple &&
                contentContainer.contains(document.activeElement) &&
                document.activeElement?.tagName === "TEXTAREA"
            ) {
                event.preventDefault();
                event.clipboardData?.setData("text", Array.from(selectedTextareas.values()).join("\\\\#"));

                for (const key of selectedTextareas.keys()) {
                    const textarea = document.getElementById(key) as HTMLTextAreaElement;
                    textarea.value = "";
                }

                saved = false;
            }
        },
        true,
    );

    await appWindow.onCloseRequested(async (event) => {
        await awaitSaving();
        (await exitProgram()) ? await exit() : event.preventDefault();
    });
});
