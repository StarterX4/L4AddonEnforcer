// SPDX-License-Identifier: LGPL-3.0-only

#![allow(non_snake_case)]
#![windows_subsystem = "windows"]
use clap::Parser;
use colored::Colorize;
use helptext::{Help, sections};
// use path_dedot::ParseDot;
use md5::{Digest, Md5};
use std::{
    /* borrow::Cow, */ env::var_os,
    fmt::{self, Debug},
    fs::{File, copy, create_dir_all, read_to_string, remove_file, write},
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
    process::exit,
};
use steamlocate::SteamDir;

mod gui;
mod gui_theming;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE_PATH")]
    file: Option<String>,
    #[arg(short, long, value_name = "NAME")]
    name: Option<String>,
    #[arg(short, long, value_name = "NAME")]
    uninstall: Option<String>,
    #[arg(short, long)]
    list: bool,
    #[arg(short, long)] // TODO
    quiet: bool,
    #[arg(short = 'p', long = "pug")]
    pug_switch: bool,
    #[arg(short = 'P', long = "checkpug")]
    pug_check: bool,
    #[arg(short, long)]
    rename: Option<String>,
    #[arg(long)]
    reset: bool,
    #[arg(short, long)]
    verbose: bool,
    #[arg(short)]
    help: bool,
    #[arg(long = "help")]
    help_long: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if (args.file.is_none() && args.rename.is_none()) && args.name.is_some() {
        let err = format!(
            "Argument -n <name> has been passed but no -f <file> or -r <old_name> have been passed!
                                \nNo addon.vpk file provided for installation
                                \nNor a currently installed addon for renaming."
        );
        eprintln!("{} {}", "Error:".red(), err);
        println!(
            "Type {} / {} for more information",
            "-h".blue(),
            "--help".blue()
        );
        return Err(Box::new(QuietErr(Some(err))));
    }

    if let Some(addon_file) = args.file {
        // Install or update logic
        if let Some(name) = args.name {
            let _ = install_addon(&addon_file, &name, args.verbose); // Unused result becase of QuietErr usage?
        } else {
            eprintln!("{} No addon name provided for installation", "Error:".red());
            println!(
                "Type {} / {} for more information",
                "-h".blue(),
                "--help".blue()
            );
            exit(22)
        }
    } else if let Some(name) = args.uninstall {
        // Uninstall logic
        let _ = uninstall_addon(&name, args.verbose); // Unused result becase of QuietErr usage?
    } else if args.list {
        // List addons
        let _ = list_addons(args.quiet, args.verbose, &mut std::io::stdout()); // Unused result becase of QuietErr usage?
    } else if args.pug_switch {
        // PuG mode switch logic
        let _ = PuG_mode_switch(args.verbose);
    } else if args.pug_check {
        // PuG mode check logic
        let _ = PuG_mode_check(args.verbose);
    } else if let Some(ren) = args.rename {
        // Rename logic
        if let Some(name) = args.name {
            let _ = rename_addon(&ren, &name, args.verbose); // Unused result becase of QuietErr usage?
        } else {
            eprintln!("{} No new addon name provided for renaming", "Error:".red());
            println!(
                "Type {} / {} for more information",
                "-h".blue(),
                "--help".blue()
            );
            exit(22)
        }
    } else if args.reset {
        // Gameinfo.txt reset logic
        let _ = gameinfo_reset(args.verbose); // Unused result becase of QuietErr usage?
    } else if args.help {
        // Help logic
        print_short_help(true);
    } else if args.help_long {
        // Long help logic
        print_long_help(true);
    } else {
        gui::main();
    }

    Ok(())
}

