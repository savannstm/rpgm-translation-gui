import { Language } from "../types/enums";

export class MainWindowLocalization {
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
    readonly compileWindowTitle: string;
    readonly readWindowTitle: string;
    readonly bookmarksButtonTitle: string;
    readonly readButtonTitle: string;
    readonly directoryAlreadyOpened: string;

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
                this.compileButtonTitle = "Скомпилировать (Alt + C)";
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
                this.missingOriginalDir = 'Директория "data" отсутствует. Проект не будет инициализирован.';
                this.missingTranslationSubdirs =
                    'Поддиректории "maps" и "other" директории "translation" отсутствуют. Проект не будет инициализирован.';
                this.noProjectSelected =
                    'Проект не выбран. Выберите директорию проекта, используя кнопку "открыть папку" в левом верхнем углу. Директория должна содержать в себе папки "data" и "translation" (с поддиректориями "maps", "other" и "plugins", если это RPG Maker MV/MZ; или "maps" и "other", если это RPG Maker XP/VX/VXAce).';
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
                this.loadingProject = "Загружаем проект";
                this.selectedFolderMissing = "Выбранная папка не существует. Проект не будет инициализирован.";
                this.missingFileText =
                    "Текст выбранного файла отсутствует. Скорее всего, этот файл и/или его _trans версия отсутствуют.";
                this.cannotDetermineEngine = "Не удалось определить тип движка игры.";
                this.compileWindowTitle = "Настройки компиляции";
                this.readWindowTitle = "Настройки чтения";
                this.bookmarksButtonTitle = "Закладки (Ctrl + B)";
                this.readButtonTitle = "Перечитать файлы (Alt + R)";
                this.directoryAlreadyOpened = "Выбранная директория уже открыта в программе.";
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
                this.compileButtonTitle = "Compile (Alt + C)";
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
                this.missingOriginalDir = "'data' directory is missing. Project won't be initialized.";
                this.missingTranslationSubdirs =
                    "'translation' directory's subdirectories 'maps', 'other' and/or 'plugins' are missing. Project won't be initialized.";
                this.noProjectSelected =
                    "No project selected. Select the project directory, using 'open folder' button in the left-top corner. Directory must contain directories  'data' and 'translation' (with 'maps', 'other' and 'plugins' subdirectories, if it's RPG Maker MV/MZ; or 'maps' and 'other', if it's RPG Maker XP/VX/VXAce).";
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
                this.loadingProject = "Loading project";
                this.selectedFolderMissing = "Selected folder is missing. Project won't be initialized.";
                this.missingFileText =
                    "Text of the selected file missing. Probably, it and it's _trans version don't exist for some reason.";
                this.cannotDetermineEngine = "Cannot determine the type of the game's engine.";
                this.compileWindowTitle = "Compilation settings";
                this.readWindowTitle = "Read settings";
                this.bookmarksButtonTitle = "Bookmarks (Ctrl + B)";
                this.readButtonTitle = "Re-read files (Alt + R)";
                this.directoryAlreadyOpened = "Selected directory is already opened in the program.";
                break;
        }
    }
}

export class SettingsWindowLocalization {
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

export class AboutWindowLocalization {
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

export class HelpWindowLocalization {
    readonly helpTitle: string;
    readonly help: string;
    readonly hotkeysTitle: string;
    readonly hotkeys: string;

