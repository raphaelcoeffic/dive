use std::{
    io::{self, IsTerminal, Write},
    os::unix::process::CommandExt,
    process::{self, Command},
};

use anstream::{eprintln, println};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List installed packages
    List,
    /// Search packages
    Search {
        #[arg(required = true)]
        keywords: Vec<String>,
    },
    /// Install a package
    Install { name: String },
    /// Remove a package
    Remove { name: String },
    #[command(name = "_CmdNotFound", hide = true, disable_help_flag = true)]
    _CmdNotFound {
        cmd: String,
        #[arg(
            trailing_var_arg = true,
            allow_hyphen_values = true,
            hide = true
        )]
        args: Vec<String>,
    },
}

fn init_logging() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .parse_env("LOGLEVEL")
        .format_timestamp(None)
        .format_target(false)
        .init();
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    init_logging();

    match &cli.command {
        Commands::List => list_packages()?,
        Commands::Search { keywords } => search_packages(keywords)?,
        Commands::Install { name } => install_package(name)?,
        Commands::Remove { name } => remove_package(name)?,
        Commands::_CmdNotFound { cmd, args } => command_not_found(cmd, args)?,
    }

    Ok(())
}

fn list_packages() -> Result<()> {
    for pkg in dive::nixpkgs::all_packages_sorted()? {
        println!("{}  ({})", pkg.name.bold(), pkg.version.dimmed());
    }
    Ok(())
}

fn search_packages(keywords: &Vec<String>) -> Result<()> {
    let matches = dive::nixpkgs::query(keywords)?;
    if matches.is_empty() {
        println!("{}", "No matches".bold());
    } else {
        for pkg in matches {
            println!("* {} ({})", pkg.name.bold(), pkg.version.dimmed());
            if let Some(description) = pkg.description {
                println!("    {}", description);
            }
            println!();
        }
    }
    Ok(())
}

fn install_package(name: &str) -> Result<()> {
    let pkgs = dive::nixpkgs::all_packages_sorted()?;
    if pkgs.iter().any(|p| p.name == name) {
        eprintln!("error: '{}' is already installed", name);
        process::exit(1);
    }
    match dive::nixpkgs::find_package(name)? {
        None => {
            eprintln!("error: '{}' does not exist", name);
            process::exit(1);
        }
        Some(pkg) => {
            if let Err(err) = dive::nixpkgs::install_package(pkg)
                .context("failed to install package")
            {
                eprintln!("error: {err}");
                process::exit(1);
            }
            Ok(())
        }
    }
}

fn remove_package(name: &str) -> Result<()> {
    let pkgs = dive::nixpkgs::all_packages_sorted()?;
    let maybe_pkg = pkgs.iter().find(|p| p.name == name);
    if maybe_pkg.is_none() {
        eprintln!("error: '{}' is not installed", name);
        process::exit(1);
    }
    let pkg = maybe_pkg.unwrap();
    if pkg.is_builtin() {
        eprintln!(
            "Error: '{}' is a built-in package and cannot be removed",
            name
        );
        process::exit(1);
    }
    dive::nixpkgs::remove_package(name).context("failed to remove package")
}

fn find_program(name: &str) -> Result<Option<String>> {
    let programs = include_bytes!("../assets/programs.csv");
    let mut rdr = csv::Reader::from_reader(programs.as_slice());
    let mut record = csv::StringRecord::new();
    while rdr.read_record(&mut record)? {
        debug_assert!(record.len() == 2, "CSV index corrupted");
        match record[0].cmp(name) {
            std::cmp::Ordering::Equal => {
                let pkg_name = &record[1];
                return Ok(Some(pkg_name.to_owned()));
            }
            std::cmp::Ordering::Greater => break,
            _ => continue,
        }
    }
    Ok(None)
}

fn command_not_found(cmd: &str, args: &Vec<String>) -> Result<()> {
    if !io::stdin().is_terminal() {
        process::exit(1);
    }
    let pkg_name = match find_program(cmd) {
        Result::Err(err) => {
            eprintln!("Error: {err}");
            process::exit(1);
        }
        Result::Ok(prog_match) => match prog_match {
            None => {
                log::debug!("No match");
                process::exit(1);
            }
            Some(pkg_name) => pkg_name,
        },
    };
    print!(
        "* {} does not exist. Install {} package? [y/N] ",
        cmd,
        pkg_name.bold()
    );
    let _ = io::stdout().flush();
    let confirmed = match console::Term::stdout().read_key() {
        Result::Err(err) => {
            log::debug!("Error: {err}");
            process::exit(1);
        }
        Result::Ok(key) => match key {
            console::Key::Char(c) => c == 'Y' || c == 'y',
            _ => false,
        },
    };
    println!();
    if confirmed {
        install_package(&pkg_name)?;
        let err = Command::new(cmd).args(args).exec();
        eprintln!("Error: failed to exec {cmd}: {err}");
    }
    process::exit(1);
}