fn install_addon(
    addon_file: &str,
    name: &str,
    verbose: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    let gameinfo_orig_md5 = format!("586b3b0b39bc44ddfb07792b1932c479");

    // Require both arguments on installation
    if (name.is_empty() && !addon_file.is_empty()) || (!name.is_empty() && addon_file.is_empty()) {
        let err = format!("Both addon name and addon file path must be provided!");
        eprintln!("{} {}", "Error:".red(), err);
        return Err(Box::new(QuietErr(Some(err))));
    }
    // Validate addon name
    if
    /*name.is_empty() || */
    name.contains(char::is_whitespace)
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
        || name.contains('*')
        || name.contains('?')
        || name.contains('"')
        || name.contains('<')
        || name.contains('>')
        || name.contains('|')
    {
        let err = format!(
            "Invalid addon name! \n\tName cannot be empty, contain whitespace, or special characters \n\tthat are known to cause problems with file managers or filesystems."
        );
        eprintln!("{} {}", "Error:".red(), err);
        return Err(Box::new(QuietErr(Some(err))));
    }

    // Validate addon file
    let addon_path = PathBuf::from(&addon_file);
    if var_os("DEBUG").is_some() {
        println!("{} {} {:?}", "[D]".blue(), "Addon path:".bold(), addon_path);
    }
    if !addon_path.is_file() {
        let err = format!("Invalid addon file path!");
        eprintln!("{} {}", "Error:".red(), err);
        return Err(Box::new(QuietErr(Some(err))));
    }

    // Locate the gameinfo.txt file
    let gameinfo_path = gameinfo_path(verbose)?;
    // Calculate the MD5 of the gameinfo.txt file
    let gameinfo_md5 = calculate_md5(&gameinfo_path).expect("Failed to calculate MD5");

    let gameinfo_backup_path = gameinfo_backup_path(verbose)?;
    if var_os("DEBUG").is_some() || verbose {
        println!(
            "{} {} {:?}",
            "[D]".blue(),
            "gameinfo_backup_path:".bold(),
            gameinfo_backup_path
        );
    }
    if !gameinfo_backup_path.exists() {
        if gameinfo_md5 != gameinfo_orig_md5 {
            println!(
                "{} gameinfo.txt file seems to be modified!",
                "Warning:".yellow()
            );
            println!("The backup may be useless. Though proceeding anyway...");
            println!("");
            println!("If you haven't already modified the gameinfo.txt, this is probably");
            println!(
                "\ta {}'s bug you can report to the dev!",
                env!("CARGO_PKG_NAME")
            );
            println!("\t\tYour gameinfo.txt MD5 hash is: {}", gameinfo_md5.bold());
        }
        // Create a backup of the gameinfo.txt file
        copy(&gameinfo_path, &gameinfo_backup_path)?;
    } else {
        if var_os("DEBUG").is_some() || verbose {
            println!(
                "{} Found a gameinfo.txt backup ({:?})",
                "[D]".blue(),
                gameinfo_backup_path.file_name().unwrap().to_string_lossy()
            );
        }
    }

    // Read the gameinfo.txt file
    let contents = read_to_string(&gameinfo_path)?;

    // Split the file into lines
    let mut lines: Vec<&str> = contents.lines().collect();

    // Create the new addon directory
    if var_os("DEBUG").is_some() || verbose {
        println!("{} Creating addon directory: {}", "[D]".blue(), name);
    }
    let l4d2_dir = l4d2_path()?;
    let addon_dir = l4d2_dir.join(format!("{}", name));
    let mut addon_dir_existed = false;
    if addon_dir.exists() {
        addon_dir_existed = true;
        if var_os("DEBUG").is_some() || verbose {
            println!(
                "{} {} An addon directory with the same name already exists! Assuming this is an update.",
                "[D]".blue(),
                "Warning:".yellow()
            );
        }
    } else {
        create_dir_all(&addon_dir)?;
    }

    // Copy the addon file to the new addon directory
    if var_os("DEBUG").is_some() || verbose {
        println!("{} Copying addon file to: {:?}", "[D]".blue(), addon_dir);
    }
    let mut destination = addon_dir.join(addon_path.file_name().unwrap());
    destination.set_file_name("pak01_dir");
    destination.set_extension("vpk");
    if destination.exists() {
        remove_file(&destination)?;
        if var_os("DEBUG").is_some() || verbose {
            println!("{} Deleted {:?}", "[D]".blue(), destination);
        }
    }
    copy(&addon_path, &destination)?;

    // The line to insert
    let new_line = format!("\t\t\tGame\t\t\t\t{}", name);

    let mut up = false;
    if lines
        .iter()
        .any(|line| line.contains("Game") && line.contains(&name))
        && addon_dir_existed
    {
        println!("Updated {} successfully.", name.italic());
        up = true;
    } else {
        // Find the line with "Game update"
        let index = lines
            .iter()
            .position(|line| line.contains("Game") && line.contains("update"))
            .unwrap_or(0);

        // Insert the new line above it
        lines.insert(index, &new_line);

        // Join the lines back into a single string
        let new_contents = lines.join("\n");

        // Write the updated contents back to the file
        write(&gameinfo_path, new_contents)?;
        println!("Installed {} successfully.", name.italic());
    }
    if up { Ok(2) } else { Ok(1) }
}

