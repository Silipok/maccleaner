//! Пользовательский интерфейс (TUI).
//!
//! ASCII-баннер, главное меню, интерактивные промпты,
//! проверка прав администратора и перезапуск через `sudo`.

use bytesize::ByteSize;
use colored::*;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use std::io::{self, Write};
use std::process::Command;

use crate::targets::{CleanTarget, RiskLevel};

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
║              Mac Disk Cleaner v0.2.0                         ║
║         Free up disk space on macOS — Aggressive Mode        ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
"#;

/// Очищает экран терминала.
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

/// Выводит ASCII-баннер.
pub fn print_banner() {
    println!("{}", BANNER.cyan().bold());
}

/// Проверяет, запущен ли процесс от root.
pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Перезапускает через `sudo`.
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

/// Выводит главное меню с новыми категориями.
pub fn print_menu() {
    if is_root() {
        println!("{}", "  [Running as Administrator]".green().bold());
    } else {
        println!(
            "{}",
            "  [Running as normal user — some items need sudo]".yellow()
        );
    }
    println!();

    println!("{}", "┌──────────────────────────────────────────┐".cyan());
    println!("{}", "│              MAIN MENU                   │".cyan());
    println!("{}", "├──────────────────────────────────────────┤".cyan());

    let items = vec![
        ("1", "Interactive Clean"),
        ("2", "Scan All"),
        ("3", "Quick Clean System"),
        ("4", "Clean Xcode"),
        ("5", "Clean Developer Tools"),
        ("6", "Deep Clean System Data"),
        ("7", "Clean Docker"),
    ];

    for (num, label) in &items {
        println!(
            "{}  {}  {}",
            "│".cyan(),
            num.yellow().bold(),
            format!("{:<36}│", label).white()
        );
    }

    if !is_root() {
        println!(
            "{}  {}  {}",
            "│".cyan(),
            "8".yellow().bold(),
            "Run as Administrator (sudo)        │".white()
        );
        println!(
            "{}  {}  {}",
            "│".cyan(),
            "9".yellow().bold(),
            "Exit                               │".white()
        );
    } else {
        println!(
            "{}  {}  {}",
            "│".cyan(),
            "8".yellow().bold(),
            "Exit                               │".white()
        );
    }
    println!("{}", "└──────────────────────────────────────────┘".cyan());
    println!();
    print!("{}", "Enter your choice: ".green().bold());
    io::stdout().flush().unwrap();
}

/// Ожидает нажатия Enter.
pub fn wait_for_enter() {
    println!();
    print!("{}", "Press Enter to continue...".dimmed());
    io::stdout().flush().unwrap();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}

/// Интерактивный множественный выбор целей с бейджами риска и sudo.
pub fn prompt_clean_targets(items: &[(&CleanTarget, u64)]) -> Vec<usize> {
    if items.is_empty() {
        println!("{}", "No targets found.".yellow());
        return vec![];
    }

    let options: Vec<String> = items
        .iter()
        .map(|(target, size)| {
            let risk_badge = match target.risk_level {
                RiskLevel::Low => "".to_string(),
                RiskLevel::Medium => " ⚠MED".yellow().to_string(),
                RiskLevel::High => " 🚨HIGH".red().bold().to_string(),
            };

            let sudo_badge = if target.requires_sudo && !is_root() {
                " [SUDO]".magenta().to_string()
            } else {
                "".to_string()
            };

            format!(
                "[{}] {} ({}){}{} - {}",
                target.category.blue(),
                target.name.bold(),
                ByteSize(*size).to_string().yellow(),
                risk_badge,
                sudo_badge,
                target.description.dimmed()
            )
        })
        .collect();

    let selection = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select items to clean (Space = select, Enter = confirm):")
        .items(&options)
        .interact()
        .unwrap_or_default();

    selection
}
