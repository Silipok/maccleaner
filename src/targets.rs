//! Определение целей очистки.
//!
//! Содержит структуру [`CleanTarget`], перечисления уровней риска и типов очистки,
//! а также фабричные функции для создания списков целей по категориям.

use crate::utils::get_home_path;
use std::path::PathBuf;

/// Уровень риска цели очистки.
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "LOW"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::High => write!(f, "HIGH"),
        }
    }
}

/// Способ очистки цели.
#[derive(Debug, Clone)]
pub enum TargetType {
    /// Удаляется содержимое директории, сама директория сохраняется
    Directory,
    /// Удаляется директория/файл целиком
    DirectoryFull,
    /// Очистка через внешнюю команду
    Command {
        program: String,
        args: Vec<String>,
    },
}

/// Цель очистки — директория или команда для освобождения места.
#[derive(Debug, Clone)]
pub struct CleanTarget {
    pub category: String,
    pub name: String,
    pub path: PathBuf,
    pub description: String,
    pub explanation: String,
    pub requires_sudo: bool,
    pub risk_level: RiskLevel,
    pub target_type: TargetType,
}

impl CleanTarget {
    pub fn new(category: &str, name: &str, path: PathBuf, description: &str) -> Self {
        Self {
            category: category.to_string(),
            name: name.to_string(),
            path,
            description: description.to_string(),
            explanation: String::new(),
            requires_sudo: false,
            risk_level: RiskLevel::Low,
            target_type: TargetType::Directory,
        }
    }

    pub fn with_explanation(mut self, text: &str) -> Self {
        self.explanation = text.to_string();
        self
    }

    pub fn with_sudo(mut self) -> Self {
        self.requires_sudo = true;
        self
    }

    pub fn with_risk(mut self, level: RiskLevel) -> Self {
        self.risk_level = level;
        self
    }

    pub fn with_command(mut self, program: &str, args: &[&str]) -> Self {
        self.target_type = TargetType::Command {
            program: program.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        };
        self
    }

    pub fn full_delete(mut self) -> Self {
        self.target_type = TargetType::DirectoryFull;
        self
    }
}

// ─── System (кэши, логи, корзина) ───────────────────────────────────────────

pub fn get_general_targets() -> Vec<CleanTarget> {
    let home = get_home_path();
    let category = "System";

    vec![
        CleanTarget::new(category, "User Caches", home.join("Library/Caches"), "Application caches")
            .with_explanation("Кэши пользовательских приложений (Chrome, Slack, Spotify и др.).\nБезопасно — приложения пересоздадут кэши. Может замедлить первый запуск."),
        CleanTarget::new(category, "User Logs", home.join("Library/Logs"), "Application and system logs")
            .with_explanation("Логи пользовательских приложений. Безопасно — новые создадутся автоматически."),
        CleanTarget::new(category, "Trash", home.join(".Trash"), "Deleted files in Trash")
            .with_risk(RiskLevel::Medium)
            .with_explanation("Содержимое Корзины macOS. После очистки файлы НЕВОЗМОЖНО восстановить!"),
    ]
}

// ─── Xcode ──────────────────────────────────────────────────────────────────

pub fn get_xcode_targets() -> Vec<CleanTarget> {
    let home = get_home_path();
    let category = "Xcode";

    vec![
        CleanTarget::new(category, "Xcode Archives", home.join("Library/Developer/Xcode/Archives"), "Old Xcode build archives")
            .with_explanation("Архивы сборок для App Store. Если уже загружены — не нужны. Сотни МБ каждый."),
        CleanTarget::new(category, "Xcode DerivedData", home.join("Library/Developer/Xcode/DerivedData"), "Xcode build cache")
            .with_explanation("Кэш сборки Xcode. Часто 10-50 GB! Безопасно — Xcode пересоберёт при необходимости."),
        CleanTarget::new(category, "iOS DeviceSupport", home.join("Library/Developer/Xcode/iOS DeviceSupport"), "iOS device debug symbols")
            .with_explanation("Debug-символы iOS устройств. ~2-5 GB на версию. Скачаются заново при подключении."),
        CleanTarget::new(category, "watchOS DevSupport", home.join("Library/Developer/Xcode/watchOS DeviceSupport"), "watchOS device debug symbols")
            .with_explanation("Debug-символы watchOS. Скачаются заново при необходимости."),
        CleanTarget::new(category, "CoreSimulator", home.join("Library/Developer/CoreSimulator/Devices"), "iOS Simulator data")
            .with_risk(RiskLevel::Medium)
            .with_explanation("Данные iOS симуляторов. Безопасно, но установленные в симулятор приложения будут потеряны."),
    ]
}

