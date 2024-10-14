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

interface Math {
    clamp(value: number, min: number, max: number): number;
}

interface Settings {
    language: Language;
    backup: {
        enabled: boolean;
        period: number;
        max: number;
    };
    theme: string;
    font: string;
    firstLaunch: boolean;
    projectPath: string;
    engineType: EngineType;
}

interface CompileSettings {
    initialized: boolean;
    logging: boolean;
    romanize: boolean;
    mapsProcessingMode: number;
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

interface Theme extends Record<string, string> {
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

interface ReadCommandOptions extends Record<string, unknown> {
    projectPath: string;
    originalDir: string;
    gameTitle: string;
    mapsProcessingMode: number;
    romanize: boolean;
    disableCustomProcessing: boolean;
    disableProcessing: boolean[];
    logging: boolean;
    processingMode: ProcessingMode;
    engineType: EngineType;
}

interface CompileCommandOptions extends Record<string, unknown> {
    projectPath: string;
    originalDir: string;
    outputPath: string;
    gameTitle: string;
    mapsProcessingMode: number;
    romanize: boolean;
    disableCustomProcessing: boolean;
    disableProcessing: boolean[];
    logging: boolean;
    engineType: EngineType;
}
