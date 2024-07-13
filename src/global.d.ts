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

interface CSSRule {
    style: CSSStyleDeclaration;
    selectorText: string;
}