    constructor(language: Language) {
        switch (language) {
            case Language.Russian:
                this.helpTitle = "Как пользоваться; что делать; куда нажимать?";
                this.help =
                    'Чтобы запустить проект и начать перевод, используйте кнопку "Открыть папку" в левом верхнем углу. Открываемая папка должна содержать в себе папку "data" с оригинальными файлами игры. Если текст игры не был распарсен ранее с использованием CLI-инструментов, прогрмма автоматически распарсит его. По умолчанию, программа поддерживает чтение и запись файлов движков RPG Maker XP/VX/VXAce/MV/MZ.<br>Программа регулярно создает резервные копии файлов переводов по пути "папка_проекта/.rpgm-translation-gui/backups", период резервного копирования и максимальное количество резервных копий можно регулировать в настройках. По умолчанию программа создает резервные копии каждые 60 секунд.<br>При сохранении перевода в программе, все файлы перевода будут сохранятся по пути "папка_проекта/.rpgm-translation-gui/translation".<br>Чтобы сохранить изменения в проекте, нажмите кнопку сохранить или используйте сочетание клавиш Ctrl + S. Программа автоматически сохранит ваш проект, и когда вы снова запустите программу, вы сможете продолжить с того места, на котором остановились.<br>Чтобы скомпилировать ваш перевод в рабочие файлы игры, нажмите кнопку компиляции или используйте сочетание клавиш Alt + C. Обязательно сохраните перевод перед компиляцией. Программа создаст полностью функциональные файлы игры с применённым переводом в папке "папка_проекта/.rpgm-translation-gui/output".<br>Вы можете еще раз прочитать это руководство или получить справку о горячих клавишах программы, выбрав пункты верхнего меню "Помощь" > "Помощь".';
                this.hotkeysTitle = "Горячие клавиши";
                this.hotkeys =
                    "Цифры от 1 до + - Редактирование разных файлов<br>Tab - Открыть панель выбора файлов<br>Ctrl + S - Сохранить файлы перевода<br>Ctrl + F - Навестись на строку поиска<br>R - Открыть панель поиска<br>Alt + Enter - Перепрыгнуть на нижнее текстовое поле<br>Ctrl + Enter - Перепрыгнуть на верхнее текстовое поле<br>Esc - Если курсор сфокусирован на текстовом поле, убрать фокусировку на текстовом поле; иначе, выйти из состояния редактирования файлов<br>Alt + C, пока вы НЕ наведены на поле ввода текста - Скомпилировать<br>Alt + C, Alt + W, Alt + R, Alt + L, Alt + T, пока вы наведены на поле ввода текста для поиска - переключить поиск по регистру, по словам целиком, по регулярным выражением, по текущему файлу, и только по переводу соответственно<br>ЛКМ на результате поиска - перейти к найденному элементу<br>ПКМ на результате поиска - заменить текст этого элемента на тот, что в данный момент введён в поле<br>ЛКМ на результате журнала - перейти к изменённому ранее элементу<br>ПКМ на результате журнала - вернуть изменённый элемент в предыдущее состояние<br>Ctrl + G - открыть поле для ввода числа линии, на которую вы хотите перейти<br>Ctrl + B - открыть окно закладок<br>Ctrl + R - открыть окно чтения файлов";
                break;
            default:
                this.helpTitle = "How to use; what to do; where to click?";
                this.help =
                    "To start the project and start the translation, use the 'Open Folder' button in the upper left corner. The folder must contain the 'data' folder with the original game files. If the text of the game has not been parsed previously using CLI tools, the program will automatically parse it. By default, the program supports reading and writing RPG Maker XP/VX/VXAce/MV/MZ engine files.<br>The program regularly creates backups of translation files in the path 'project_folder/.rpgm-translation-gui/backups', the backup period and the maximum number of backups can be adjusted in the settings. By default, the program creates backups every 60 seconds.<br>When saving a translation in the program, all translation files will be saved in the path 'project_folder/.rpgm-translation-gui/translation'. To save the changes in the project, click save or use the keyboard shortcut Ctrl + S. The program will automatically save your project, and when you run the program again, you can pick up where you left off. To compile your translation into working game files, click the compile button or use the Alt + C keyboard shortcut. Be sure to save the translation before compiling. The program will create fully functional game files with the applied translation in the folder 'project_folder/.rpgm-translation-gui/output'.<br>You can read this manual again or get help about the program's hotkeys by selecting the 'Help' > 'Help' item in the top menu.";
                this.hotkeysTitle = "Hotkeys";
                this.hotkeys =
                    'Digits from "1" to "+" - Open different files to edit<br>Tab - Open the panel for selecting files<br>Ctrl + S - Save the translation files<br>Ctrl + F - Focus on the search field<br>R - Open the search panel<br>Alt + Enter - Jump to the textarea below the focused<br>Ctrl + Enter - Jump to the textarea above the focused<br>Esc - If focused element is the textarea, remove the focus, else exit from editing mode<br>Alt + C, if you are NOT focused on the textarea - Compile translation<br>Alt + C, Alt + W, Alt + R, Alt + L, Alt + T, while you are focused on the search field - toggle case, whole text, regular expressions, only current file, and only translation text search respectively<br>LMB on the search result - scroll into the view of the result element<br>RMB on the search result - replace matching text of this element with the one that\'s currently entered into the replace field<br>LMB on the log result - scroll into the view of the earlier changed element<br>RMB on the log result - revert changed element to the original state<br>Ctrl + G - open the input field for jumping to specific row<br>Ctrl + B - open the bookmarks window<br>Ctrl + R - open the files reading window';
                break;
        }
    }
}

export class CompileWindowLocalization {
    readonly options: string;
    readonly enableLoggingOption: string;
    readonly romanizeOption: string;
    readonly shuffleOption: string;
    readonly shuffleLevel: string;
    readonly chooseOptionText: string;
    readonly shuffleLinesOption: string;
    readonly shuffleAllOption: string;
    readonly disableCustomParsing: string;
    readonly customOutputPath: string;
    readonly selectOutputPath: string;
    readonly disableProcessing: string;
    readonly disableMapsProcessingOption: string;
    readonly disableOtherProcessingOption: string;
    readonly disableSystemProcessingOption: string;
    readonly disablePluginsProcessingOption: string;
    readonly dontAskAgain: string;
    readonly compileButtonText: string;