fn list_addons(
    quiet: bool,
    verbose: bool,
    buf_writer: &mut impl Write,
) -> Result<(), Box<dyn std::error::Error>> {
    // Locate the gameinfo.txt file
    let gameinfo_path = gameinfo_path(verbose)?;

    // Read the gameinfo.txt file
    let contents = read_to_string(&gameinfo_path)?;

    // Split the file into lines
    let lines: Vec<&str> = contents.lines().collect();

    let excl_addons = vec![
        "update",
        "left4dead2_dlc3",
        "left4dead2_dlc2",
        "left4dead2_dlc1",
        "hl2",
        "|gameinfo_path|.",
    ];

    // List the installed custom addons
    let mut SearchPaths = false;
    if lines.iter().any(|line| line.contains("SearchPaths")) {
        if !quiet {
            println!("{}", "Installed addons:".bold());
        }
        for line in lines.iter() {
            if line.contains("SearchPaths") {
                SearchPaths = true;
            } else if SearchPaths && line.contains("}") {
                SearchPaths = false;
            }
            if SearchPaths
                && line.contains("Game")
                && !excl_addons.iter().any(|&s| line.contains(s))
            {
                let addon = line
                    .trim_start_matches('\t')
                    .trim_start_matches("Game")
                    .replace("\t\t\t\t", "\t");
                writeln!(buf_writer, "{}", addon).unwrap();
            }
        }
        return Ok(());
    }
    Ok(())
}

fn rename_addon(
    ren_name: &str,
    new_name: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if ren_name.is_empty() || new_name.is_empty() {
        let err = format!("No addon names provided for renaming");
        eprintln!("{} {}", "Error:".red(), err);
        return Err(Box::new(QuietErr(Some(err))));
    }
    // Validate addon name
    if new_name.contains(char::is_whitespace)
        || new_name.contains('/')
        || new_name.contains('\\')
        || new_name.contains(':')
        || new_name.contains('*')
        || new_name.contains('?')
        || new_name.contains('"')
        || new_name.contains('<')
        || new_name.contains('>')
        || new_name.contains('|')
    {
        let err = format!(
            "Invalid addon name! \n\tNew name cannot be empty, contain whitespace, or special characters \n\tthat are known to cause problems with file managers or filesystems."
        );
        eprintln!("{} {}", "Error:".red(), err);
        return Err(Box::new(QuietErr(Some(err))));
    }

    // Locate the Left 4 Dead 2 directory ... LEAVE THIS WITH THE OLD WAY FOR NOW ...
    if var_os("DEBUG").is_some() || verbose {
        println!("{} Locating L4D2 directory...", "[D]".blue());
    }
    let l4d2_dir = l4d2_path().expect("Failed to locate Left 4 Dead 2 directory");
    let gameinfo_path = l4d2_dir.join("left4dead2/gameinfo.txt");

    if !gameinfo_path.exists() {
        let err = format!("Unable to locate gameinfo.txt file. Is the game installation broken?");
        eprintln!("{} {}", "Error:".red(), err);
        return Err(Box::new(QuietErr(Some(err))));
    }

    if var_os("DEBUG").is_some() || verbose {
        println!(
            "{} {} {:?}",
            "[D]".blue(),
            "Gameinfo.txt path:".bold(),
            gameinfo_path
        );
    }

    // Read the gameinfo.txt file
    let contents = read_to_string(&gameinfo_path)?;

    // Split the file into lines
    let mut lines: Vec<&str> = contents.lines().collect();

    let excl_addons = vec![
        "update",
        "left4dead2_dlc3",
        "left4dead2_dlc2",
        "left4dead2_dlc1",
        "hl2",
        "|gameinfo_path|.",
    ];

    // Delete the selected addon
    if !ren_name.is_empty()
        && !new_name.is_empty()
        && !excl_addons.iter().any(|&s| ren_name.contains(s))
        && !excl_addons.iter().any(|&s| new_name.contains(s))
    {
        if lines
            .iter()
            .any(|line| line.contains("Game") && line.contains(&new_name))
        {
            let err = format!(
                "{} is already installed! \n\t Please choose a different name.",
                new_name
            );
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }

        if lines
            .iter()
            .any(|line| line.contains("Game") && line.contains(&ren_name))
        {
            // The line to insert
            let new_line = format!("\t\t\tGame\t\t\t\t{}", new_name);
            // Find the line with "Game {ren_name}"
            let index = lines
                .iter()
                .position(|line| line.contains("Game") && line.contains(&ren_name))
                .unwrap();
            lines.remove(index);
            if var_os("DEBUG").is_some() || verbose {
                println!(
                    "{} Removing line \n{} \nfrom gameinfo.txt",
                    "[D]".blue(),
                    index
                );
            }
            // Insert the new line above it
            lines.insert(index, &new_line);
            if var_os("DEBUG").is_some() || verbose {
                println!(
                    "{} Inserting line \n{} \n in the line \n {} \n in gameinfo.txt",
                    "[D]".blue(),
                    new_line,
                    index
                );
            }
            let new_contents = lines.join("\n");
            write(&gameinfo_path, new_contents)?;
        } else {
            let err = format!("{} not found in the gameinfo.txt file!", ren_name);
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }
        let ren_addon_dir = l4d2_dir.join(format!("{}", ren_name));
        let ren_new_addon_dir = l4d2_dir.join(format!("{}", new_name));
        if ren_addon_dir.exists() {
            if ren_addon_dir.is_dir() {
                std::fs::rename(&ren_addon_dir, &ren_new_addon_dir)?;
                println!(
                    "Renamed {} to {} successfully.",
                    ren_name.italic(),
                    new_name.italic()
                );
            } else {
                if var_os("DEBUG").is_some() || verbose {
                    println!(
                        "{} {} appears to not be a directory! (filesystem damaged? installation failed?)",
                        "[D]".blue(),
                        ren_addon_dir.display()
                    );
                }
            }
        }
        return Ok(());
    } else {
        if excl_addons.iter().any(|&s| ren_name.contains(s)) {
            let err = format!(
                "Core game components cannot be renamed! \n\t Found: \"{}\"",
                ren_name
            );
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }
        if excl_addons.iter().any(|&s| new_name.contains(s)) {
            let err = format!(
                "New name \"{}\" conflicts with one of the core game components!",
                new_name
            );
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }
    }
    Ok(())
}

