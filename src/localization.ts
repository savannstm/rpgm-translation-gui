import { Language } from "./enums";

export class MainLocalization {
    readonly cannotGetSettings: string;
    readonly askCreateSettings: string;
    readonly createdSettings: string;
    readonly unsavedChanges: string;
    readonly originalTextIrreplacable: string;
    readonly invalidRegexp: string;
    readonly textReverted: string;
    readonly reloadButton: string;
    readonly helpButton: string;
    readonly aboutButton: string;
    readonly hotkeysButton: string;
    readonly exit: string;
    readonly fileMenu: string;
    readonly helpMenu: string;
    readonly languageMenu: string;
    readonly menuButtonTitle: string;
    readonly saveButtonTitle: string;
    readonly compileButtonTitle: string;
    readonly optionsButtonTitle: string;
    readonly searchButtonTitle: string;
    readonly searchInputTitle: string;
    readonly replaceButtonTitle: string;
    readonly replaceInputTitle: string;
    readonly caseButtonTitle: string;
    readonly wholeButtonTitle: string;
    readonly regexButtonTitle: string;
    readonly translationButtonTitle: string;
    readonly locationButtonTitle: string;
    readonly noMatches: string;
    readonly currentPage: string;
    readonly separator: string;
    readonly goToRow: string;
    readonly missingTranslationDir: string;
    readonly missingOriginalDir: string;
    readonly missingTranslationSubdirs: string;
    readonly noProjectSelected: string;
    readonly backgroundDark: string;
    readonly backgroundPrimary: string;
    readonly backgroundSecond: string;
    readonly backgroundThird: string;
    readonly outlinePrimary: string;
    readonly outlineSecond: string;
    readonly outlineThird: string;
    readonly outlineFocused: string;
    readonly borderPrimary: string;
    readonly borderSecond: string;
    readonly borderFocused: string;
    readonly backgroundPrimaryHovered: string;
    readonly backgroundSecondHovered: string;
    readonly textPrimary: string;
    readonly textSecond: string;
    readonly textThird: string;
    readonly createTheme: string;
    readonly allowedThemeNameCharacters: string;
    readonly invalidThemeName: string;
    readonly themeName: string;
    readonly compileSuccess: string;
    readonly themeButtonTitle: string;
    readonly openButtonTitle: string;
    readonly loadingProject: string;
    readonly missingFileText: string;
    readonly cannotDetermineEngine: string;
    readonly selectedFolderMissing: string;

