//! Логика сканирования и удаления файлов.
//!
//! Реализует методы [`CleanTarget::size()`] и [`CleanTarget::clean()`],
//! а также публичные функции [`scan_targets()`] и [`clean_targets()`]
//! для пакетной обработки нескольких целей.

use bytesize::ByteSize;
use colored::*;
use std::fs;
use std::io::{self, Write};

use crate::targets::{CleanTarget, RiskLevel, TargetType};
use crate::ui::is_root;
use crate::utils::{calculate_dir_size, command_exists, run_command};

impl CleanTarget {
    /// Возвращает физический размер директории в байтах.
    pub fn size(&self) -> u64 {
        calculate_dir_size(&self.path)
    }

    /// Удаляет цель в зависимости от её типа.
    pub fn clean(&self) -> io::Result<u64> {
        match &self.target_type {
            TargetType::Directory => self.clean_directory_contents(),
            TargetType::DirectoryFull => self.clean_directory_full(),
            TargetType::Command { program, args } => {
                self.clean_via_command(program, args)
            }
        }
    }

    /// Удаляет содержимое директории, сохраняя саму папку.
    fn clean_directory_contents(&self) -> io::Result<u64> {
        if !self.path.exists() {
            return Ok(0);
        }

        let mut freed_bytes = 0;

        for entry in fs::read_dir(&self.path)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            let size = if metadata.is_dir() {
                calculate_dir_size(&path)
            } else {
                use std::os::unix::fs::MetadataExt;
                metadata.blocks() * 512
            };

            if metadata.is_dir() {
                if let Err(e) = fs::remove_dir_all(&path) {
                    eprintln!("  {} Failed to remove {}: {}", "⚠".yellow(), path.display(), e);
                } else {
                    freed_bytes += size;
                }
            } else if let Err(e) = fs::remove_file(&path) {
                eprintln!("  {} Failed to remove {}: {}", "⚠".yellow(), path.display(), e);
            } else {
                freed_bytes += size;
            }
        }

        Ok(freed_bytes)
    }

    /// Удаляет директорию/файл целиком.
    fn clean_directory_full(&self) -> io::Result<u64> {
        if !self.path.exists() {
            return Ok(0);
        }

        let size = calculate_dir_size(&self.path);
        if self.path.is_dir() {
            fs::remove_dir_all(&self.path)?;
        } else {
            fs::remove_file(&self.path)?;
        }
        Ok(size)
    }

    /// Очищает через внешнюю команду (например, docker system prune).
    fn clean_via_command(&self, program: &str, args: &[String]) -> io::Result<u64> {
        if !command_exists(program) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Command '{}' not found in PATH", program),
            ));
        }

        let size_before = self.size();

        println!(
            "  {} Running: {} {}",
            "→".blue(),
            program,
            args.join(" ")
        );

        match run_command(program, args) {
            Ok(output) => {
                if !output.is_empty() {
                    for line in output.lines().take(10) {
                        println!("    {}", line.dimmed());
                    }
                }
            }
            Err(e) => {
                return Err(io::Error::other(format!("{}", e)));
            }
        }

        let size_after = self.size();
        Ok(size_before.saturating_sub(size_after))
    }
}

/// Сканирует список целей и выводит таблицу размеров.
pub fn scan_targets(targets: &[CleanTarget], title: &str) -> u64 {
    println!("\n{}", format!("Scanning {}...", title).bold().blue());
    let mut total_size = 0;

    println!(
        "{:<24} {:<15} {:<30} Path",
        "Category", "Size", "Description"
    );
    println!("{}", "-".repeat(100));

    for target in targets {
        let size = target.size();
        total_size += size;

        let risk_badge = match target.risk_level {
            RiskLevel::Low => "".to_string(),
            RiskLevel::Medium => " [MED]".yellow().to_string(),
            RiskLevel::High => " [HIGH]".red().bold().to_string(),
        };

        let sudo_badge = if target.requires_sudo && !is_root() {
            " [SUDO]".magenta().to_string()
        } else {
            "".to_string()
        };

        println!(
            "{:<24} {:<15} {:<30} {}{}{}",
            target.name.bold(),
            ByteSize(size).to_string().yellow(),
            target.description.italic(),
            target.path.display().to_string().dimmed(),
            risk_badge,
            sudo_badge,
        );
    }

    println!("{}", "-".repeat(100));
    println!("Total: {}", ByteSize(total_size).to_string().green().bold());

    total_size
}