fn uninstall_addon(del_name: &str, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if del_name.is_empty() {
        let err = format!("No addon name provided for uninstallation");
        eprintln!("{} {}", "Error:".red(), err);
        return Err(Box::new(QuietErr(Some(err))));
    }

    // Locate the gameinfo.txt file
    let gameinfo_path = gameinfo_path(verbose)?;

    // Read the gameinfo.txt file
    let contents = read_to_string(&gameinfo_path)?;

    // Split the file into lines
    let mut lines: Vec<&str> = contents.lines().collect();

    let excl_addons = vec![
        "update",
        "left4dead2_dlc3",
        "left4dead2_dlc2",
        "left4dead2_dlc1",
        "hl2",
        "|gameinfo_path|.",
    ];

    // Delete the selected addon
    if !del_name.is_empty() && !excl_addons.iter().any(|&s| del_name.contains(s)) {
        if lines
            .iter()
            .any(|line| line.contains("Game") && line.contains(&del_name))
        {
            let index = lines
                .iter()
                .position(|line| line.contains("Game") && line.contains(&del_name))
                .unwrap();
            lines.remove(index);
            let new_contents = lines.join("\n");
            write(&gameinfo_path, new_contents)?;
            if var_os("DEBUG").is_some() || verbose {
                println!(
                    "{} Removing line \n{} \nfrom gameinfo.txt",
                    "[D]".blue(),
                    index
                );
            }
        } else {
            let err = format!("{} not found in the gameinfo.txt file!", del_name);
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }
        let del_addon_dir = l4d2_path()?.join(format!("{}", del_name));
        if del_addon_dir.exists() {
            if del_addon_dir.is_dir() {
                std::fs::remove_dir_all(&del_addon_dir)?;
                println!("Uninstalled {} successfully.", del_name.italic());
            } else {
                if var_os("DEBUG").is_some() || verbose {
                    println!(
                        "{} {} appears to not be a directory! (filesystem damaged? installation failed?)",
                        "[D]".blue(),
                        del_addon_dir.display()
                    );
                }
            }
        }
        return Ok(());
    } else {
        if excl_addons.iter().any(|&s| del_name.contains(s)) {
            let err = format!("Core game components cannot be uninstalled!");
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }
    }
    Ok(())
}