    constructor(language: Language) {
        switch (language) {
            case Language.Russian:
                this.cannotGetSettings = "Не удалось найти файл настроек программы.";
                this.askCreateSettings = "Создать настройки?";
                this.createdSettings =
                    "Настройки созданы.\nРезервное копирование: Включено\nПериод копирования: 60 сек.\nМаксимальное число копий: 99.\nТема: Классный цинк";
                this.unsavedChanges = "У вас остались несохранённые изменения. Сохранить прогресс и выйти?";
                this.originalTextIrreplacable = "Оригинальные строки не могут быть заменены.";
                this.invalidRegexp = "Некорректное регулярное выражение";
                this.textReverted = "Текст был возвращён к исходному значению";
                this.reloadButton = "Перезагрузить (F5)";
                this.helpButton = "Помощь";
                this.aboutButton = "О программе";
                this.hotkeysButton = "Горячие клавиши";
                this.exit = "Выйти без сохранения?";
                this.fileMenu = "Файл";
                this.helpMenu = "Помощь";
                this.languageMenu = "Язык";
                this.menuButtonTitle = "Вкладки (Tab)";
                this.saveButtonTitle = "Сохранить файлы перевода (Ctrl + S)";
                this.compileButtonTitle = "Скомпилировать в JSON файлы (Alt + C)";
                this.optionsButtonTitle = "Настройки";
                this.searchButtonTitle = "Поиск (Ctrl + F)";
                this.searchInputTitle = "Поиск";
                this.replaceButtonTitle = "Заменить все совпадения на";
                this.replaceInputTitle = "Замена";
                this.caseButtonTitle = "Учитывать регистр (Alt + C)";
                this.wholeButtonTitle = "Искать слова целиком (Alt + W)";
                this.regexButtonTitle = "Поиск по регулярным выражениям (Alt + R)";
                this.translationButtonTitle = "Поиск только по переводу (Alt + T)";
                this.locationButtonTitle = "Поиск только в текущем файле (Alt + L)";
                this.noMatches = "Нет совпадений";
                this.currentPage = "Нет";
                this.separator = "из";
                this.goToRow = "Перейти к строке... от 1 до";
                this.missingTranslationDir = 'Директория "translation" отсутствует. Проект не будет инициализирован.';
                this.missingOriginalDir =
                    'Директория "original" или "data" отсутствует. Проект не будет инициализирован.';
                this.missingTranslationSubdirs =
                    'Поддиректории "maps"; "other" и/или "plugins" директории "translation" отсутствуют. Проект не будет инициализирован.';
                this.noProjectSelected =
                    'Проект не выбран. Выберите директорию проекта, используя кнопку "открыть папку" в левом верхнем углу. Директория должна содержать в себе папки "original" или "data" и "translation" (с поддиректориями "maps"; "other" и "plugins"; если это RPG Maker MV/MZ, или "maps" и "other"; если это RPG Maker XP/VX/VXAce).';
                this.backgroundDark = "Тёмный цвет фона";
                this.backgroundPrimary = "Основной цвет фона";
                this.backgroundSecond = "Второстепенный цвет фона";
                this.backgroundThird = "Третий цвет фона";
                this.outlinePrimary = "Основной цвет контура";
                this.outlineSecond = "Второстепенный цвет контура";
                this.outlineThird = "Третий цвет контура";
                this.outlineFocused = "Цвет контура при фокусировке";
                this.borderPrimary = "Основной цвет границы";
                this.borderSecond = "Второстепенный цвет границы";
                this.borderFocused = "Цвет границы при фокусировке";
                this.backgroundPrimaryHovered = "Основной цвет фона при наведении курсора";
                this.backgroundSecondHovered = "Второстепенный цвет фона при наведении курсора";
                this.textPrimary = "Основной цвет текста";
                this.textSecond = "Второстепенный цвет текста";
                this.textThird = "Третий цвет текста";
                this.createTheme = "Создать тему";
                this.allowedThemeNameCharacters = "Разрешенные символы: a-z, A-Z, 0-9, -, _.";
                this.invalidThemeName = "Название темы недопустимо.";
                this.themeName = "Название темы:";
                this.compileSuccess = "Все файлы записаны успешно.\nПотрачено (в секундах):";
                this.themeButtonTitle = "Меню тем";
                this.openButtonTitle = "Открыть папку";
                this.loadingProject = "Загружаем проект...";
                this.selectedFolderMissing = "Выбранная папка не существует. Проект не будет инициализирован.";
                this.missingFileText =
                    "Текст выбранного файла отсутствует. Скорее всего, этот файл и/или его _trans версия отсутствуют.";
                this.cannotDetermineEngine = "Не удалось определить тип движка игры.";
                break;
            default:
                this.cannotGetSettings = "Cannot find program's settings.";
                this.askCreateSettings = "Create settings?";
                this.createdSettings =
                    "Settings created.\nBackups: Enabled\nBackup period: 60 secs.\nMaximum backups: 99.\nTheme: Cool Zinc";
                this.unsavedChanges = "You have unsaved changes. Save progress and quit?";
                this.originalTextIrreplacable = "Original text is irreplacable.";
                this.invalidRegexp = "Invalid regular expression.";
                this.textReverted = "Text was reverted to the original state";
                this.reloadButton = "Reload (F5)";
                this.helpButton = "Help";
                this.aboutButton = "About";
                this.hotkeysButton = "Hotkeys";
                this.exit = "Quit without saving?";
                this.fileMenu = "File";
                this.helpMenu = "Help";
                this.languageMenu = "Language";
                this.menuButtonTitle = "Tabs (Tab)";
                this.saveButtonTitle = "Save the translation files (Ctrl + S)";
                this.compileButtonTitle = "Compile to JSON (Alt + C)";
                this.optionsButtonTitle = "Options";
                this.searchButtonTitle = "Search (Ctrl + F)";
                this.searchInputTitle = "Search";
                this.replaceButtonTitle = "Replace all matches with";
                this.replaceInputTitle = "Replace";
                this.caseButtonTitle = "Consider case (Alt + C)";
                this.wholeButtonTitle = "Search the whole text (Alt + W)";
                this.regexButtonTitle = "Search by regular expressions (Alt + R)";
                this.translationButtonTitle = "Search only by translation (Alt + T)";
                this.locationButtonTitle = "Search only in the current file (Alt + L)";
                this.noMatches = "No matches";
                this.currentPage = "None";
                this.separator = "of";
                this.goToRow = "Go to row... from 1 to";
                this.missingTranslationDir = "'translation' directory is missing. Project won't be initialized.";
                this.missingOriginalDir = "'original' or 'data' directory is missing. Project won't be initialized.";
                this.missingTranslationSubdirs =
                    "'translation' directory's subdirectories 'maps', 'other' and/or 'plugins' are missing. Project won't be initialized.";
                this.noProjectSelected =
                    "No project selected. Select the project directory, using 'open folder' button in the left-top corner. Directory must contain directories 'original' or 'data' and 'translation' (with 'maps', 'other' and 'plugins' subdirectories, if it's RPG Maker MV/MZ, or 'maps' and 'other', if it's RPG Maker XP/VX/VXAce).";
                this.backgroundDark = "Dark background color";
                this.backgroundPrimary = "Primary background color";
                this.backgroundSecond = "Second background color";
                this.backgroundThird = "Third background color";
                this.outlinePrimary = "Primary outline color";
                this.outlineSecond = "Second outline color";
                this.outlineThird = "Third outline color";
                this.outlineFocused = "Focused outline color";
                this.borderPrimary = "Primary border color";
                this.borderSecond = "Second border color";
                this.borderFocused = "Focused border color";
                this.backgroundPrimaryHovered = "Primary background color when hovered";
                this.backgroundSecondHovered = "Second background color when hovered";
                this.textPrimary = "Primary text color";
                this.textSecond = "Second text color";
                this.textThird = "Third text color";
                this.createTheme = "Create theme";
                this.allowedThemeNameCharacters = "Allowed characters: a-z, A-Z, 0-9, -, _.";
                this.invalidThemeName = "Theme name is invalid.";
                this.themeName = "Theme name:";
                this.compileSuccess = "All files were written successfully.\nTime spent (in seconds):";
                this.themeButtonTitle = "Themes menu";
                this.openButtonTitle = "Open folder";
                this.loadingProject = "Loading project...";
                this.selectedFolderMissing = "Selected folder is missing. Project won't be initialized.";
                this.missingFileText =
                    "Text of the selected file missing. Probably, it and it's _trans version don't exist for some reason.";
                this.cannotDetermineEngine = "Cannot determine the type of the game's engine.";
                break;
        }
    }
}