/// Показывает размеры, пояснения, запрашивает подтверждение и удаляет.
pub fn clean_targets(targets: &[CleanTarget], category_name: &str) {
    let mut total_possible = 0;
    let mut skipped_sudo = Vec::new();

    for target in targets {
        if target.requires_sudo && !is_root() {
            skipped_sudo.push(target);
        }
        total_possible += target.size();
    }

    if total_possible == 0 {
        println!(
            "\n{}",
            format!("Nothing to clean in {}.", category_name).green()
        );
        return;
    }

    println!("\n{}", format!("Ready to clean {}:", category_name).bold());
    scan_targets(targets, category_name);

    // Показать пояснения для каждой непустой цели
    let has_explanations = targets.iter().any(|t| !t.explanation.is_empty() && t.size() > 0);
    if has_explanations {
        println!("\n{}", "─── Explanations ───".cyan().bold());
        for target in targets {
            if target.explanation.is_empty() || target.size() == 0 {
                continue;
            }
            println!("\n  {} {}:", "▸".cyan(), target.name.bold());
            for line in target.explanation.lines() {
                println!("    {}", line.dimmed());
            }
        }
        println!("{}", "─".repeat(40).cyan());
    }

    // Предупреждение о sudo-protected целях
    if !skipped_sudo.is_empty() {
        println!(
            "\n{}",
            "⚠  Some targets require sudo and will be SKIPPED:"
                .yellow()
                .bold()
        );
        for t in &skipped_sudo {
            println!("   • {} ({})", t.name, t.path.display());
        }
        println!(
            "{}",
            "   Run with 'sudo' or use menu option to get admin rights."
                .yellow()
        );
    }

    // Проверяем наличие high-risk целей
    let has_high_risk = targets
        .iter()
        .any(|t| t.risk_level == RiskLevel::High && t.size() > 0);

    if has_high_risk {
        println!(
            "\n{}",
            "🚨 WARNING: HIGH RISK targets included! Data loss is IRREVERSIBLE!"
                .red()
                .bold()
        );
    }

    println!(
        "\n{}",
        "⚠  This will permanently delete the files listed above."
            .red()
            .bold()
    );
    print!("{}", "Are you sure you want to proceed? (y/N): ".yellow());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        println!("{}", "Failed to read input.".red());
        return;
    }
    if input.trim().to_lowercase() != "y" {
        println!("{}", "Aborted.".yellow());
        return;
    }

    // Двойное подтверждение для high-risk
    if has_high_risk {
        print!(
            "{}",
            "🚨 Type 'DELETE' to confirm HIGH RISK cleanup: ".red().bold()
        );
        io::stdout().flush().unwrap();

        let mut confirm = String::new();
        if io::stdin().read_line(&mut confirm).is_err() || confirm.trim() != "DELETE" {
            println!("{}", "Aborted (high risk confirmation failed).".yellow());
            return;
        }
    }

    println!("\n{}", "Cleaning...".bold().blue());
    let mut total_freed = 0;

    for target in targets {
        // Пропускаем sudo-protected цели если не root
        if target.requires_sudo && !is_root() {
            continue;
        }

        // Пропускаем пустые цели
        if target.size() == 0 {
            continue;
        }

        match target.clean() {
            Ok(freed) => {
                total_freed += freed;
                println!(
                    "  {} Cleaned {}: {}",
                    "✓".green(),
                    target.name,
                    ByteSize(freed).to_string().green()
                );
            }
            Err(e) => {
                eprintln!(
                    "  {} Error cleaning {}: {}",
                    "✗".red(),
                    target.name,
                    e
                );
            }
        }
    }

    println!(
        "\n{} {}",
        "Total space freed:".bold(),
        ByteSize(total_freed).to_string().green().bold()
    );
}