fn gameinfo_reset(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let gameinfo_orig_md5 = format!("586b3b0b39bc44ddfb07792b1932c479");
    // Locate the gameinfo.txt file
    let gameinfo_path = gameinfo_path(verbose)?;

    // Calculate the MD5 of the gameinfo.txt file
    let gameinfo_md5 = calculate_md5(&gameinfo_path).expect("Failed to calculate MD5");

    let gameinfo_backup_path = gameinfo_backup_path(verbose)?;
    if var_os("DEBUG").is_some() || verbose {
        println!(
            "{} {} {:?}",
            "[D]".blue(),
            "gameinfo_backup_path:".bold(),
            gameinfo_backup_path
        );
    }
    if !gameinfo_backup_path.exists() {
        if gameinfo_md5 != gameinfo_orig_md5 {
            println!(
                "{} gameinfo.txt file seems to be modified, but no backup is present!",
                "Warning:".yellow()
            );
            println!("");
            println!("If you haven't already modified the gameinfo.txt, this is probably");
            println!(
                "\ta {}'s bug you can report to the dev!",
                env!("CARGO_PKG_NAME")
            );
            println!("\t\tYour gameinfo.txt MD5 hash is: {}", gameinfo_md5.bold());
        } else {
            let err = format!("gameinfo.txt is already at its default state!");
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }
    } else {
        if gameinfo_md5 != gameinfo_orig_md5 {
            if var_os("DEBUG").is_some() || verbose {
                println!(
                    "{} Copying gameinfo.txt backup ({:?}) to {:?}",
                    "[D]".blue(),
                    &gameinfo_backup_path.file_name().unwrap().to_string_lossy(),
                    &gameinfo_path.file_name().unwrap().to_string_lossy()
                );
            }
            copy(&gameinfo_backup_path, &gameinfo_path)?;
            println!("Succesfully reset gameinfo.txt to default.");
        } else {
            let err = format!("gameinfo.txt is already at its default state!");
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        }
    }
    Ok(())
}

fn PuG_mode_switch(verbose: bool) -> Result<i32, Box<dyn std::error::Error>> {
    let gameinfo_orig_md5 = format!("586b3b0b39bc44ddfb07792b1932c479");
    // Locate the gameinfo.txt
    let gameinfo_path = gameinfo_path(verbose)?;
    // Calculate the MD5 of the gameinfo.txt file
    let gameinfo_md5 = calculate_md5(&gameinfo_path).expect("Failed to calculate MD5");

    let gameinfo_backup_path = gameinfo_backup_path(verbose)?;

    // path for user's modified gameinfo to be held in
    let gameinfo_custom = gameinfo_path.with_extension("txt.custom");
    if var_os("DEBUG").is_some() || verbose {
        println!(
            "{} {} {:?}",
            "[D]".blue(),
            "gameinfo_backup_path:".bold(),
            gameinfo_backup_path
        );
    }
    if !gameinfo_backup_path.exists() {
        if gameinfo_md5 != gameinfo_orig_md5 {
            let err = format!(
                "gameinfo.txt file seems to be modified, but no backup is present!
                \n If you haven't already modified the gameinfo.txt, this is probably
                \n\ta {}'s bug you can report to the dev!
                \n\t\tYour gameinfo.txt MD5 hash is: {}",
                env!("CARGO_PKG_NAME"),
                gameinfo_md5
            );
            eprintln!("{} {}", "Error:".red(), err);
            return Err(Box::new(QuietErr(Some(err))));
        } else {
            // Create a backup of the gameinfo.txt file. Because why not?
            copy(&gameinfo_path, &gameinfo_backup_path)?;
            if !gameinfo_custom.exists() {
                let err = format!(
                    "gameinfo.txt is already at its default state, but no custom gameinfo.txt is to be found!
                    \n (PuG Mode is disabled, and gameinfo in unmodified vanilla state)"
                );
                eprintln!("{} {}", "Error:".red(), err);
                return Err(Box::new(QuietErr(Some(err))));
            } else {
                // Copy custom to gameinfo.txt
                if var_os("DEBUG").is_some() || verbose {
                    println!(
                        "{} Copying custom gameinfo.txt ({:?}) to {:?}",
                        "[D]".blue(),
                        &gameinfo_custom.file_name().unwrap().to_string_lossy(),
                        &gameinfo_path.file_name().unwrap().to_string_lossy()
                    );
                }
                copy(&gameinfo_custom, &gameinfo_path)?;
                if var_os("DEBUG").is_some() || verbose {
                    println!(
                        "{} Deleting unneeded custom gameinfo.txt copy ({:?})",
                        "[D]".blue(),
                        &gameinfo_custom.file_name().unwrap().to_string_lossy()
                    );
                }
                remove_file(&gameinfo_custom)?;
                println!("PuG Mode is now disabled.");
                return Ok(2);
            }
        }
    } else {
        if gameinfo_md5 != gameinfo_orig_md5 {
            // if gameinfo_custom.exists() {
            //     let err = format!(
            //         "gameinfo.txt file seems to be modified, and a custom gameinfo.txt is present!
            //         \n If you haven't already modified the gameinfo.txt, this is probably
            //         \n\ta {}'s bug you can report to the dev!
            //         \n\t\tYour gameinfo.txt MD5 hash is: {}",
            //         crate::env!("CARGO_PKG_NAME"),
            //         gameinfo_md5
            //     );
            //     eprintln!(
            //         "{} {}",
            //         "Error:".red(),
            //         err
            //     );
            //     return Err(Box::new(QuietErr(Some(err))));
            // }
            if var_os("DEBUG").is_some() || verbose {
                println!(
                    "{} Copying current custom gameinfo.txt ({:?}) to the custom backup {:?}",
                    "[D]".blue(),
                    &gameinfo_path.file_name().unwrap().to_string_lossy(),
                    &gameinfo_custom.file_name().unwrap().to_string_lossy()
                );
            }
            copy(&gameinfo_path, &gameinfo_custom)?;
            if var_os("DEBUG").is_some() || verbose {
                println!(
                    "{} Copying gameinfo.txt backup ({:?}) to {:?}",
                    "[D]".blue(),
                    &gameinfo_backup_path.file_name().unwrap().to_string_lossy(),
                    &gameinfo_path.file_name().unwrap().to_string_lossy()
                );
            }
            copy(&gameinfo_backup_path, &gameinfo_path)?;
            println!("PuG Mode is now enabled.");
            return Ok(1);
        } else {
            // Copy custom to gameinfo.txt
            if var_os("DEBUG").is_some() || verbose {
                println!(
                    "{} Copying custom gameinfo.txt ({:?}) to {:?}",
                    "[D]".blue(),
                    &gameinfo_custom.file_name().unwrap().to_string_lossy(),
                    &gameinfo_path.file_name().unwrap().to_string_lossy()
                );
            }
            copy(&gameinfo_custom, &gameinfo_path)?;
            if var_os("DEBUG").is_some() || verbose {
                println!(
                    "{} Deleting unneeded custom gameinfo.txt copy ({:?})",
                    "[D]".blue(),
                    &gameinfo_custom.file_name().unwrap().to_string_lossy()
                );
            }
            remove_file(&gameinfo_custom)?;
            println!("PuG Mode is now disabled.");
            return Ok(2);
        }
    }
}