    constructor(language: Language) {
        switch (language) {
            case Language.Russian:
                this.options = "Опции:";
                this.enableLoggingOption = "Включить логирование (в текущий момент не имеет функционала)";
                this.romanizeOption =
                    "Романизация игрового текста. Используйте эту опцию, лишь если вы прочитали текст с её использованием, чтобы корректно записать все файлы.";
                this.shuffleOption = "Перемешивание";
                this.shuffleLevel = "Уровень перемешивания";
                this.chooseOptionText = "Выберите опцию";
                this.shuffleLinesOption = "Перемешать линии в строках";
                this.shuffleAllOption = "Перемешать линии и слова";
                this.disableCustomParsing =
                    "Выключить индивидуальную обработку (используйте лишь если вы прочитали файлы без индивидуальной обработки)";
                this.customOutputPath = "Другой выходной путь";
                this.selectOutputPath = "Выбрать выходной путь";
                this.disableProcessing = "Выключить обработку...";
                this.disableMapsProcessingOption = "Выключить обработку файлов maps";
                this.disableOtherProcessingOption = "Выключить обработку файлов other";
                this.disableSystemProcessingOption = "Выключить обработку файла system";
                this.disablePluginsProcessingOption = "Выключить обработку файла plugins/scripts";
                this.dontAskAgain =
                    "Больше не спрашивать (вы можете вновь открыть это окно двойным нажатием по кнопке компиляции)";
                this.compileButtonText = "Скомпилировать";
                break;
            default:
                this.options = "Options:";
                this.enableLoggingOption = "Enable logging (currently does nothing)";
                this.romanizeOption =
                    "Whether to romanize text. Only use this option if you've read text with it, to correctly write all files.";
                this.shuffleOption = "Shuffle";
                this.shuffleLevel = "Shuffle level";
                this.chooseOptionText = "Choose an option";
                this.shuffleLinesOption = "Shuffle text lines";
                this.shuffleAllOption = "Shuffle both lines and words";
                this.disableCustomParsing =
                    "Disable custom parsing (use only if you've read files without custom parsing)";
                this.customOutputPath = "Custom output path";
                this.selectOutputPath = "Select output path";
                this.disableProcessing = "Disable processing of...";
                this.disableMapsProcessingOption = "Disable maps processing";
                this.disableOtherProcessingOption = "Disable other processing";
                this.disableSystemProcessingOption = "Disable system processing";
                this.disablePluginsProcessingOption = "Disable plugins/scripts processing";
                this.dontAskAgain =
                    "Don't ask again (you can open this window again by double-clicking compile button)";
                this.compileButtonText = "Compile";
                break;
        }
    }
}

export class ReadWindowLocalization {
    readonly mode: string;
    readonly chooseReadingMode: string;
    readonly appendReadMode: string;
    readonly forceReadMode: string;
    readonly appendModeDescription: string;
    readonly forceModeDescription: string;
    readonly options: string;
    readonly enableLoggingOption: string;
    readonly romanizeOption: string;
    readonly disableCustomParsing: string;
    readonly disableProcessing: string;
    readonly disableMapsProcessingOption: string;
    readonly disableOtherProcessingOption: string;
    readonly disableSystemProcessingOption: string;
    readonly disablePluginsProcessingOption: string;
    readonly dontAskAgain: string;
    readonly readButtonText: string;
    readonly readingInAppendMode: string;
    readonly readingInForceMode: string;

