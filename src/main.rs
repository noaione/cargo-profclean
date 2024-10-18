use std::{path::{Path, PathBuf}, sync::LazyLock};

use clap::{builder::{styling::{AnsiColor, Effects}, Styles}, Parser};
use color_print::cprintln;

const CARGO_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let binding = directories::UserDirs::new().unwrap();
    let home = binding.home_dir();
    home.join(".cargo")
});

#[derive(Parser)]
#[command(name = "cargo-profclean")]
#[command(bin_name = "cargo-profclean")]
#[command(author, version = env!("CARGO_PKG_VERSION"), about, long_about = None, styles = cli_styles())]
#[command(propagate_version = true, disable_help_subcommand = true)]
enum CargoProfcleanCli {
    /// Clean all profdata files
    Clean {
        /// Path to cargo base dir, defaults to $HOME/.cargo
        input: Option<std::path::PathBuf>,
    },
}

fn main() {
    match CargoProfcleanCli::parse() {
        CargoProfcleanCli::Clean { input } => {
            let crates_dir = get_index_crates_io(&input.unwrap_or(CARGO_DIR.to_path_buf()));

            match crates_dir {
                None => {
                    cprintln!("No <m,s>index.crates.io</> directory found in <m,s>{}</>", CARGO_DIR.display());
                }
                Some(dir) => {
                    let all_profdata = collect_crates_folder(&dir);
                    let has_profdata = !all_profdata.is_empty();
                    let profdata_count = all_profdata.len();
                    cprintln!("Found {} profdata files in <m,s>{}</>", profdata_count, dir.display());
                    for mm_profdata in all_profdata {
                        cprintln!(" Cleaning <m,s>{}</>", mm_profdata.display());
                        match std::fs::remove_file(&mm_profdata) {
                            Ok(_) => {}
                            Err(err) => cprintln!("  Error cleaning <m,s>{}</>: {}", mm_profdata.display(), err),
                        }
                    }
                    if has_profdata {
                        cprintln!("Cleaned <m,s>{}</> profdata files", profdata_count);
                    } else {
                        cprintln!("No profdata files found in <m,s>{}</>", dir.display());
                    }
                }
            }
        }
    }
}

fn get_index_crates_io(dir: &Path) -> Option<PathBuf> {
    let src_dir = dir.join("registry").join("src");
    match src_dir.read_dir() {
        Ok(entries) => {
            // Find index.crates.io-XXXXXXXXXXXX folder
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.starts_with("index.crates.io-") {
                            return Some(entry.path());
                        }
                    }
                }
            }
            None
        }
        Err(_) => None,
    }
}

fn collect_crates_folder(dir: &Path) -> Vec<PathBuf> {
    let mut mm_profdata = vec![];
    if let Ok(entries) = dir.read_dir() {
        for entry in entries {
            if let Ok(entry) = entry {
                mm_profdata.extend_from_slice(&collect_mm_profdata(&entry.path()));
            }
        }
    }
    mm_profdata
}

fn collect_mm_profdata(dir: &Path) -> Vec<PathBuf> {
    let mut mm_profdata = vec![];
    if let Ok(entries) = dir.read_dir() {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".mm_profdata") {
                        mm_profdata.push(entry.path());
                    }
                }
            }
        }
    }
    mm_profdata
}


fn cli_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default() | Effects::BOLD)
        .usage(AnsiColor::Magenta.on_default() | Effects::BOLD | Effects::UNDERLINE)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::BrightCyan.on_default())
}