fn PuG_mode_check(verbose: bool) -> Result<i32, Box<dyn std::error::Error>> {
    // Locate the gameinfo.txt file
    let gameinfo_path = gameinfo_path(verbose)?;

    // path for user's modified gameinfo to be held in
    let gameinfo_custom = gameinfo_path.with_extension("txt.custom");

    if !gameinfo_custom.exists() {
        println!("PuG Mode is currently disabled.");
        Ok(2)
    } else {
        println!("PuG Mode is currently enabled.");
        Ok(1)
    }
}

#[derive(Debug)]
pub struct QuietErr(Option<String>);
impl fmt::Display for QuietErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref msg) = self.0 {
            write!(f, "{}", msg)
        } else {
            write!(f, "")
        }
    }
}
impl std::error::Error for QuietErr {}

fn calculate_md5(filepath: &Path) -> Result<String, std::io::Error> {
    let mut file = BufReader::new(File::open(filepath)?); // Buffered for performance
    let mut hasher = Md5::new();
    let mut buffer = [0; 4096]; // Read in chunks for efficiency
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let result = hasher.finalize();
    Ok(format!("{:x}", result)) // Format as hexadecimal string
}

// fn clean_path(path: &str) -> Cow<str> {
//     if path.contains("/../") {
//         let path_buf = PathBuf::from(format!("/{path}"));
//         let Ok(absolute_path) = path_buf.parse_dot_from("/") else {
//             return path.into();
//         };
//         let path = absolute_path.to_str().unwrap().trim_start_matches('/');
//         String::from(path).into()
//     } else {
//         path.into()
//     }
// }

// #[test]
// fn test_clean_path() {
//     assert_eq!("foo/bar", clean_path("foo/bar"));
//     assert_eq!("foo/bar", clean_path("foo/asd/../bar"));
//     assert_eq!("../bar", clean_path("../bar"));
// }

