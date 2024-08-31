/* eslint-disable @typescript-eslint/no-unused-expressions */
/* eslint-disable @typescript-eslint/no-dynamic-delete */
import {
    animateProgressText,
    applyLocalization,
    applyTheme,
    extractStrings,
    getThemeStyleSheet,
    readScripts,
    romanizeString,
} from "./extensions/functions";
import "./extensions/htmlelement-extensions";
import { invokeCompile, invokeRead } from "./extensions/invokes";
import { MainWindowLocalization } from "./extensions/localization";
import "./extensions/string-extensions";
import { EngineType, Language, ProcessingMode, State } from "./types/enums";

import { ask, message, open as openPath } from "@tauri-apps/api/dialog";
import { emit, listen } from "@tauri-apps/api/event";
import {
    FileEntry,
    createDir,
    exists,
    readBinaryFile,
    readDir,
    readTextFile,
    removeFile,
    writeBinaryFile,
    writeTextFile,
} from "@tauri-apps/api/fs";
import { locale as getLocale } from "@tauri-apps/api/os";
import { BaseDirectory, join } from "@tauri-apps/api/path";
import { exit } from "@tauri-apps/api/process";
import { invoke } from "@tauri-apps/api/tauri";
import { WebviewWindow, appWindow } from "@tauri-apps/api/window";
const { Resource } = BaseDirectory;

import { dump, load } from "@savannstm/marshal";
import { deflate, inflate } from "pako";
import XRegExp from "xregexp";