    constructor(language: Language) {
        switch (language) {
            case Language.Russian:
                this.mode = "Режим чтения:";
                this.chooseReadingMode = "Выберите режим чтения";
                this.appendReadMode = "Добавление";
                this.forceReadMode = "Перезапись";
                this.appendModeDescription =
                    "В случае обновления игры, текст которой вы запарсили, либо же графического интерфейса, имеет смысл перечитать файлы в этом режиме, чтобы добавить новый текст к имеющемуся без потери прогресса.";
                this.forceModeDescription =
                    "Принудительно перезаписывает файлы перевода. Используйте, если вам нужно полностью перечитать файлы с определёнными настройками.";
                this.options = "Опции:";
                this.enableLoggingOption = "Включить логирование (в текущий момент не имеет функционала)";
                this.romanizeOption =
                    'Романизация текста. Если вы парсите текст из японской игры, содержащей символы вроде 「」,являющимися обычными японскими кавычками, они будут автоматически заменены на их европейские эквиваленты. (в данном случае, "")';
                this.disableCustomParsing =
                    "Выключить индивидуальную обработку (используйте лишь если вы прочитали файлы без индивидуальной обработки)";
                this.disableProcessing = "Выключить обработку...";
                this.disableMapsProcessingOption = "Выключить обработку файлов maps";
                this.disableOtherProcessingOption = "Выключить обработку файлов other";
                this.disableSystemProcessingOption = "Выключить обработку файла system";
                this.disablePluginsProcessingOption = "Выключить обработку файла plugins/scripts";
                this.dontAskAgain =
                    "Больше не спрашивать (вы можете вновь открыть это окно двойным нажатием по кнопке компиляции)";
                this.readButtonText = "Прочитать";
                this.readingInAppendMode = "Читаем в режиме добавления";
                this.readingInForceMode = "Читаем в режиме принудительной перезаписи";
                break;
            default:
                this.mode = "Reading mode:";
                this.chooseReadingMode = "Choose reading mode";
                this.appendReadMode = "Append";
                this.forceReadMode = "Force rewrite";
                this.appendModeDescription =
                    "In case, when the game text you've parsed updates, or the GUI update, it makes sense to re-read files in this mode, to append new text to existing translation without overwriting the progress.";
                this.forceModeDescription =
                    "Forcefully rewrites translation files. Use, only if you need to completely re-read files using certain settings.";
                this.options = "Options:";
                this.enableLoggingOption = "Enable logging (currently does nothing)";
                this.romanizeOption =
                    'Whether to romanize text. If you parsing text from a Japanese game, that contains symbols like 「」, which are just the Japanese quotation marks, it automatically replaces these symbols by their roman equivalents. (in this case, "")';
                this.disableCustomParsing =
                    "Disable custom parsing (use only if you've read files without custom parsing)";
                this.disableProcessing = "Disable processing of...";
                this.disableMapsProcessingOption = "Disable maps processing";
                this.disableOtherProcessingOption = "Disable other processing";
                this.disableSystemProcessingOption = "Disable system processing";
                this.disablePluginsProcessingOption = "Disable plugins/scripts processing";
                this.dontAskAgain =
                    "Don't ask again (you can open this window again by double-clicking compile button)";
                this.readButtonText = "Read";
                this.readingInAppendMode = "Reading in append mode";
                this.readingInForceMode = "Reading in force rewrite mode";
                break;
        }
    }
}

export type Localization =
    | MainWindowLocalization
    | HelpWindowLocalization
    | AboutWindowLocalization
    | ReadWindowLocalization
    | CompileWindowLocalization
    | SettingsWindowLocalization;
