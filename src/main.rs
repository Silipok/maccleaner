//! # MacCleaner
//!
//! Консольная утилита для очистки диска на macOS.
//!
//! Сканирует и безопасно удаляет кэши, логи, артефакты Xcode
//! и кэши инструментов разработчика (Homebrew, CocoaPods, Yarn, NPM, Gradle).
//!
//! ## Архитектура
//!
//! - [`targets`] — определение целей очистки (`CleanTarget`)
//! - [`cleaner`] — логика сканирования и удаления
//! - [`ui`] — пользовательский интерфейс (меню, баннер, промпты)
//! - [`utils`] — утилиты (параллельный подсчёт размера, домашний путь)

#![deny(clippy::all)]

mod cleaner;
mod targets;
mod ui;
mod utils;

use bytesize::ByteSize;
use colored::*;
use std::io;

use cleaner::{clean_targets, scan_targets};
use targets::{get_dev_targets, get_general_targets, get_xcode_targets, CleanTarget};
use ui::{
    clear_screen, is_root, print_banner, print_menu, prompt_clean_targets, restart_with_sudo,
    wait_for_enter,
};

/// Сканирует все категории (System, Xcode, Developer Tools) и выводит общий итог.
///
/// Не удаляет файлы — только подсчитывает размеры и выводит таблицу.
fn scan_all() {
    let general = get_general_targets();
    let xcode = get_xcode_targets();
    let dev = get_dev_targets();

    let general_size = scan_targets(&general, "System (Caches, Logs, Trash)");
    let xcode_size = scan_targets(&xcode, "Xcode");
    let dev_size = scan_targets(&dev, "Developer Tools");

    println!("\n{}", "=".repeat(100));
    println!(
        "{}  {}",
        "GRAND TOTAL RECLAIMABLE:".bold(),
        ByteSize(general_size + xcode_size + dev_size)
            .to_string()
            .green()
            .bold()
    );
}

/// Интерактивный режим очистки.
///
/// Собирает все цели из всех категорий, сканирует их размеры,
/// затем показывает пользователю интерактивный checkbox-список
/// (через [`dialoguer::MultiSelect`]). Пользователь выбирает
/// конкретные цели, после чего выполняется очистка с подтверждением.
fn interactive_clean() {
    let mut all_targets = Vec::new();
    all_targets.extend(get_general_targets());
    all_targets.extend(get_xcode_targets());
    all_targets.extend(get_dev_targets());

    println!("{}", "Scanning all targets...".blue());

    // Scan sizes
    let mut options = Vec::new();
    for target in &all_targets {
        // We calculate size here. Note: this might take a moment.
        options.push((target, target.size()));
    }

    let selected_indices = prompt_clean_targets(&options);

    if selected_indices.is_empty() {
        println!("{}", "No items selected.".yellow());
        return;
    }

    let selected_targets: Vec<CleanTarget> = selected_indices
        .into_iter()
        .map(|i| options[i].0.clone())
        .collect();

    // Confirm and clean
    clean_targets(&selected_targets, "Selected Items");
}

/// Быстрая очистка системных кэшей, логов и Корзины.
fn clean_system() {
    let targets = get_general_targets();
    clean_targets(&targets, "System (Caches, Logs, Trash)");
}

/// Очистка артефактов Xcode (DerivedData, Archives, DeviceSupport, CoreSimulator).
fn clean_xcode() {
    let targets = get_xcode_targets();
    clean_targets(&targets, "Xcode");
}

/// Очистка кэшей инструментов разработчика (Homebrew, CocoaPods, Yarn, NPM, Gradle).
fn clean_dev() {
    let targets = get_dev_targets();
    clean_targets(&targets, "Developer Tools");
}

/// Точка входа.
///
/// Запускает бесконечный цикл главного меню:
/// очистка экрана → баннер → меню → чтение ввода → вызов функции.
/// Цикл завершается при выборе "Exit" или ошибке ввода.
fn main() {
    loop {
        clear_screen();
        print_banner();
        print_menu();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("{}", "Failed to read input. Exiting.".red());
            break;
        }

        match input.trim() {
            "1" => {
                interactive_clean();
                wait_for_enter();
            }
            "2" => {
                scan_all();
                wait_for_enter();
            }
            "3" => {
                clean_system();
                wait_for_enter();
            }
            "4" => {
                clean_xcode();
                wait_for_enter();
            }
            "5" => {
                clean_dev();
                wait_for_enter();
            }
            "6" if !is_root() => {
                restart_with_sudo();
                wait_for_enter();
            }
            "6" if is_root() => {
                println!("\n{}", "Goodbye!".green().bold());
                break;
            }
            "7" | "q" | "Q" if !is_root() => {
                println!("\n{}", "Goodbye!".green().bold());
                break;
            }
            _ => {
                println!("\n{}", "Invalid choice. Please try again.".red());
                wait_for_enter();
            }
        }
    }
}
