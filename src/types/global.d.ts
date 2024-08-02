interface String {
    replaceAllMultiple(replacementObj: Record<string, string>): string;
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
    projectPath: string;
}

interface CompileSettings {
    initialized: boolean;
    logging: boolean;
    romanize: boolean;
    shuffle: {
        enabled: boolean;
        level: number;
    };
    disableCustomProcessing: boolean;
    customOutputPath: {
        enabled: boolean;
        path: string;
    };
    disableProcessing: {
        enabled: boolean;
        of: {
            maps: boolean;
            other: boolean;
            system: boolean;
            plugins: boolean;
        };
    };
    doNotAskAgain: boolean;
}

type ThemeObject = Record<string, Theme>;

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

interface CSSRule {
    style: CSSStyleDeclaration;
    selectorText: string;
}

interface ReadCommandOptions extends Record<string, T> {
    projectPath: string;
    originalPath: string;
    gameTitle: string;
    romanize: boolean;
    disableCustomProcessing: boolean;
    disableProcessing: boolean[];
    logging: boolean;
    processingMode: ProcessingMode;
    engineType: EngineType;
}

interface CompileCommandOptions extends Record<string, T> {
    projectPath: string;
    originalPath: string;
    outputPath: string;
    gameTitle: string;
    romanize: boolean;
    shuffleLevel: number;
    disableCustomProcessing: boolean;
    disableProcessing: boolean[];
    logging: boolean;
    engineType: EngineType;
}