fn l4d2_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let err = "Failed to find L4D2 install location".to_string();
    let err1 = "Failed to find Steam installation location".to_string();
    let err2 = "Failed to locate Left 4 Dead 2 directory".to_string();
    if let Some(path) = var_os("L4D2_DIR") {
        let path: PathBuf = path.into();
        if path.is_dir() {
            Ok(path)
        } else {
            eprintln!("{} {}: {}", "Error:".red(), err, err1);
            let e = format!("\n{}: \n{}", err, err1);
            return Err(Box::new(QuietErr(Some(e))));
        }
    } else {
        let (app, library) = SteamDir::locate()
            .map_err(|_| {
                let e = format!("{}: {}", err.clone(), err1);
                eprintln!("{} {}", "Error:".red(), e);
                QuietErr(Some(e))
            })?
            .find_app(550)
            .map_err(|_| {
                let e = format!("{}: {}", err.clone(), err2);
                eprintln!("{} {}", "Error:".red(), e);
                QuietErr(Some(e))
            })?
            .ok_or({
                let e = err.clone();
                eprintln!("{} {}", "Error:".red(), e);
                QuietErr(Some(e))
            })?;
        Ok(library.resolve_app_dir(&app))
    }
}

fn gameinfo_path(verbose: bool) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Locate the Left 4 Dead 2 directory
    if var_os("DEBUG").is_some() || verbose {
        println!("{} Locating L4D2 directory...", "[D]".blue());
    }
    let l4d2_dir = l4d2_path().expect("Failed to locate Left 4 Dead 2 directory");
    let gameinfo_path = l4d2_dir.join("left4dead2/gameinfo.txt");

    if !gameinfo_path.exists() {
        let err = format!("Unable to locate gameinfo.txt file. Is the game installation broken?");
        eprintln!("{} {}", "Error:".red(), err);
        return Err(Box::new(QuietErr(Some(err))));
    }

    if var_os("DEBUG").is_some() || verbose {
        println!(
            "{} {} {:?}",
            "[D]".blue(),
            "Gameinfo.txt path:".bold(),
            gameinfo_path
        );
    }
    Ok(gameinfo_path)
}

fn gameinfo_backup_path(verbose: bool) -> Result<PathBuf, Box<dyn std::error::Error>> {
    //let backup_path = PathBuf::new();
    let l4d2_dir = l4d2_path()?;
    if let Some(path) = var_os("BACKUP_PATH") {
        if var_os("DEBUG").is_some() || verbose {
            println!(
                "{} {}={:?}",
                "[D]".blue(),
                "env BACKUP_PATH".bold(),
                path.to_string_lossy()
            );
        }
        let path: PathBuf = path.into();
        if path.is_file() {
            Ok(path)
        } else {
            let err = "is not a file".to_string();
            eprintln!("{} {} {}", "Error:".red(), path.to_string_lossy(), err);
            let e = format!("\n{} \n{}", path.to_string_lossy(), err);
            return Err(Box::new(QuietErr(Some(e))));
        }
    } else if let Some(name) = var_os("BACKUP_NAME") {
        if var_os("DEBUG").is_some() || verbose {
            println!(
                "{} {}={:?}",
                "[D]".blue(),
                "env BACKUP_NAME".bold(),
                name.to_string_lossy()
            );
        }
        let name = name.to_string_lossy().to_string();
        let place = format!("left4dead2/{}", name);
        let place_invalid = format!("left4dead2/");
        if place_invalid == place {
            let err = "the backup place[from name!] does not contain any file name!".to_string();
            eprintln!("{} {}  <- {}", "Error:".red(), place, err);
            let e = format!("\n{} \n^-- {} --^", place, err);
            return Err(Box::new(QuietErr(Some(e))));
        } else {
            let backup_name = l4d2_dir.join(place);
            if backup_name.exists() {
                Ok(backup_name)
            } else {
                let err = "the backup file does not exist!".to_string();
                eprintln!(
                    "{} {} — {}",
                    "Error:".red(),
                    backup_name.to_string_lossy(),
                    err
                );
                let e = format!("\n{} \n^-- {} --^", backup_name.to_string_lossy(), err);
                Err(Box::new(QuietErr(Some(e))))
            }
        }
    } else {
        if var_os("DEBUG").is_some() || verbose {
            println!(
                "{} {}",
                "[D]".blue(),
                "No custom backup path is provided. Using default 'gameinfo.txt.orig'".bold(),
            );
        }
        let backup_def_path = l4d2_dir.join("left4dead2/gameinfo.txt.orig");
        Ok(backup_def_path)
    }
}

