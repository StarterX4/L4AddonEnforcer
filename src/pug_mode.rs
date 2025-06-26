// SPDX-License-Identifier: LGPL-3.0-only
use crate::*;

pub fn PuG_mode_switch(verbose: bool) -> Result<i32, Box<dyn std::error::Error>> {
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

pub fn PuG_mode_check(verbose: bool) -> Result<i32, Box<dyn std::error::Error>> {
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