export class OptionsLocalization {
    readonly backupPeriodLabel: string;
    readonly backupPeriodNote: string;
    readonly backupMaxLabel: string;
    readonly backupMaxNote: string;
    readonly backup: string;

    constructor(language: Language) {
        switch (language) {
            case Language.Russian:
                this.backupPeriodLabel = "Создавать резервные копии каждые:";
                this.backupPeriodNote = "секунд (минимум 60, максимум 3600)";
                this.backupMaxLabel = "Максимальное количество резервных копий:";
                this.backupMaxNote = "(минимум 1, максимум 99)";
                this.backup = "Резервное копирование";
                break;
            default:
                this.backupPeriodLabel = "Create backup every:";
                this.backupPeriodNote = "seconds (min 60, max 3600)";
                this.backupMaxLabel = "Max number of backups:";
                this.backupMaxNote = "(min 1, max 99)";
                this.backup = "Backup";
                break;
        }
    }
}

export class AboutLocalization {
    readonly version: string;
    readonly about: string;
    readonly socials: string;
    readonly vkLink: string;
    readonly tgLink: string;
    readonly githubLink: string;
    readonly license: string;

    constructor(language: Language) {
        switch (language) {
            case Language.Russian:
                this.version = "Версия";
                this.about = "Программа написана собственноручно мной.";
                this.socials = "Мои соцсети:";
                this.vkLink = "ВК";
                this.tgLink = "Телеграм";
                this.githubLink = "GitHub программы";
                this.license = "Лицензия";
                break;
            default:
                this.version = "Version";
                this.about = "Program is written all by myself.";
                this.socials = "My socials:";
                this.vkLink = "VK";
                this.tgLink = "Telegram";
                this.githubLink = "Program's GitHub";
                this.license = "License";
                break;
        }
    }
}

export class HotkeysLocalization {
    readonly hotkeysTitle: string;
    readonly hotkeys: string;

    constructor(language: Language) {
        switch (language) {
            case Language.Russian:
                this.hotkeysTitle = "Горячие клавиши";
                this.hotkeys =
                    "Цифры от 1 до + - Редактирование разных файлов<br>Tab - Открыть панель выбора файлов<br>Ctrl + S - Сохранить файлы перевода<br>Ctrl + F - Навестись на строку поиска<br>R - Открыть панель поиска<br>Alt + Enter - Перепрыгнуть на нижнее текстовое поле<br>Ctrl + Enter - Перепрыгнуть на верхнее текстовое поле<br>Esc - Если курсор сфокусирован на текстовом поле, убрать фокусировку на текстовом поле; иначе, выйти из состояния редактирования файлов<br>Alt + C, пока вы НЕ наведены на поле ввода текста - Скомпилировать в JSON файлы<br>Alt + C, Alt + W, Alt + R, Alt + L, Alt + T, пока вы наведены на поле ввода текста для поиска - переключить поиск по регистру, по словам целиком, по регулярным выражением, по текущему файлу, и только по переводу соответственно<br>ЛКМ на результате поиска - перейти к найденному элементу<br>ПКМ на результате поиска - заменить текст этого элемента на тот, что в данный момент введён в поле<br>ЛКМ на результате журнала - перейти к изменённому ранее элементу<br>ПКМ на результате журнала - вернуть изменённый элемент в предыдущее состояние<br>Ctrl + G - открыть поле для ввода числа линии, на которую вы хотите перейти";
                break;
            default:
                this.hotkeysTitle = "Hotkeys";
                this.hotkeys =
                    'Digits from "1" to "+" - Open different files to edit<br>Tab - Open the panel for selecting files<br>Ctrl + S - Save the translation files<br>Ctrl + F - Focus on the search field<br>R - Open the search panel<br>Alt + Enter - Jump to the textarea below the focused<br>Ctrl + Enter - Jump to the textarea above the focused<br>Esc - If focused element is the textarea, remove the focus, else exit from editing mode<br>Alt + C, if you are NOT focused on the textarea - Compile translation to JSON files<br>Alt + C, Alt + W, Alt + R, Alt + L, Alt + T, while you are focused on the search field - toggle case, whole text, regular expressions, only current file, and only translation text search respectively<br>LMB on the search result - scroll into the view of the result element<br>RMB on the search result - replace matching text of this element with the one that\'s currently entered into the replace field<br>LMB on the log result - scroll into the view of the earlier changed element<br>RMB on the log result - revert changed element to the original state<br>Ctrl + G - open the input field for jumping to specific row';
                break;
        }
    }
}