document.addEventListener("DOMContentLoaded", async () => {
    const tw = (strings: TemplateStringsArray, ...values: string[]): string => String.raw({ raw: strings }, ...values);
    // #region Static constants
    const sheet = getThemeStyleSheet() as CSSStyleSheet;

    const resDir = "res";
    const translationDir = "translation";
    const backupDir = "backups";

    const mapsDir = "maps";
    const otherDir = "other";
    const pluginsDir = "plugins";

    const programDataDir = ".rpgm-translation-gui";
    const logFile = "replacement-log.json";
    const compileSettingsFile = "compile-settings.json";
    const bookmarksFile = "bookmarks.txt";

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
    const topPanelButtonsContainer = document.getElementById("top-panel-buttons") as HTMLDivElement;
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
    const bookmarksButton = document.getElementById("bookmarks-button") as HTMLButtonElement;
    const bookmarksMenu = document.getElementById("bookmarks-menu") as HTMLDivElement;
    const projectStatus = document.getElementById("project-status") as HTMLDivElement;
    const currentGameEngine = document.getElementById("current-game-engine") as HTMLDivElement;
    const currentGameTitle = document.getElementById("current-game-title") as HTMLInputElement;
    // #endregion

    // #region Program initialization
    let clickTimer: number | null = null;
    let backupIsActive = false;
    let originalDir = "";

    let windowLocalization: MainWindowLocalization;
    let windowLanguage: Language;

    const settings: Settings = (await exists(settingsPath, { dir: Resource }))
        ? JSON.parse(await readTextFile(settingsPath, { dir: Resource }))
        : await createSettings();

    // Set theme
    const themes = JSON.parse(await readTextFile(themesPath, { dir: Resource })) as ThemeObject;

    let theme: Theme = themes[settings.theme];
    let currentTheme: string;
    let nextBackupNumber: number;

    await setTheme(theme);

    // Set language
    await setLanguage(settings.language);

    // Initialize the project
    await initializeProject(settings.projectPath);
    // #endregion

    const replaced = new Map<string, Record<string, string>>();
    const activeGhostLines: HTMLDivElement[] = [];

    const selectedTextareas = new Map<string, string>();
    const replacedTextareas = new Map<string, string>();

    const bookmarks: Set<string> = await fetchBookmarks(
        await join(settings.projectPath, programDataDir, bookmarksFile),
    );

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

    let multipleTextAreasSelected = false;

    leftPanel.style.height = `${window.innerHeight - topPanel.clientHeight - menuBar.clientHeight}px`;

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

    function addBookmark(bookmarkTitle: string) {
        const bookmarkElement = document.createElement("button");
        bookmarkElement.className = tw`backgroundPrimary backgroundSecondHovered flex flex-row items-center justify-center p-2`;
        bookmarkElement.innerHTML = bookmarkTitle;

        bookmarksMenu.appendChild(bookmarkElement);
    }

    async function fetchBookmarks(bookmarksFilePath: string) {
        if (!(await exists(bookmarksFilePath))) {
            return new Set<string>();
        }

        const bookmarksFileContent = await readTextFile(bookmarksFilePath);
        const bookmarkTitles = bookmarksFileContent.split(",");

        for (const bookmarkTitle of bookmarkTitles) {
            if (!bookmarkTitle) {
                continue;
            }

            const parts = bookmarkTitle.split("-", 2);
            const rowElement = document.getElementById(`${parts[0]}-row-${parts[1]}`);
            rowElement?.lastElementChild?.firstElementChild?.classList.toggle("backgroundThird");

            addBookmark(bookmarkTitle);
        }

        return new Set<string>(bookmarkTitles);
    }

    async function createDataDir(path: string) {
        if (!(await exists(path))) {
            await createDir(path);
        }
    }

    async function determineLanguage(): Promise<Language> {
        const locale: string = (await getLocale()) ?? "en";

        switch (locale) {
            case "ru":
            case "uk":
            case "be":
                return Language.Russian;
            default:
                return Language.English;
        }
    }

    async function handleMousedown(event: MouseEvent) {
        const target = event.target as HTMLElement;

        if (target.classList.contains("bookmarkButton")) {
            const row = target.closest("[id]") as HTMLDivElement;
            const parts = row.id.split("-", 3) as string[];
            const bookmarkText = `${parts[0]}-${parts.at(-1) as string}`;

            if (bookmarks.has(bookmarkText)) {
                bookmarks.delete(bookmarkText);

                for (const bookmark of bookmarksMenu.children) {
                    if ((bookmark.textContent as string) === bookmarkText) {
                        bookmark.remove();
                    }
                }
            } else {
                bookmarks.add(bookmarkText);
                addBookmark(bookmarkText);
            }

            await writeTextFile(
                await join(settings.projectPath, programDataDir, bookmarksFile),
                Array.from(bookmarks).join(","),
            );
            target.classList.toggle("backgroundThird");
        } else if (event.button === 0) {
            if (shiftPressed) {
                if (
                    contentContainer.contains(document.activeElement) &&
                    document.activeElement?.tagName === "TEXTAREA"
                ) {
                    event.preventDefault();
                    selectedTextareas.clear();

                    multipleTextAreasSelected = true;
                    const target = event.target as HTMLTextAreaElement;

                    const targetId = target.id.split("-");
                    const targetRow = Number.parseInt(targetId.pop() as string);

                    const focusedElementId = document.activeElement.id.split("-");
                    const focusedElementRow = Number.parseInt(focusedElementId.pop() as string);

                    const rowsRange = targetRow - focusedElementRow;
                    const rowsToSelect = Math.abs(rowsRange);

                    if (rowsRange > 0) {
                        for (let i = 0; i < rowsToSelect + 1; i++) {
                            const line = focusedElementRow + i;

                            const nextElement = document.getElementById(
                                `${targetId.join("-")}-${line}`,
                            ) as HTMLTextAreaElement;

                            nextElement.style.borderColor = theme.borderFocused;
                            selectedTextareas.set(nextElement.id, nextElement.value);
                        }
                    } else {
                        for (let i = rowsToSelect; i >= 0; i--) {
                            const line = focusedElementRow - i;

                            const nextElement = document.getElementById(
                                `${targetId.join("-")}-${line}`,
                            ) as HTMLTextAreaElement;

                            nextElement.style.borderColor = theme.borderFocused;
                            selectedTextareas.set(nextElement.id, nextElement.value);
                        }
                    }
                }
            } else {
                multipleTextAreasSelected = false;

                for (const id of selectedTextareas.keys()) {
                    const element = document.getElementById(id) as HTMLTextAreaElement;
                    element.style.borderColor = "";
                }
            }
        }
    }

    function showThemeWindow(): void {
        themeWindow.classList.remove("hidden");

        function changeStyle(inputElement: HTMLInputElement) {
            const id = inputElement.id;
            const value = inputElement.value;

            applyTheme(sheet, [id, value]);
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

    async function ensureProjectIsValid(pathToProject: string): Promise<boolean> {
        if (!pathToProject) {
            return false;
        }

        if (!(await exists(pathToProject))) {
            await message(windowLocalization.selectedFolderMissing);
            return false;
        }

        if (await exists(await join(pathToProject, "Data"))) {
            originalDir = "Data";
        } else if (await exists(await join(pathToProject, "data"))) {
            originalDir = "data";
        }

        if (await exists(await join(pathToProject, "original"))) {
            originalDir = "original";
        }

        if (await exists(await join(pathToProject, originalDir, "System.rxdata"))) {
            settings.engineType = EngineType.XP;
            currentGameEngine.innerHTML = "XP";
        } else if (await exists(await join(pathToProject, originalDir, "System.rvdata"))) {
            settings.engineType = EngineType.VX;
            currentGameEngine.innerHTML = "VX";
        } else if (await exists(await join(pathToProject, originalDir, "System.rvdata2"))) {
            settings.engineType = EngineType.VXAce;
            currentGameEngine.innerHTML = "VX Ace";
        } else if (await exists(await join(pathToProject, originalDir, "System.json"))) {
            settings.engineType = EngineType.New;
            currentGameEngine.innerHTML = "MV / MZ";
        } else {
            await message(windowLocalization.cannotDetermineEngine);

            changeState(null);
            contentContainer.innerHTML = "";
            currentGameEngine.innerHTML = "";
            currentGameTitle.value = "";
            return false;
        }

        return true;
    }

    async function openDirectory(): Promise<void> {
        const directory = (await openPath({ directory: true, multiple: false })) as string;

        if (directory) {
            if (directory === settings.projectPath) {
                await message(windowLocalization.directoryAlreadyOpened);
                return;
            }

            changeState(null);
            contentContainer.innerHTML = "";
            currentGameTitle.innerHTML = "";

            await initializeProject(directory);
        }
    }

    async function createSettings(): Promise<Settings | undefined> {
        const language = await determineLanguage();

        windowLanguage = language;
        windowLocalization = new MainWindowLocalization(language);

        await message(windowLocalization.cannotGetSettings);
        const askCreateSettings = await ask(windowLocalization.askCreateSettings);

        if (askCreateSettings) {
            await writeTextFile(
                settingsPath,
                JSON.stringify({
                    backup: { enabled: true, period: 60, max: 99 },
                    language: language,
                    theme: "cool-zinc",
                    font: "Segoe UI",
                    firstLaunch: true,
                    projectPath: "",
                    engineType: null,
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

    async function setLanguage(language: Language) {
        if (!windowLanguage || !windowLocalization) {
            windowLanguage = language;
            windowLocalization = new MainWindowLocalization(language);
        }

        initializeLocalization();
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

        const attr = searchCase ? "g" : "gi";

        try {
            return XRegExp(regexp, attr);
        } catch (err) {
            await message(`${windowLocalization.invalidRegexp} (${text}), ${err}`);
            return;
        }
    }

    function appendMatch(element: HTMLDivElement, result: string): void {
        const resultContainer = document.createElement("div");

        const resultElement = document.createElement("div");
        resultElement.className = tw`textSecond borderPrimary backgroundSecond my-1 cursor-pointer border-2 p-1 text-xl`;

        const thirdParent = element.parentElement?.parentElement?.parentElement as HTMLDivElement;

        const [counterpartElement, sourceIndex] = element.id.includes("original")
            ? [document.getElementById(element.id.replace("original", "translation")) as HTMLElement, 1]
            : [document.getElementById(element.id.replace("translation", "original")) as HTMLElement, 0];

        const [source, row] = element.id.split("-", 3).slice(1, 3);

        const mainDiv = document.createElement("div");
        mainDiv.className = tw`text-base`;

        const resultDiv = document.createElement("div");
        resultDiv.innerHTML = result;
        mainDiv.appendChild(resultDiv);

        const originalInfo = document.createElement("div");
        originalInfo.className = tw`textThird text-xs`;

        const secondParent = element.parentElement?.parentElement as HTMLElement;
        const currentFile = secondParent.id.slice(0, secondParent.id.lastIndexOf("-"));
        originalInfo.innerHTML = `${currentFile} - ${source} - ${row}`;
        mainDiv.appendChild(originalInfo);

        const arrow = document.createElement("div");
        arrow.className = tw`textSecond flex items-center justify-center font-material text-xl`;
        arrow.innerHTML = "arrow_downward";
        mainDiv.appendChild(arrow);

        const counterpart = document.createElement("div");
        counterpart.innerHTML =
            counterpartElement.tagName === "TEXTAREA"
                ? (counterpartElement as HTMLTextAreaElement).value.replaceAllMultiple({ "<": "&lt;", ">": "&gt;" })
                : counterpartElement.innerHTML.replaceAllMultiple({ "<": "&lt;", ">": "&gt;" });
        mainDiv.appendChild(counterpart);

        const counterpartInfo = document.createElement("div");
        counterpartInfo.className = tw`textThird text-xs`;

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

        for (const file of await readDir(await join(settings.projectPath, programDataDir))) {
            if (file.name?.startsWith("matches")) {
                await removeFile(file.path);
            }
        }

        const results = replace ? new Map<HTMLTextAreaElement, string>() : null;
        const objectToWrite = new Map<string, string>();
        let file = 0;

        const searchArray = (
            searchLocation && state
                ? [...(document.getElementById(state)?.children as HTMLCollectionOf<HTMLElement>)]
                : [...(contentContainer.children as HTMLCollectionOf<HTMLElement>)].flatMap((parent) => [
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

                    replace
                        ? (results as Map<HTMLTextAreaElement, string>).set(node[2], result)
                        : objectToWrite.set(node[2].id, result);
                }
            }

            if (!searchTranslation) {
                const elementText = node[1].innerHTML.replaceAllMultiple({ "<": "&lt;", ">": "&gt;" });
                const matches = elementText.match(regexp) as RegExpMatchArray;

                if (matches) {
                    const result = createMatchesContainer(elementText, matches);

                    replace
                        ? (results as Map<HTMLTextAreaElement, string>).set(node[1], result)
                        : objectToWrite.set(node[1].id, result);
                }
            }

            if ((objectToWrite.size + 1) % 1000 === 0) {
                await writeTextFile(
                    await join(settings.projectPath, programDataDir, `matches-${file}.json`),
                    JSON.stringify(Object.fromEntries(objectToWrite)),
                );

                objectToWrite.clear();
                file++;
            }
        }

        if (objectToWrite.size > 0) {
            await writeTextFile(
                await join(settings.projectPath, programDataDir, `matches-${file}.json`),
                JSON.stringify(Object.fromEntries(objectToWrite)),
            );
        }

        searchTotalPages.textContent = file.toString();
        searchCurrentPage.textContent = "0";

        for (const [id, result] of Object.entries(
            JSON.parse(await readTextFile(await join(settings.projectPath, programDataDir, "matches-0.json"))),
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

            element.innerHTML = `<span class="text-base"><code>${element.firstElementChild?.textContent}</code>\n${windowLocalization.textReverted}\n<code>${element.children[1].textContent}</code></span>`;
            element.setAttribute("reverted", "");

            const replacementLogContent: Record<string, { original: string; translation: string }> = JSON.parse(
                await readTextFile(await join(settings.projectPath, programDataDir, logFile)),
            );

            delete replacementLogContent[clicked.id];

            await writeTextFile(
                await join(settings.projectPath, programDataDir, logFile),
                JSON.stringify(replacementLogContent),
            );
        }
    }

    function showSearchPanel(hide = true): void {
        if (searchPanel.getAttribute("moving") === "false") {
            if (hide) {
                searchPanel.toggleMultiple("translate-x-0", "translate-x-full");
            } else {
                searchPanel.classList.replace("translate-x-full", "translate-x-0");
            }

            searchPanel.setAttribute("moving", "true");
        }

        let loadingContainer: HTMLDivElement | null = null;

        if (searchPanelFound.children.length > 0 && searchPanelFound.firstElementChild?.id !== "no-results") {
            loadingContainer = document.createElement("div");
            loadingContainer.className = tw`flex size-full items-center justify-center`;
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

            if (loadingContainer) {
                searchPanelFound.removeChild(loadingContainer);
            }

            searchPanel.setAttribute("moving", "false");
        }
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
        const target = event.target as HTMLDivElement;

        const resultElement = (
            target.parentElement?.hasAttribute("data")
                ? target.parentElement
                : target.parentElement?.parentElement?.hasAttribute("data")
                  ? target.parentElement?.parentElement
                  : target.parentElement?.parentElement?.parentElement
        ) as HTMLDivElement;

        if (!searchPanelFound.contains(resultElement)) {
            return;
        }

        const [thirdParent, element, counterpartIndex] = (resultElement.getAttribute("data") as string).split(",", 3);

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

            const highlightedReplacement = document.createElement("span");
            highlightedReplacement.className = tw`bg-red-600`;
            highlightedReplacement.textContent = replacerText;

            const newText: string[] = text.value.split(regexp);
            const newTextParts: (string | HTMLSpanElement)[] = newText.flatMap((part: string, i: number) => [
                part,
                i < newText.length - 1 ? highlightedReplacement : "",
            ]);

            const newValue: string = newText.join(replacerText);

            replaced.set(text.id, { original: text.value, translation: newValue });
            const prevFile: Record<string, Record<string, string>> = JSON.parse(
                await readTextFile(await join(settings.projectPath, programDataDir, logFile)),
            );

            const newObject: Record<string, Record<string, string>> = {
                ...prevFile,
                ...Object.fromEntries([...replaced]),
            };

            await writeTextFile(await join(settings.projectPath, programDataDir, logFile), JSON.stringify(newObject));
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
            await readTextFile(await join(settings.projectPath, programDataDir, logFile)),
        );

        const newObject: Record<string, Record<string, string>> = {
            ...prevFile,
            ...Object.fromEntries([...replaced]),
        };

        await writeTextFile(await join(settings.projectPath, programDataDir, logFile), JSON.stringify(newObject));
        replaced.clear();
    }

    async function save(backup = false): Promise<void> {
        if (saving || !settings.projectPath) {
            return;
        }

        saving = true;
        saveButton.firstElementChild?.classList.add("animate-spin");

        let dirName: string = await join(settings.projectPath, programDataDir, translationDir);

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

            nextBackupNumber = (nextBackupNumber % settings.backup.max) + 1;

            dirName = await join(
                settings.projectPath,
                programDataDir,
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
                const node = child.firstElementChild?.children[2] as HTMLTextAreaElement;
                outputArray.push(node.value.replaceAll("\n", "\\#"));
            }

            if (contentElement.id === "system") {
                outputArray.push((document.getElementById("current-game-title") as HTMLInputElement).value);
            }

            const dirPath =
                settings.engineType === EngineType.New
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

    function backup() {
        const intervalId = setInterval(async () => {
            if (settings.backup.enabled) {
                backupIsActive = true;
                await save(true);
            } else {
                backupIsActive = false;
                clearInterval(intervalId);
            }
        }, settings.backup.period * 1000);
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

    function goToRow() {
        goToRowInput.classList.remove("hidden");
        goToRowInput.focus();

        const element = document.getElementById(state as string) as HTMLDivElement;
        const lastRow = element.lastElementChild?.id.split("-", 3).at(-1) as string;

        goToRowInput.placeholder = `${windowLocalization.goToRow} ${lastRow}`;
    }

    function jumpToRow(key: string): void {
        const focusedElement = document.activeElement as HTMLElement;
        if (!contentContainer.contains(focusedElement) && focusedElement.tagName !== "TEXTAREA") {
            return;
        }

        const idParts = focusedElement.id.split("-", 3);
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

        const scrollOffset = nextElement.clientHeight;
        window.scrollBy(0, step * scrollOffset);
        focusedElement.blur();
        nextElement.focus();
        nextElement.setSelectionRange(0, 0);
    }

    async function handleKeypress(event: KeyboardEvent): Promise<void> {
        if (!settings.projectPath) {
            return;
        }

        if (event.key === "Tab") {
            event.preventDefault();
        }

        if (document.activeElement?.id === document.body.id) {
            if (event.ctrlKey) {
                switch (event.code) {
                    case "KeyZ":
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
                        break;
                    case "KeyS":
                        await save();
                        break;
                    case "KeyG":
                        event.preventDefault();

                        if (state && goToRowInput.classList.contains("hidden")) {
                            goToRow();
                        } else if (!goToRowInput.classList.contains("hidden")) {
                            goToRowInput.classList.add("hidden");
                        }
                        break;
                    case "KeyF":
                        event.preventDefault();

                        searchMenu.classList.replace("hidden", "flex");
                        searchInput.focus();
                        break;
                    case "KeyB":
                        bookmarksMenu.toggleMultiple("hidden", "flex");

                        requestAnimationFrame(() => {
                            bookmarksMenu.style.left = `${bookmarksButton.offsetLeft}px`;
                            bookmarksMenu.style.top = `${menuBar.clientHeight + topPanel.clientHeight}px`;
                        });
                        break;
                }
            } else if (event.altKey) {
                switch (event.code) {
                    case "KeyC":
                        await compile(false);
                        break;
                    case "KeyR":
                        await createReadWindow();
                        break;
                    case "F4":
                        await appWindow.close();
                        break;
                }
            } else {
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
                    case "Backquote":
                        changeState(State.Maps);
                        break;
                    case "Digit1":
                        changeState(State.Names);
                        break;
                    case "Digit2":
                        changeState(State.Actors);
                        break;
                    case "Digit3":
                        changeState(State.Armors);
                        break;
                    case "Digit4":
                        changeState(State.Classes);
                        break;
                    case "Digit5":
                        changeState(State.CommonEvents);
                        break;
                    case "Digit6":
                        changeState(State.Enemies);
                        break;
                    case "Digit7":
                        changeState(State.Items);
                        break;
                    case "Digit8":
                        changeState(State.Skills);
                        break;
                    case "Digit9":
                        changeState(State.States);
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

        if (event.key === "Shift" && !event.repeat) {
            shiftPressed = true;
        }
    }

    async function handleSearchInputKeypress(event: KeyboardEvent): Promise<void> {
        if (!settings.projectPath) {
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
        if (!settings.projectPath) {
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

    function fromHTML(html: string): HTMLElement | HTMLCollection {
        const template = document.createElement("template");
        template.innerHTML = html;

        const result = template.content.children as HTMLCollectionOf<HTMLElement>;

        if (result.length === 1) {
            return result[0];
        }

        return result;
    }

    async function createContent(): Promise<void> {
        if (!settings.projectPath) {
            return;
        }

        const contentNames: string[] = [];
        const content: string[][] = [];

        for (const entry of await readDir(await join(settings.projectPath, programDataDir, translationDir), {
            recursive: true,
        })) {
            const folder = entry.name as string;

            if (folder.startsWith(".")) {
                continue;
            }

            for (const file of entry.children as FileEntry[]) {
                const name = file.name as string;

                if (!name.endsWith(".txt") || name.includes("plain")) {
                    continue;
                }

                contentNames.push(name.slice(0, -4));
                content.push(
                    (
                        await readTextFile(
                            await join(settings.projectPath, programDataDir, translationDir, folder, name),
                        )
                    ).split("\n"),
                );
            }
        }

        if (contentNames.includes("scripts")) {
            (leftPanel.lastElementChild as HTMLElement).innerHTML = State.Scripts;
        }

        for (let i = 0; i < contentNames.length - 1; i += 2) {
            const contentName = contentNames[i];
            const contentDiv = document.createElement("div");
            contentDiv.id = contentName;
            contentDiv.className = tw`hidden flex-col`;

            if (contentName === "system") {
                const originalGameTitle = content[i].pop() as string;
                const translatedGameTitle = content[i + 1].pop() as string;

                if (translatedGameTitle === "") {
                    currentGameTitle.value = originalGameTitle;
                } else {
                    currentGameTitle.value = translatedGameTitle;
                }
            }

            let contentDivHeight = 0;

            for (let j = 0; j < content[i].length; j++) {
                const originalText = content[i][j].replaceAll("\\#", "\n");
                const translationText = content[i + 1][j];

                const textParent = document.createElement("div");
                textParent.id = `${contentName}-${j + 1}`;
                textParent.className = tw`-mb-0.5 h-auto w-full`;

                const textContainer = document.createElement("div");
                textContainer.className = tw`flex h-full flex-row justify-around`;

                const originalTextElement = document.createElement("div");
                originalTextElement.title = windowLocalization.originalTextFieldTitle;
                originalTextElement.id = `${contentName}-original-${j + 1}`;
                originalTextElement.className = tw`backgroundPrimary borderPrimary -ml-0.5 inline-block w-full cursor-pointer whitespace-pre-wrap border-2 p-1`;
                originalTextElement.textContent = originalText;

                const translationTextSplit = translationText.split("\\#");
                const translationTextElement = document.createElement("textarea");
                translationTextElement.id = `${contentName}-translation-${j + 1}`;
                translationTextElement.rows = translationTextSplit.length;
                translationTextElement.className = tw`borderPrimary backgroundPrimary borderFocused -ml-0.5 h-auto w-full resize-none overflow-hidden border-2 p-1 outline-none focus:z-10`;
                translationTextElement.value = translationTextSplit.join("\n");

                const rowElement = document.createElement("div");
                rowElement.id = `${contentName}-row-${j + 1}`;
                rowElement.className = tw`backgroundPrimary borderPrimary flex w-48 flex-row border-2 p-1`;

                const spanElement = document.createElement("span");
                spanElement.textContent = (j + 1).toString();

                const innerDiv = document.createElement("div");
                innerDiv.className = tw`flex w-full items-start justify-end p-0.5`;

                const button = document.createElement("button");
                button.className = tw`borderPrimary backgroundPrimaryHovered textThird flex h-6 w-6 items-center justify-center rounded-md border-2 font-material text-xl`;
                button.textContent = "bookmark";

                innerDiv.appendChild(button);
                rowElement.appendChild(spanElement);
                rowElement.appendChild(innerDiv);

                document.body.appendChild(originalTextElement);

                const minHeight =
                    originalTextElement.getBoundingClientRect().height -
                    Number.parseInt(window.getComputedStyle(originalTextElement).borderTop);

                document.body.removeChild(originalTextElement);

                rowElement.style.minHeight =
                    originalTextElement.style.minHeight =
                    translationTextElement.style.minHeight =
                    textParent.style.minHeight =
                        `${minHeight}px`;

                // this 2 subtraction just required
                contentDivHeight += minHeight - 2;

                textContainer.appendChild(rowElement);
                textContainer.appendChild(originalTextElement);
                textContainer.appendChild(translationTextElement);
                textParent.appendChild(textContainer);
                contentDiv.appendChild(textParent);
            }

            contentDiv.style.minHeight = `${contentDivHeight}px`;
            contentContainer.appendChild(contentDiv);
        }

        if (projectStatus) {
            projectStatus.innerHTML = "";
        }
    }

    async function writeScripts(scriptsFilePath: string, otherPath: string, outputPath: string, romanize: boolean) {
        const scriptEntries = load(await readBinaryFile(scriptsFilePath), { stringMode: "binary" }) as {
            __type: "bytes";
            data: number[];
        }[][];
        const originalScriptsText = (await readTextFile(await join(otherPath, "scripts.txt"))).split("\n");
        const translatedScriptsText = (await readTextFile(await join(otherPath, "scripts_trans.txt"))).split("\n");

        const scriptsTranslationMap = new Map(originalScriptsText.map((value, i) => [value, translatedScriptsText[i]]));

        const decoder = new TextDecoder("utf-8", { fatal: true });

        for (const [i, script] of scriptEntries.entries()) {
            let code: string;

            {
                const inflated = inflate(new Uint8Array(script[2].data));

                try {
                    code = decoder.decode(inflated);
                } catch {
                    try {
                        code = new TextDecoder("windows-1252", { fatal: true }).decode(inflated);
                    } catch {
                        try {
                            code = new TextDecoder("shift-jis", { fatal: true }).decode(inflated);
                        } catch {
                            code = new TextDecoder("gbk", { fatal: true }).decode(inflated);
                        }
                    }
                }
            }

            const [stringArray, indexArray] = extractStrings(code, true) as [string[], number[]];

            for (let i = stringArray.length - 1; i >= 0; i--) {
                let string = stringArray[i];

                if (string.length === 0 || !scriptsTranslationMap.has(string)) {
                    continue;
                }

                if (romanize) {
                    string = romanizeString(string);
                }

                const translated = scriptsTranslationMap.get(string) as string;

                if (translated) {
                    const before = code.slice(0, indexArray[i]);
                    const after = code.slice(indexArray[i] + string.length);

                    code = before + translated + after;
                }
            }

            scriptEntries[i][2] = { __type: "bytes", data: Array.from(deflate(code, { level: 6 })) };
        }

        await writeBinaryFile(
            await join(
                outputPath,
                `Scripts.${
                    settings.engineType === EngineType.VXAce
                        ? "rvdata2"
                        : settings.engineType === EngineType.VX
                          ? "rvdata"
                          : "rxdata"
                }`,
            ),
            dump(scriptEntries),
        );
    }

    async function compile(silent: boolean): Promise<void> {
        if (!settings.projectPath) {
            return;
        }

        const compileSettings: CompileSettings = JSON.parse(
            await readTextFile(await join(settings.projectPath, programDataDir, compileSettingsFile)),
        );

        if (!compileSettings.initialized || !compileSettings.doNotAskAgain || !silent) {
            const compileWindow = new WebviewWindow("compile", {
                url: "./compile.html",
                title: windowLocalization.compileWindowTitle,
                width: 640,
                height: 480,
                center: true,
            });

            const compileUnlisten = await compileWindow.once("compile", async () => {
                await compile(true);
            });

            await compileWindow.once("tauri://destroyed", compileUnlisten);
        } else {
            compileButton.firstElementChild?.classList.add("animate-spin");

            const executionTime = await invokeCompile({
                projectPath: settings.projectPath,
                originalDir,
                outputPath: compileSettings.customOutputPath.path,
                gameTitle: currentGameTitle.value,
                romanize: compileSettings.romanize,
                shuffleLevel: compileSettings.shuffle.level,
                disableCustomProcessing: compileSettings.disableCustomProcessing,
                disableProcessing: Object.values(compileSettings.disableProcessing.of),
                logging: compileSettings.logging,
                engineType: settings.engineType,
            });

            if (!compileSettings.disableProcessing.of.plugins) {
                await writeScripts(
                    await join(
                        settings.projectPath,
                        originalDir,
                        `Scripts.${
                            settings.engineType === EngineType.VXAce
                                ? "rvdata2"
                                : settings.engineType === EngineType.VX
                                  ? "rvdata"
                                  : "rxdata"
                        }`,
                    ),
                    await join(settings.projectPath, programDataDir, translationDir, otherDir),
                    await join(settings.projectPath, programDataDir, "output", "Data"),
                    compileSettings.romanize,
                );
            }

            alert(`${windowLocalization.compileSuccess} ${executionTime}`);
            compileButton.firstElementChild?.classList.remove("animate-spin");
        }
    }

    function getNewLinePositions(textarea: HTMLTextAreaElement): { left: number; top: number }[] {
        const positions: { left: number; top: number }[] = [];
        const lines = textarea.value.split("\n");

        const { lineHeight, paddingTop, fontSize, fontFamily } = window.getComputedStyle(textarea);
        const lineHeightNumber = Number.parseInt(lineHeight);
        const paddingTopNumber = Number.parseInt(paddingTop);

        const offsetTop = textarea.offsetTop + paddingTopNumber / 2;
        const offsetLeft = textarea.offsetLeft;

        const canvas = document.createElement("canvas");
        const context = canvas.getContext("2d") as CanvasRenderingContext2D;
        context.font = `${fontSize} ${fontFamily}`;

        let top = offsetTop;

        for (let i = 0; i < lines.length - 1; i++) {
            const line = lines[i];
            const textWidth = context.measureText(`${line} `).width;
            const left = offsetLeft + textWidth;

            positions.push({ left, top });
            top += lineHeightNumber;
        }

        return positions;
    }

    function trackFocus(focusedElement: HTMLTextAreaElement): void {
        for (const ghost of activeGhostLines) {
            ghost.remove();
        }

        const result = getNewLinePositions(focusedElement);

        for (const { left, top } of result) {
            const ghostNewLine = fromHTML(
                `<div class="z-50 cursor-default pointer-events-none select-none absolute textThird" style="left: ${left}px; top: ${top}px">\\n</div>`,
            ) as HTMLDivElement;

            activeGhostLines.push(ghostNewLine);
            document.body.appendChild(ghostNewLine);
        }
    }

    function handleFocus(event: FocusEvent): void {
        const target = event.target as HTMLTextAreaElement;

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
                target.removeEventListener("keyup", () => target.calculateHeight());
                target.removeEventListener("input", () => trackFocus(target));
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

    async function createSettingsWindow() {
        const settingsWindow = new WebviewWindow("settings", {
            url: "./settings.html",
            title: windowLocalization.settingsButtonTitle,
            width: 800,
            height: 600,
            center: true,
            resizable: false,
        });

        const settingsUnlisten = await settingsWindow.once<[boolean, number, number]>("backup-settings", (data) => {
            const [enabled, max, period] = data.payload;

            if (enabled && !backupIsActive) {
                backup();
            }

            settings.backup.enabled = enabled;
            settings.backup.max = max;
            settings.backup.period = period;
        });

        await settingsWindow.once("tauri://destroyed", settingsUnlisten);
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
                    await writeTextFile(settingsPath, JSON.stringify(settings), { dir: Resource });
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
                if (windowLanguage !== Language.Russian) {
                    windowLanguage = Language.Russian;
                    await setLanguage(Language.Russian);
                }
                break;
            case "en-button":
                if (windowLanguage !== Language.English) {
                    windowLanguage = Language.English;
                    await setLanguage(Language.English);
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

    // that's the dumbest function i've ever written
    async function awaitSaving() {
        while (saving) {
            await new Promise((resolve) => setTimeout(resolve, 2000));
        }
    }

    async function createCompileSettings(path: string) {
        if (!(await exists(path))) {
            await writeTextFile(
                path,
                JSON.stringify({
                    initialized: false,
                    logging: false,
                    romanize: false,
                    shuffle: { enabled: false, level: 0 },
                    customParsing: false,
                    customOutputPath: { enabled: false, path: "" },
                    disableCustomProcessing: false,
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

    async function createLogFile(path: string) {
        if (!(await exists(path))) {
            await writeTextFile(path, "{}");
        }
    }

    async function setTheme(newTheme: Theme) {
        if (newTheme.name === currentTheme) {
            return;
        }

        [currentTheme, theme] = [newTheme.name, newTheme];

        applyTheme(sheet, theme);
        settings.theme = newTheme.name;
    }

    function initializeLocalization() {
        applyLocalization(windowLocalization, theme);

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

        settings.firstLaunch = false;
    }

    async function loadProject() {
        const translationPath = await join(settings.projectPath, programDataDir, translationDir);
        const parsed = await exists(translationPath);

        const mapsPath = await join(translationPath, mapsDir);
        const otherPath = await join(translationPath, otherDir);

        await createDir(mapsPath, { recursive: true });
        await createDir(otherPath, { recursive: true });

        if (!parsed) {
            let gameTitle!: string;

            if (settings.engineType === EngineType.New) {
                gameTitle = JSON.parse(await readTextFile(await join(settings.projectPath, originalDir, "System.json")))
                    .gameTitle as string;
            } else {
                const iniFileContent = (await readTextFile(await join(settings.projectPath, "Game.ini"))).split("\n");

                for (const line of iniFileContent) {
                    if (line.toLowerCase().startsWith("title")) {
                        gameTitle = line.split("=")[1].trim();
                    }
                }

                const decoder = new TextDecoder();
                const codes: string[] = [];

                const serializedScriptsData = load(
                    await readBinaryFile(
                        await join(
                            settings.projectPath,
                            originalDir,
                            `Scripts.${
                                settings.engineType === EngineType.VXAce
                                    ? "rvdata2"
                                    : settings.engineType === EngineType.VX
                                      ? "rvdata"
                                      : "rxdata"
                            }`,
                        ),
                    ),
                    {
                        stringMode: "binary",
                    },
                ) as { __type: "bytes"; data: number[] }[][];

                for (const arr of serializedScriptsData) {
                    codes.push(decoder.decode(inflate(new Uint8Array(arr[2].data))));
                }

                await readScripts(codes.join(""), otherPath, false);
            }

            await invokeRead({
                projectPath: settings.projectPath,
                originalDir,
                gameTitle,
                romanize: false,
                disableCustomProcessing: false,
                disableProcessing: [false, false, false],
                logging: false,
                processingMode: ProcessingMode.Default,
                engineType: settings.engineType,
            });
        }
    }

    async function initializeProject(pathToProject: string): Promise<void> {
        projectStatus.innerHTML = windowLocalization.loadingProject;
        const interval = animateProgressText(projectStatus);

        const projectIsValid = await ensureProjectIsValid(pathToProject);

        if (!projectIsValid) {
            settings.projectPath = "";
            projectStatus.innerHTML = windowLocalization.noProjectSelected;
        } else {
            settings.projectPath = pathToProject;

            await loadProject();

            await createDataDir(await join(settings.projectPath, programDataDir));
            await createLogFile(await join(settings.projectPath, programDataDir, logFile));
            await createCompileSettings(await join(settings.projectPath, programDataDir, compileSettingsFile));
            initializeThemes();
            await createDir(await join(settings.projectPath, programDataDir, backupDir), { recursive: true });

            nextBackupNumber = (await readDir(await join(settings.projectPath, programDataDir, backupDir)))
                .map((entry) => {
                    const name = entry.name as string;
                    const number = name.slice(0, -2);
                    return Number.parseInt(number);
                })
                .sort((a, b) => a - b)[0];

            if (!nextBackupNumber) {
                nextBackupNumber = 0;
            }

            if (settings.backup.enabled) {
                backup();
            }

            if (settings.firstLaunch) {
                await initializeFirstLaunch();
            }

            await createContent();
        }

        clearInterval(interval);
    }

    function initializeThemes() {
        for (const themeName of Object.keys(themes)) {
            const themeButton = document.createElement("button");
            themeButton.id = themeName;
            themeButton.innerHTML = themeName;
            themeButton.className = tw`backgroundPrimary backgroundPrimaryHovered p-2 text-base`;

            themeMenu.insertBefore(themeButton, createThemeMenuButton);
        }
    }

    async function createReadWindow() {
        const readWindow = new WebviewWindow("read", {
            title: windowLocalization.readWindowTitle,
            url: "./read.html",
            width: 640,
            height: 720,
            center: true,
        });

        const unlistenRestart = await readWindow.once("restart", async () => {
            await writeTextFile(settingsPath, JSON.stringify(settings), { dir: Resource });
            location.reload();
        });

        const unlistenFetch = await readWindow.once("fetch", async () => {
            await emit("metadata", [
                (document.getElementById("current-game-title") as HTMLInputElement).value,
                settings.engineType,
            ]);
        });

        await readWindow.once("tauri://destroyed", () => {
            unlistenRestart();
            unlistenFetch();
        });
    }

    leftPanel.addEventListener("click", (event) => {
        const newState = leftPanel.secondHighestParent(event.target as HTMLElement).textContent as State;
        changeState(newState, true);
    });

    topPanelButtonsContainer.addEventListener("click", async (event) => {
        const target = topPanelButtonsContainer.secondHighestParent(event.target as HTMLElement);

        if ((!settings.projectPath && target.id !== "open-directory-button") || target === topPanelButtonsContainer) {
            return;
        }

        switch (target.id) {
            case "menu-button":
                leftPanel.toggleMultiple("translate-x-0", "-translate-x-full");
                break;
            case saveButton.id:
                await save();
                break;
            case "compile-button":
                if (clickTimer === null) {
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
            case "open-directory-button":
                await openDirectory();
                break;
            case "settings-button":
                createSettingsWindow();
                break;
            case themeButton.id:
                themeMenu.toggleMultiple("hidden", "flex");

                requestAnimationFrame(() => {
                    themeMenu.style.left = `${themeButton.offsetLeft}px`;
                    themeMenu.style.top = `${menuBar.clientHeight + topPanel.clientHeight}px`;

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
            case bookmarksButton.id:
                bookmarksMenu.toggleMultiple("hidden", "flex");

                requestAnimationFrame(() => {
                    bookmarksMenu.style.left = `${bookmarksButton.offsetLeft}px`;
                    bookmarksMenu.style.top = `${menuBar.clientHeight + topPanel.clientHeight}px`;
                });
                break;
            case "read-button": {
                await createReadWindow();
                break;
            }
            case "search-button":
                if (searchMenu.classList.contains("hidden")) {
                    searchMenu.classList.remove("hidden");
                    requestAnimationFrame(() => {
                        searchMenu.style.left = `${searchButton.offsetLeft}px`;
                        searchMenu.style.top = `${menuBar.clientHeight + topPanel.clientHeight}px`;
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

                    const searchSwitch = target;

                    if (searchSwitch.innerHTML.trim() === "search") {
                        searchSwitch.innerHTML = "menu_book";

                        const replacementLogContent: Record<string, { original: string; translation: string }> =
                            JSON.parse(await readTextFile(await join(settings.projectPath, programDataDir, logFile)));

                        for (const [key, value] of Object.entries(replacementLogContent)) {
                            const replacedContainer: HTMLDivElement = document.createElement("div");

                            const replacedElement: HTMLDivElement = document.createElement("div");
                            replacedElement.className = tw`textSecond borderPrimary backgroundSecond my-1 cursor-pointer border-2 p-1 text-xl`;

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

                if (page > 0) {
                    searchCurrentPage.textContent = (page - 1).toString();
                    searchPanelFound.innerHTML = "";

                    for (const [id, result] of Object.entries(
                        JSON.parse(
                            await readTextFile(
                                await join(
                                    settings.projectPath,
                                    programDataDir,
                                    `matches-${Number.parseInt(searchCurrentPage.textContent)}.json`,
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
                                    settings.projectPath,
                                    programDataDir,
                                    `matches-${Number.parseInt(searchCurrentPage.textContent)}.json`,
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
        if (event.key === "Shift") {
            shiftPressed = false;
        }
    });

    contentContainer.addEventListener("focus", handleFocus, true);
    contentContainer.addEventListener("blur", handleBlur, true);
    contentContainer.addEventListener("click", (event) => {
        const target = event.target as HTMLElement;

        if (target.id.includes("-original-")) {
            navigator.clipboard.writeText(target.textContent as string);
        }
    });

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
        if (!settings.projectPath) {
            return;
        }

        switch ((event.target as Element).id) {
            case "replace-button":
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
                .split(",", 2)
                .map((element) => Number.parseInt(element)) as [number, number];

            const newLeft = Math.max(
                0,
                Math.min(event.clientX - offsets[0], window.innerWidth - searchMenu.offsetWidth),
            );
            const newTop = Math.max(
                0,
                Math.min(event.clientY - offsets[1], window.innerHeight - searchMenu.offsetHeight),
            );

            searchMenu.style.left = `${newLeft}px`;
            searchMenu.style.top = `${newTop}px`;
        }
    });

    contentContainer.addEventListener("mousedown", async (event) => await handleMousedown(event));

    contentContainer.addEventListener("paste", (event) => {
        const text = event.clipboardData?.getData("text/plain");

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
    });

    contentContainer.addEventListener("copy", (event) => {
        if (
            multipleTextAreasSelected &&
            contentContainer.contains(document.activeElement) &&
            document.activeElement?.tagName === "TEXTAREA"
        ) {
            event.preventDefault();
            selectedTextareas.set(document.activeElement.id, (document.activeElement as HTMLTextAreaElement).value);
            event.clipboardData?.setData("text/plain", Array.from(selectedTextareas.values()).join("\\\\#"));
        }
    });

    bookmarksMenu.addEventListener("click", (event) => {
        const target = event.target as HTMLElement;

        if (target.id === bookmarksMenu.id) {
            return;
        }

        const parts = (target.textContent as string).split("-", 2);
        const rowId = `${parts[0]}-row-${parts[1]}`;

        changeState(parts[0] as State);
        (document.getElementById(rowId) as HTMLDivElement).scrollIntoView({
            inline: "center",
            block: "center",
        });
    });

    contentContainer.addEventListener("cut", (event) => {
        if (
            multipleTextAreasSelected &&
            contentContainer.contains(document.activeElement) &&
            document.activeElement?.tagName === "TEXTAREA"
        ) {
            event.preventDefault();
            event.clipboardData?.setData("text/plain", Array.from(selectedTextareas.values()).join("\\\\#"));

            for (const key of selectedTextareas.keys()) {
                const textarea = document.getElementById(key) as HTMLTextAreaElement;
                textarea.value = "";
            }

            saved = false;
        }
    });

    await listen("fetch-settings", async () => {
        await emit("settings", settings);
    });

    await appWindow.onCloseRequested(async (event) => {
        await awaitSaving();

        if (await exitProgram()) {
            await writeTextFile(settingsPath, JSON.stringify(settings), { dir: Resource });
            await exit();
        } else {
            event.preventDefault();
        }
    });
});