const HELP: Help = Help(sections!(
    [{env!("CARGO_PKG_NAME")} " " {env!("CARGO_PKG_VERSION")}]
    ["Use " c:"-h" " for short descriptions and " c:"--help" " for more details."]
    []
    "USAGE" {
        [{env!("CARGO_PKG_NAME")} " [OPTIONS] [<ARGS>]"]
    }
    "OPTIONS" {
        table Auto {
            "-f, --file <FILE>" => {
                ["VPK Addon file path"]
            }
            "-l, --list" => {
                ["List currently installed addons"]
            }
            "-n, --name <NAME>" => {
                ["Name for the addon that will be installed/updated or renamed (if already installed)"]
                Long ["a directory that will be named after," "and an entry in the gameinfo.txt.\n"
                "You can name it whatever you like, to later know what that addon is."]
            }
            "-u, --uninstall <NAME>" => {
                ["Uninstall the already installed addon"]
            }
            "-r, --rename <CURRENT_NAME>" => {
                ["Rename the already installed addon"]
            }
            "-p, --pug" => {
                ["Switches the PuG Mode (on/off)."]
                Long ["Temporarily backup the current gameinfo.txt and restore the vanilla one.\n"
                "Or vice versa.\n"
                "Useful for competitive PuG-type servers that enforce file consistency."]
            }
            "-P, --checkpug" => {
                ["Checks the PuG Mode status."]
            }
            "--reset" => {
                ["Reset the current gameinfo.txt to its default state using the existing backup"]
            }
            "-h, --help" => {
                ["Print help information"]
                Long ["Use " c:"-h" " for short descriptions and " c:"--help" " for more details."]
            }
            "-v, --verbose" => {
                ["Enable verbose output"]
                Long ["showing more details about the operations being performed."]
            }
            "-V, --version" => {
                ["Print version information"]
            }
//             "-W, --warnings <DIAGNOSTICS>" => {
//                 Short ["Disable certain warnings (disable all with " c:"-W0" ")"]
//                 Long ["Disable some or all warnings. A single warning can be disabled by specifying
// the name followed by " c:"=0" ", for example:

//     " c!"-Wcompat=0" "

// Multiple warnings can be disabled by setting this option multiple times, or
// using a comma-separated list:

//     " c!"-Wcompat=0 -Wdeprecated=0
//     -Wcompat=0,deprecated=0" "

// To disable all warnings, use " c:"-W0" ".

// Currently, the following warnings can be disabled:"]
//                 Long table Compact {
//                     "compat"     => { ["Compatibility warnings"] }
//                     "deprecated" => { ["A used feature will be removed in the future"] }
//                 }
//             }
        }
    }
    "EXAMPLE" {
        ["\t" g:{env!("CARGO_PKG_NAME")} " " C:"-f" " " c:"/home/user/Downloads/ion_vocalizer.vpk" " " C:"-n" " " c:"vocalizer"]
        [g:{env!("CARGO_PKG_NAME")} " " C:"-u" " " c:"vocalizer"]
        [g:{env!("CARGO_PKG_NAME")} " " C:"-r" " " c:"vocalizer" " " C:"-n" " " c:"ion_vocalizer"]
    }
    "ENVIRONMENT VARIABLES" {
        [c:"\tL4D2_DIR\n\t"
            "Directory where the game is installed"
        ]
        []
        [c:"BACKUP_PATH\n\t"
            "Path to the backup file (either to be created in or to be used in restore)"
        ]
        []
        [c:"BACKUP_NAME\n\t"
            "Name of the backup file residing in the left4dead2 directory \n\t(either to be created as or to be used in restore)\n\t"
            "If this is set, the selected backup file name will be used.\n\t"
            "If this is not set, the default name will be used: " m:"gameinfo.txt.orig"
        ]
        []
        [c:"DEBUG\n\t"
            "Enables verbose output\n\t"
            "(Equivalent to the \"-v\" option)"
        ]
    }
));

fn print_short_help(use_colors: bool) {
    #![allow(unused_must_use)]
    HELP.write(
        &mut std::io::stdout().lock(),
        false, // don't show long help
        use_colors,
    );
}

fn print_long_help(use_colors: bool) {
    #![allow(unused_must_use)]
    HELP.write(
        &mut std::io::stdout().lock(),
        true, // show long help
        use_colors,
    );
}