export class HelpLocalization {
    readonly helpTitle: string;
    readonly help: string;

    constructor(language: Language) {
        switch (language) {
            case Language.Russian:
                this.helpTitle = "Как пользоваться; что делать; куда нажимать?";
                this.help =
                    'Чтобы запустить проект и начать перевод, используйте кнопку "Открыть папку" в левом верхнем углу. Открываемая папка ДОЛЖНА содержать в себе папку data или original с оригинальными файлами игры. Если папка не содержит в себе распарсенный текст игры в папке translation, программа автоматически распарсит их. По умолчанию, программа поддерживает чтение и запись файлов движков RPG Maker MV/MZ, т. е. файлы в формате .json. Чтобы программа могла читать и записывать файлы движков RPG Maker XP/VX/VXAce (.rxdata, .rvdata и .rvdata2), вам необходимо установить Ruby, а затем установить rvpacker-txt используя gem install rvpacker-txt. Программа регулярно создает резервные копии файлов переводов по пути "папка_проекта/backups", период резервного копирования и максимальное количество резервных копий можно регулировать в настройках. По умолчанию программа создает резервные копии каждые 60 секунд.<br>При сохранении перевода в программе, все файлы перевода будут сохранятся по пути "папка_проекта/translation".<br>Чтобы сохранить изменения в проекте, просто нажмите кнопку сохранить или используйте сочетание клавиш Ctrl + S. Программа автоматически сохранит ваш проект, и когда вы снова запустите программу, вы сможете продолжить с того места, на котором остановились.<br>Чтобы скомпилировать ваш перевод в изначальные файлы (.json по умолчанию и .rxdata, .rvdata и .rvdata2 если у вас установлен Ruby и rvpacker-txt) игры, просто нажмите кнопку компиляции или используйте сочетание клавиш Alt + C. Обязательно сохраните перевод перед компиляцией. Программа создаст полностью функциональные файлы в папке "папка_проекта/output".<br>Вы можете еще раз прочитать это руководство или получить справку о горячих клавишах программы, выбрав пункты верхнего меню "О программе" и "Горячие клавиши" соответственно.';
                break;
            default:
                this.helpTitle = "How to use; what to do; where to click?";
                this.help =
                    'To start the project and start the translation, use the "Open Folder" button in the upper left corner. The folder to open MUST contain the data or original folder with the original game files. If the folder does not contain the parsed text of the game in the translation folder, the program will automatically parse it. By default, the program supports reading and writing RPG Maker MV/MZ engine files, i.e. files in the format .json. In order for the program to read and write RPG Maker XP/VX/VXAce engine files (.rxdata, .rvdata and .rvdata2), you need to install Ruby, and then install rvpacker-txt using gem install rvpacker-txt. The program regularly creates backups of translation files along the path "project_folder/backups", the backup period and the maximum number of backups can be adjusted in the settings. By default, the program creates backups every 60 seconds.<br>When saving a translation in the program, all translation files will be saved in the path "project_folder/translation".To save the changes in the project, just click save or use the keyboard shortcut Ctrl +S. The program will automatically save your project, and when you run the program again, you can pick up where you left off.<br>To compile your translation into the original files (by default .json and .rxdata, .rvdata and .rvdata2 if you have Ruby and rvpacker-txt installed) games, just click the compile button or use the Alt+C keyboard shortcut. Be sure to save the translation before compiling. The program will create fully functional files in the folder "project_folder/output".<br>You can read this manual again or get help about the program\'s hotkeys by selecting the "About" and "Hotkeys" items in the top menu, respectively.';
                break;
        }
    }
}
