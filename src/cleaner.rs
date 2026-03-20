//! Логика сканирования и удаления файлов.
//!
//! Реализует методы [`CleanTarget::size()`] и [`CleanTarget::clean()`],
//! а также публичные функции [`scan_targets()`] и [`clean_targets()`]
//! для пакетной обработки нескольких целей.

use bytesize::ByteSize;
use colored::*;
use std::fs;
use std::io::{self, Write};

use crate::targets::CleanTarget;
use crate::utils::calculate_dir_size;

impl CleanTarget {
    /// Возвращает физический размер директории в байтах.
    ///
    /// Использует параллельный подсчёт через [`calculate_dir_size()`](crate::utils::calculate_dir_size)
    /// с учётом блочного размера APFS и дедупликацией hard links.
    pub fn size(&self) -> u64 {
        calculate_dir_size(&self.path)
    }

    /// Удаляет **содержимое** директории, сохраняя саму папку.
    ///
    /// Обходит непосредственных потомков целевой директории:
    /// - Поддиректории удаляются через [`fs::remove_dir_all()`]
    /// - Файлы удаляются через [`fs::remove_file()`]
    ///
    /// При ошибке удаления отдельного элемента выводит предупреждение
    /// и продолжает работу.
    ///
    /// # Returns
    ///
    /// Количество успешно освобождённых байт (физический размер в блоках).
    pub fn clean(&self) -> io::Result<u64> {
        if !self.path.exists() {
            return Ok(0);
        }

        let mut freed_bytes = 0;

        // Strategy: Iterate over immediate children and delete them.
        // We don't want to delete the root directory itself (e.g. ~/.Trash), just contents.
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
                    eprintln!("Failed to remove {}: {}", path.display(), e);
                } else {
                    freed_bytes += size;
                }
            } else if let Err(e) = fs::remove_file(&path) {
                eprintln!("Failed to remove {}: {}", path.display(), e);
            } else {
                freed_bytes += size;
            }
        }

        Ok(freed_bytes)
    }
}

/// Сканирует список целей и выводит таблицу размеров.
///
/// Для каждой цели подсчитывается физический размер директории
/// (параллельно через `jwalk`). Результат выводится в форматированной
/// таблице с цветным выделением.
///
/// # Returns
///
/// Общий размер всех целей в байтах.
pub fn scan_targets(targets: &[CleanTarget], title: &str) -> u64 {
    println!("\n{}", format!("Scanning {}...", title).bold().blue());
    let mut total_size = 0;

    println!(
        "{:<20} {:<15} {:<30} Path",
        "Category", "Size", "Description"
    );
    println!("{}", "-".repeat(100));

    for target in targets {
        let size = target.size();
        total_size += size;
        println!(
            "{:<20} {:<15} {:<30} {}",
            target.name.bold(),
            ByteSize(size).to_string().yellow(),
            target.description.italic(),
            target.path.display().to_string().dimmed()
        );
    }

    println!("{}", "-".repeat(100));
    println!("Total: {}", ByteSize(total_size).to_string().green().bold());

    total_size
}

/// Показывает размеры целей, запрашивает подтверждение и удаляет содержимое.
///
/// Последовательность действий:
/// 1. Подсчёт общего размера всех целей
/// 2. Если ничего не найдено — вывод сообщения и выход
/// 3. Отображение таблицы размеров
/// 4. Предупреждение и запрос подтверждения `(y/N)`
/// 5. Последовательное удаление каждой цели с отчётом
pub fn clean_targets(targets: &[CleanTarget], category_name: &str) {
    let mut total_possible = 0;

    for target in targets {
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

    println!(
        "\n{}",
        "WARNING: This will permanently delete the files above."
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

    println!("\n{}", "Cleaning...".bold().blue());
    let mut total_freed = 0;

    for target in targets {
        match target.clean() {
            Ok(freed) => {
                total_freed += freed;
                println!(
                    "Cleaned {}: {}",
                    target.name,
                    ByteSize(freed).to_string().green()
                );
            }
            Err(e) => {
                eprintln!("Error cleaning {}: {}", target.name, e);
            }
        }
    }

    println!(
        "\nTotal space freed: {}",
        ByteSize(total_freed).to_string().green().bold()
    );
}