// ─── Developer Tools ────────────────────────────────────────────────────────

pub fn get_dev_targets() -> Vec<CleanTarget> {
    let home = get_home_path();
    let category = "Developer Tools";

    vec![
        CleanTarget::new(category, "Homebrew Cache", home.join("Library/Caches/Homebrew"), "Homebrew package cache")
            .with_explanation("Скачанные пакеты Homebrew. Безопасно — скачаются заново."),
        CleanTarget::new(category, "CocoaPods Cache", home.join("Library/Caches/CocoaPods"), "CocoaPods package cache")
            .with_explanation("Кэш CocoaPods. Безопасно — скачается при pod install."),
        CleanTarget::new(category, "Yarn Cache", home.join("Library/Caches/Yarn"), "Yarn package cache")
            .with_explanation("Кэш Yarn. Безопасно — пакеты скачаются заново."),
        CleanTarget::new(category, "NPM Cache", home.join(".npm"), "NPM package cache")
            .with_explanation("Кэш NPM. Безопасно — скачается при npm install."),
        CleanTarget::new(category, "Gradle Cache", home.join(".gradle/caches"), "Gradle package cache")
            .with_explanation("Кэш Gradle (Android). Безопасно — скачается при следующей сборке."),
    ]
}

// ─── System Data (НОВОЕ) ────────────────────────────────────────────────────

pub fn get_system_data_targets() -> Vec<CleanTarget> {
    let home = get_home_path();
    let category = "System Data";

    vec![
        CleanTarget::new(category, "System Caches", PathBuf::from("/Library/Caches"), "System-wide application caches")
            .with_sudo()
            .with_explanation("Кэши системных демонов и приложений. Безопасно — пересоздадутся автоматически.\nТребует прав администратора."),
        CleanTarget::new(category, "System Logs", PathBuf::from("/private/var/log"), "System log files")
            .with_sudo()
            .with_explanation("Системные логи macOS: ядро, демоны, установщики.\nБезопасно — новые создадутся автоматически."),
        CleanTarget::new(category, "System Temp Files", PathBuf::from("/private/var/tmp"), "Temporary system files")
            .with_sudo()
            .with_explanation("Временные файлы системных процессов. Обычно безопасно удалять."),
        CleanTarget::new(category, "Crash Reports", home.join("Library/Logs/DiagnosticReports"), "User crash reports")
            .with_explanation("Отчёты о крашах приложений. Полезны для отладки, но обычно не нужны."),
        CleanTarget::new(category, "System Crash Reports", PathBuf::from("/Library/Logs/DiagnosticReports"), "System crash reports")
            .with_sudo()
            .with_explanation("Системные отчёты о крашах. Безопасно удалять."),
        CleanTarget::new(category, "Saved App State", home.join("Library/Saved Application State"), "Saved window positions")
            .with_explanation("Сохранённое состояние окон (позиция, размер). Приложения откроются с настройками по умолчанию."),
        CleanTarget::new(category, "Mail Downloads", home.join("Library/Containers/com.apple.mail/Data/Library/Mail Downloads"), "Downloaded attachments")
            .with_explanation("Скачанные вложения из писем Mail.app. Оригиналы на сервере."),
        CleanTarget::new(category, "QuickLook Cache", home.join("Library/Caches/com.apple.QuickLook.thumbnailcache"), "Preview thumbnails")
            .with_explanation("Кэш миниатюр Quick Look. Пересоздаётся автоматически."),
        CleanTarget::new(category, "Diagnostics Data", PathBuf::from("/private/var/db/diagnostics"), "System diagnostics")
            .with_sudo()
            .with_risk(RiskLevel::Medium)
            .with_explanation("Данные unified logging macOS. Могут занимать несколько ГБ.\nУдаление безопасно, но затрудняет диагностику через Console.app."),
        CleanTarget::new(category, "Software Updates", PathBuf::from("/Library/Updates"), "Pending macOS updates")
            .with_sudo()
            .with_explanation("Скачанные обновления macOS. Безопасно — скачаются заново из App Store."),
    ]
}

