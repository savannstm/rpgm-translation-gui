interface String {
    replaceAllMultiple(replacementObj: { [key: string]: string }): string;
    count(char: string): number;
}

interface HTMLElement {
    toggleMultiple(...classes: string[]): void;
    secondHighestParent(childElement: HTMLElement): HTMLElement;
}

interface HTMLTextAreaElement {
    calculateHeight(): void;
}

interface BackupSetting {
    enabled: boolean;
    period: number;
    max: number;
}

interface Settings {
    language: Language;
    backup: BackupSetting;
    theme: ThemeName;
    firstLaunch: boolean;
    project: string | null;
}

interface mainLocalization {
    [key: string]: string;
    cannotGetSettings: string;
    askCreateSettings: string;
    createdSettings: string;
    unsavedChanges: string;
    originalTextIrreplacable: string;
    invalidRegexp: string;
    textReverted: string;
    reloadButton: string;
    helpButton: string;
    aboutButton: string;
    hotkeysButton: string;
    exit: string;
    fileMenu: string;
    helpMenu: string;
    languageMenu: string;
    menuButtonTitle: string;
    saveButtonTitle: string;
    compileButtonTitle: string;
    optionsButtonTitle: string;
    searchButtonTitle: string;
    searchInputTitle: string;
    replaceButtonTitle: string;
    replaceInputTitle: string;
    caseButtonTitle: string;
    wholeButtonTitle: string;
    regexButtonTitle: string;
    translationButtonTitle: string;
    locationButtonTitle: string;
    noMatches: string;
    currentPage: string;
    separator: string;
    goToRow: string;
    missingTranslationDir: string;
    missingOriginalDir: string;
    missingTranslationSubdirs: string;
    noProjectSelected: string;
    backgroundDark: string;
    backgroundPrimary: string;
    backgroundSecond: string;
    backgroundThird: string;
    outlinePrimary: string;
    outlineSecond: string;
    outlineThird: string;
    outlineFocused: string;
    borderPrimary: string;
    borderSecond: string;
    borderFocused: string;
    backgroundPrimaryHovered: string;
    backgroundSecondHovered: string;
    textPrimary: string;
    textSecond: string;
    textThird: string;
    createTheme: string;
    allowedThemeNameCharacters: string;
    invalidThemeName: string;
    themeName: string;
    compileSuccess: string;
    themeButtonTitle: string;
    openButtonTitle: string;
    loadingProject: string;
}

interface optionsLocalization {
    backupPeriodLabel: string;
    backupPeriodNote: string;
    backupMaxLabel: string;
    backupMaxNote: string;
    backup: string;
}

interface aboutLocalization {
    version: string;
    about: string;
    socials: string;
    vkLink: string;
    tgLink: string;
    githubLink: string;
    license: string;
}

interface hotkeysLocalization {
    hotkeysTitle: string;
    hotkeys: string;
}

interface helpLocalization {
    helpTitle: string;
    help: string;
}

interface Localization {
    main: mainLocalization;
    options: optionsLocalization;
    about: aboutLocalization;
    hotkeys: hotkeysLocalization;
    help: helpLocalization;
}

interface ThemeObject {
    [name: string]: Theme;
}

interface Theme {
    [key: string]: string;
    name: string;
    backgroundDark: string;
    backgroundPrimary: string;
    backgroundSecond: string;
    backgroundThird: string;
    outlinePrimary: string;
    outlineSecond: string;
    outlineThird: string;
    outlineFocused: string;
    borderPrimary: string;
    borderSecond: string;
    borderFocused: string;
    backgroundPrimaryHovered: string;
    backgroundSecondHovered: string;
    textPrimary: string;
    textSecond: string;
    textThird: string;
}

type Language = "en" | "ru";

type State =
    | null
    | "maps"
    | "names"
    | "actors"
    | "armors"
    | "classes"
    | "commonevents"
    | "enemies"
    | "items"
    | "skills"
    | "system"
    | "troops"
    | "weapons"
    | "plugins";

interface CSSRule {
    style: CSSStyleDeclaration;
    selectorText: string;
}
