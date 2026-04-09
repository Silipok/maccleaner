//! # MacCleaner
//!
//! Консольная утилита для агрессивной очистки диска на macOS.
//!
//! Сканирует и безопасно удаляет кэши, логи, артефакты Xcode,
//! кэши dev-инструментов, System Data, данные Docker и браузеров.

#![deny(clippy::all)]

mod cleaner;
mod targets;
mod ui;
mod utils;

use bytesize::ByteSize;
use colored::*;
use std::io;

use cleaner::{clean_targets, scan_targets};
use targets::{
    get_browser_targets, get_dev_targets, get_docker_targets, get_general_targets,
    get_macos_targets, get_system_data_targets, get_xcode_targets, CleanTarget,
};
use ui::{
    clear_screen, is_root, print_banner, print_menu, prompt_clean_targets, restart_with_sudo,
    wait_for_enter,
};

/// Сканирует ВСЕ категории и выводит общий итог.
fn scan_all() {
    let categories: Vec<(Vec<CleanTarget>, &str)> = vec![
        (get_general_targets(), "System (Caches, Logs, Trash)"),
        (get_xcode_targets(), "Xcode"),
        (get_dev_targets(), "Developer Tools"),
        (get_system_data_targets(), "System Data"),
        (get_browser_targets(), "Browsers"),
        (get_docker_targets(), "Docker"),
        (get_macos_targets(), "macOS"),
    ];

    let mut grand_total = 0u64;

    for (targets, title) in &categories {
        if targets.is_empty() {
            continue;
        }
        let size = scan_targets(targets, title);
        grand_total += size;
    }

    println!("\n{}", "═".repeat(100));
    println!(
        "{}  {}",
        "GRAND TOTAL RECLAIMABLE:".bold(),
        ByteSize(grand_total).to_string().green().bold()
    );
}

/// Интерактивная очистка — выбор из ВСЕХ категорий.
fn interactive_clean() {
    let mut all_targets = Vec::new();
    all_targets.extend(get_general_targets());
    all_targets.extend(get_xcode_targets());
    all_targets.extend(get_dev_targets());
    all_targets.extend(get_system_data_targets());
    all_targets.extend(get_browser_targets());
    all_targets.extend(get_docker_targets());
    all_targets.extend(get_macos_targets());

    println!("{}", "Scanning all targets...".blue());

    let mut options = Vec::new();
    for target in &all_targets {
        options.push((target, target.size()));
    }

    // Фильтруем пустые цели
    let non_empty: Vec<(&CleanTarget, u64)> = options
        .into_iter()
        .filter(|(_, size)| *size > 0)
        .collect();

    if non_empty.is_empty() {
        println!("{}", "Nothing to clean! Your disk is already clean.".green());
        return;
    }

    let selected_indices = prompt_clean_targets(&non_empty);

    if selected_indices.is_empty() {
        println!("{}", "No items selected.".yellow());
        return;
    }

    let selected_targets: Vec<CleanTarget> = selected_indices
        .into_iter()
        .map(|i| non_empty[i].0.clone())
        .collect();

    clean_targets(&selected_targets, "Selected Items");
}

/// Быстрая очистка системных кэшей, логов и Корзины.
fn clean_system() {
    let targets = get_general_targets();
    clean_targets(&targets, "System (Caches, Logs, Trash)");
}

/// Очистка артефактов Xcode.
fn clean_xcode() {
    let targets = get_xcode_targets();
    clean_targets(&targets, "Xcode");
}

/// Очистка кэшей dev-инструментов.
fn clean_dev() {
    let targets = get_dev_targets();
    clean_targets(&targets, "Developer Tools");
}

/// Глубокая очистка System Data + Browsers + macOS installers.
fn deep_clean_system_data() {
    let mut targets = Vec::new();
    targets.extend(get_system_data_targets());
    targets.extend(get_browser_targets());
    targets.extend(get_macos_targets());
    clean_targets(&targets, "System Data + Browsers + macOS");
}

/// Очистка Docker данных.
fn clean_docker() {
    let targets = get_docker_targets();
    if targets.is_empty() {
        println!("\n{}", "No Docker targets found.".yellow());
        return;
    }
    clean_targets(&targets, "Docker");
}

/// Точка входа — главный цикл меню.
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
            "6" => {
                deep_clean_system_data();
                wait_for_enter();
            }
            "7" => {
                clean_docker();
                wait_for_enter();
            }
            "8" if !is_root() => {
                restart_with_sudo();
                wait_for_enter();
            }
            "8" if is_root() => {
                println!("\n{}", "Goodbye!".green().bold());
                break;
            }
            "9" | "q" | "Q" if !is_root() => {
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
