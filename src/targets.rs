//! Определение целей очистки.
//!
//! Содержит структуру [`CleanTarget`] и фабричные функции для создания
//! списков целей по категориям: системные, Xcode и инструменты разработчика.

use crate::utils::get_home_path;
use std::path::PathBuf;

/// Цель очистки — директория на диске, содержимое которой можно удалить.
///
/// Каждая цель принадлежит к определённой категории (System, Xcode, Developer Tools)
/// и содержит человекочитаемое имя, путь и описание для пользователя.
///
/// Методы [`size()`](CleanTarget::size) и [`clean()`](CleanTarget::clean)
/// реализованы в модуле [`crate::cleaner`].
#[derive(Debug, Clone)]
pub struct CleanTarget {
    /// Категория цели ("System", "Xcode", "Developer Tools")
    pub category: String,
    /// Человекочитаемое название ("User Caches", "Xcode DerivedData" и т.д.)
    pub name: String,
    /// Абсолютный путь к директории на диске
    pub path: PathBuf,
    /// Краткое описание для отображения в таблице
    pub description: String,
}

impl CleanTarget {
    pub fn new(category: &str, name: &str, path: PathBuf, description: &str) -> Self {
        Self {
            category: category.to_string(),
            name: name.to_string(),
            path,
            description: description.to_string(),
        }
    }
}

/// Возвращает список системных целей очистки.
///
/// Включает: User Caches (`~/Library/Caches`), User Logs (`~/Library/Logs`),
/// Trash (`~/.Trash`).
pub fn get_general_targets() -> Vec<CleanTarget> {
    let home = get_home_path();
    let category = "System";

    vec![
        CleanTarget::new(
            category,
            "User Caches",
            home.join("Library/Caches"),
            "Application caches",
        ),
        CleanTarget::new(
            category,
            "User Logs",
            home.join("Library/Logs"),
            "Application and system logs",
        ),
        CleanTarget::new(
            category,
            "Trash",
            home.join(".Trash"),
            "Deleted files in Trash",
        ),
    ]
}

/// Возвращает список целей очистки Xcode.
///
/// Включает: Archives, DerivedData, iOS/watchOS DeviceSupport, CoreSimulator.
pub fn get_xcode_targets() -> Vec<CleanTarget> {
    let home = get_home_path();
    let category = "Xcode";

    vec![
        CleanTarget::new(
            category,
            "Xcode Archives",
            home.join("Library/Developer/Xcode/Archives"),
            "Old Xcode build archives",
        ),
        CleanTarget::new(
            category,
            "Xcode DerivedData",
            home.join("Library/Developer/Xcode/DerivedData"),
            "Xcode build cache",
        ),
        CleanTarget::new(
            category,
            "iOS DeviceSupport",
            home.join("Library/Developer/Xcode/iOS DeviceSupport"),
            "iOS device debug symbols",
        ),
        CleanTarget::new(
            category,
            "watchOS DevSupport",
            home.join("Library/Developer/Xcode/watchOS DeviceSupport"),
            "watchOS device debug symbols",
        ),
        CleanTarget::new(
            category,
            "CoreSimulator",
            home.join("Library/Developer/CoreSimulator/Devices"),
            "iOS Simulator data",
        ),
    ]
}

/// Возвращает список целей очистки инструментов разработчика.
///
/// Включает кэши: Homebrew, CocoaPods, Yarn, NPM, Gradle.
pub fn get_dev_targets() -> Vec<CleanTarget> {
    let home = get_home_path();
    let category = "Developer Tools";

    vec![
        CleanTarget::new(
            category,
            "Homebrew Cache",
            home.join("Library/Caches/Homebrew"),
            "Homebrew package cache",
        ),
        CleanTarget::new(
            category,
            "CocoaPods Cache",
            home.join("Library/Caches/CocoaPods"),
            "CocoaPods package cache",
        ),
        CleanTarget::new(
            category,
            "Yarn Cache",
            home.join("Library/Caches/Yarn"),
            "Yarn package cache",
        ),
        CleanTarget::new(
            category,
            "NPM Cache",
            home.join(".npm"),
            "NPM package cache",
        ),
        CleanTarget::new(
            category,
            "Gradle Cache",
            home.join(".gradle/caches"),
            "Gradle package cache",
        ),
    ]
}
