//! Пользовательский интерфейс (TUI).
//!
//! Управляет визуальным выводом в терминал:
//! ASCII-баннер, главное меню, интерактивный промпт для выбора целей,
//! проверка прав администратора и перезапуск через `sudo`.

use bytesize::ByteSize;
use colored::*;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use std::io::{self, Write};
use std::process::Command;

use crate::targets::CleanTarget;

const BANNER: &str = r#"
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║   ███╗   ███╗ █████╗  ██████╗ ██████╗██╗     ███████╗       ║
║   ████╗ ████║██╔══██╗██╔════╝██╔════╝██║     ██╔════╝       ║
║   ██╔████╔██║███████║██║     ██║     ██║     █████╗         ║
║   ██║╚██╔╝██║██╔══██║██║     ██║     ██║     ██╔══╝         ║
║   ██║ ╚═╝ ██║██║  ██║╚██████╗╚██████╗███████╗███████╗       ║
║   ╚═╝     ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═════╝╚══════╝╚══════╝       ║
║                                                              ║
║              Mac Disk Cleaner v0.1.0                         ║
║              Free up disk space on macOS                     ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
"#;

/// Очищает экран терминала с помощью ANSI escape-кодов.
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

/// Выводит ASCII-баннер приложения с названием и версией.
pub fn print_banner() {
    println!("{}", BANNER.cyan().bold());
}

/// Проверяет, запущен ли процесс от имени суперпользователя (root, UID=0).
///
/// Использует `libc::geteuid()` для получения effective UID.
pub fn is_root() -> bool {
    // SAFETY: geteuid() is a simple syscall with no memory safety concerns.
    // It only returns the effective user ID as a u32.
    unsafe { libc::geteuid() == 0 }
}

/// Перезапускает текущий исполняемый файл через `sudo`.
///
/// При успешном перезапуске текущий процесс завершается с кодом 0.
/// При ошибке выводит сообщение и возвращается в меню.
pub fn restart_with_sudo() {
    let exe = std::env::current_exe().expect("Failed to get current executable path");
    println!("\n{}", "Requesting administrator privileges...".yellow());

    let status = Command::new("sudo").arg(exe).status();

    match status {
        Ok(s) if s.success() => {
            std::process::exit(0);
        }
        Ok(_) => {
            println!("{}", "Failed to obtain administrator privileges.".red());
        }
        Err(e) => {
            println!("{}: {}", "Error running sudo".red(), e);
        }
    }
}

/// Выводит главное меню.
///
/// Набор пунктов зависит от прав пользователя:
/// - Обычный пользователь: 7 пунктов (включая "Run as Administrator")
/// - Root: 6 пунктов ("Run as Administrator" заменяется на "Exit")
pub fn print_menu() {
    if is_root() {
        println!("{}", "[Running as Administrator]".green().bold());
    } else {
        println!(
            "{}",
            "[Running as normal user - some items may be protected]".yellow()
        );
    }
    println!();

    println!("{}", "┌──────────────────────────────────────┐".cyan());
    println!("{}", "│           MAIN MENU                  │".cyan());
    println!("{}", "├──────────────────────────────────────┤".cyan());
    println!(
        "{}  {}  {}",
        "│".cyan(),
        "1.".yellow().bold(),
        "Interactive Clean           │".white()
    );
    println!(
        "{}  {}  {}",
        "│".cyan(),
        "2.".yellow().bold(),
        "Scan all                    │".white()
    );
    println!(
        "{}  {}  {}",
        "│".cyan(),
        "3.".yellow().bold(),
        "Quick Clean System          │".white()
    );
    println!(
        "{}  {}  {}",
        "│".cyan(),
        "4.".yellow().bold(),
        "Clean Xcode                 │".white()
    );
    println!(
        "{}  {}  {}",
        "│".cyan(),
        "5.".yellow().bold(),
        "Clean Developer Tools       │".white()
    );

    if !is_root() {
        println!(
            "{}  {}  {}",
            "│".cyan(),
            "6.".yellow().bold(),
            "Run as Administrator (sudo) │".white()
        );
        println!(
            "{}  {}  {}",
            "│".cyan(),
            "7.".yellow().bold(),
            "Exit                        │".white()
        );
    } else {
        println!(
            "{}  {}  {}",
            "│".cyan(),
            "6.".yellow().bold(),
            "Exit                        │".white()
        );
    }
    println!("{}", "└──────────────────────────────────────┘".cyan());
    println!();
    print!("{}", "Enter your choice: ".green().bold());
    io::stdout().flush().unwrap();
}

/// Ожидает нажатия Enter для возврата в меню.
pub fn wait_for_enter() {
    println!();
    print!("{}", "Press Enter to continue...".dimmed());
    io::stdout().flush().unwrap();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}

/// Показывает интерактивный список с множественным выбором.
///
/// Для каждой цели отображаются: категория, имя, размер и описание.
/// Пользователь выбирает элементы клавишей `Space` и подтверждает клавишей `Enter`.
///
/// # Returns
///
/// Вектор индексов выбранных элементов из входного массива.
pub fn prompt_clean_targets(items: &[(&CleanTarget, u64)]) -> Vec<usize> {
    if items.is_empty() {
        println!("{}", "No targets found.".yellow());
        return vec![];
    }

    let options: Vec<String> = items
        .iter()
        .map(|(target, size)| {
            format!(
                "[{}] {} ({}) - {}",
                target.category.blue(),
                target.name.bold(),
                ByteSize(*size).to_string().yellow(),
                target.description.dimmed()
            )
        })
        .collect();

    let selection = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select items to clean (Space to select, Enter to confirm):")
        .items(&options)
        .interact()
        .unwrap_or_default();

    selection
}