// ─── Browsers (НОВОЕ) ──────────────────────────────────────────────────────

pub fn get_browser_targets() -> Vec<CleanTarget> {
    let home = get_home_path();
    let category = "Browsers";

    vec![
        CleanTarget::new(category, "Chrome Cache", home.join("Library/Caches/Google/Chrome"), "Chrome browser cache")
            .with_explanation("Кэш Chrome. Безопасно — страницы загрузятся заново.\nЛогины, закладки и пароли НЕ затрагиваются."),
        CleanTarget::new(category, "Chrome SW Cache", home.join("Library/Application Support/Google/Chrome/Default/Service Worker/CacheStorage"), "Chrome offline data")
            .with_explanation("Кэш Service Worker'ов (оффлайн-данные PWA). Безопасно, но PWA потеряют оффлайн-доступ."),
        CleanTarget::new(category, "Safari Cache", home.join("Library/Caches/com.apple.Safari"), "Safari browser cache")
            .with_explanation("Кэш Safari. Безопасно — данные загрузятся заново."),
        CleanTarget::new(category, "Firefox Cache", home.join("Library/Caches/Firefox"), "Firefox browser cache")
            .with_explanation("Кэш Firefox. Безопасно — страницы загрузятся заново."),
        CleanTarget::new(category, "Edge Cache", home.join("Library/Caches/Microsoft Edge"), "Edge browser cache")
            .with_explanation("Кэш Microsoft Edge. Безопасно удалять."),
    ]
}

// ─── Docker (НОВОЕ) ─────────────────────────────────────────────────────────

pub fn get_docker_targets() -> Vec<CleanTarget> {
    let home = get_home_path();
    let category = "Docker";

    vec![
        CleanTarget::new(category, "Docker System Prune", home.join("Library/Containers/com.docker.docker"), "All Docker data")
            .with_risk(RiskLevel::High)
            .with_command("docker", &["system", "prune", "-af", "--volumes"])
            .with_explanation(
                "⚠️  АГРЕССИВНАЯ ОЧИСТКА DOCKER!\n\n\
                 Удаляет ВСЁ:\n\
                 • Все остановленные контейнеры\n\
                 • Все неиспользуемые сети\n\
                 • ВСЕ образы (images) — придётся скачивать заново\n\
                 • ВСЕ тома (volumes) — ДАННЫЕ БУДУТ ПОТЕРЯНЫ!\n\n\
                 После очистки: docker pull, docker-compose up заново.\n\
                 Используйте только если в volumes нет важных данных."
            ),
    ]
}

// ─── macOS (НОВОЕ) ──────────────────────────────────────────────────────────

pub fn get_macos_targets() -> Vec<CleanTarget> {
    let category = "macOS";
    let mut targets = Vec::new();

    // Сканирование /Applications для старых установщиков macOS
    if let Ok(entries) = std::fs::read_dir("/Applications") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("Install macOS") && name.ends_with(".app") {
                targets.push(
                    CleanTarget::new(
                        category,
                        &format!("Installer: {}", name.trim_end_matches(".app")),
                        entry.path(),
                        "Old macOS installer (12-15 GB)",
                    )
                    .full_delete()
                    .with_risk(RiskLevel::Medium)
                    .with_explanation(&format!(
                        "Установщик: {}. Занимает 12-15 GB.\n\
                         Можно удалить если не планируете переустановку.\n\
                         Скачивается заново через App Store.",
                        name
                    )),
                );
            }
        }
    }

    targets
}
