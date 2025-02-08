use colored::Colorize;
// use path_dedot::ParseDot;
use md5::{Digest, Md5};
use std::{
    /* borrow::Cow, */ env,
    env::var_os,
    fmt::Debug,
    fs::{copy, create_dir_all, read_to_string, remove_file, write, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
    process::exit,
};
use steamlocate::SteamDir;
use thiserror::Error;

fn main() -> std::io::Result<()> {
    let mut addon_file = String::new();
    //let addon_path = PathBuf::new();
    let mut name = String::new();
    //let mut del_name = String::new();
    let gameinfo_orig_md5 = format!("586b3b0b39bc44ddfb07792b1932c479");
    let mut verbose = false;

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let mut iter = args.iter().skip(1);
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-a" | "-f" | "--addon" | "--file" => {
                    if let Some(value) = iter.next() {
                        addon_file = value.clone();
                    } else {
                        eprintln!("{} No addon.vpk file provided for {}", "Error:".red(), arg);
                        exit(22);
                    }
                }
                // "-d" | "--delete" => {
                //     if let Some(value) = iter.next() {
                //         del_name = value.clone();
                //     } else {
                //         eprintln!("Error: No addon name provided for {}", arg);
                //         exit(22);
                //     }
                // },
                "-n" | "--name" => {
                    if let Some(value) = iter.next() {
                        name = value.clone();
                    } else {
                        eprintln!("{} No value provided for {}", "Error:".red(), arg);
                        exit(22);
                    }
                }
                //"-R" | "--reset" => gameinfo_reset(),
                "-v" | "--verbose" => {
                    verbose = true;
                }
                // "-h" => print_short_help(true),
                // "--help" => print_long_help(true),
                _ => println!("Invalid argument: {}", args[1]),
            }
        }
    } else {
        println!("{} No options were passed!", "Error:".red());
        exit(22);
    }

    // Require both arguments on installation
    if (name.is_empty() && !addon_file.is_empty()) || (!name.is_empty() && addon_file.is_empty()) {
        eprintln!(
            "{} Both addon name and addon file path must be provided!",
            "Error:".red()
        );
        exit(22);
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
        eprintln!("{} Invalid addon name!", "Error:".red());
        eprintln!("\tName cannot be empty, contain whitespace, or special characters");
        eprintln!("\tthat are known to cause problems with file managers or filesystems.");
        exit(36); // 'File name too long' xd
    }

    // Validate addon file
    let addon_path = PathBuf::from(&addon_file);
    if var_os("DEBUG").is_some() {
        println!("{} {} {:?}", "[D]".blue(), "Addon path:".bold(), addon_path);
    }
    if !addon_path.is_file() {
        eprintln!("{} Invalid addon file path!", "Error:".red());
        exit(2); // 'No such file or directory'
    }

    // Locate the Left 4 Dead 2 directory
    if var_os("DEBUG").is_some() || verbose {
        println!("{} Locating L4D2 directory...", "[D]".blue());
    }
    let l4d2_dir = l4d2_path().expect("Failed to locate Left 4 Dead 2 directory");
    let gameinfo_path = l4d2_dir.join("left4dead2/gameinfo.txt");

    if !gameinfo_path.exists() {
        eprintln!(
            "{} Unable to locate gameinfo.txt file. Is the game installation broken?",
            "Error:".red()
        );
        exit(2);
    }

    if var_os("DEBUG").is_some() || verbose {
        println!(
            "{} {} {:?}",
            "[D]".blue(),
            "Gameinfo.txt path:".bold(),
            gameinfo_path
        );
    }
    // Calculate the MD5 of the gameinfo.txt file
    let gameinfo_md5 = calculate_md5(&gameinfo_path).expect("Failed to calculate MD5");

    let gameinfo_backup_path = gameinfo_path.with_extension("txt.orig");
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
                crate::env!("CARGO_PKG_NAME")
            );
            println!(
				"\t\tYour gameinfo.txt MD5 hash is: {}",
				gameinfo_md5.bold());
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

    // Create the new addon directory
    if var_os("DEBUG").is_some() || verbose {
        println!("{} Creating addon directory: {}", "[D]".blue(), name);
    }
    let addon_dir = l4d2_dir.join(format!("{}", name));
    let mut addon_dir_existed = false;
    if addon_dir.exists() {
        addon_dir_existed = true;
        if var_os("DEBUG").is_some() || verbose {
            println!("{} {} An addon directory with the same name already exists! Assuming this is an update.",
														"[D]".blue(),
														"Warning:".yellow());
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

    // Read the gameinfo.txt file
    let contents = read_to_string(&gameinfo_path)?;

    // The line to insert
    let new_line = format!("\t\t\tGame\t\t\t\t{}", name);

    // Split the file into lines
    let mut lines: Vec<&str> = contents.lines().collect();

    if lines
        .iter()
        .any(|line| line.contains("Game") && line.contains(&name))
        && addon_dir_existed
    {
        println!("Updated {} successfully.", name);
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
        println!("Installed {} successfully.", name);
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Failed to find L4D2 install location")]
    L4D2NotFound,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

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

fn l4d2_path() -> Result<PathBuf, LoaderError> {
    if let Some(path) = var_os("L4D2_DIR") {
        let path: PathBuf = path.into();
        if path.is_dir() {
            Ok(path)
        } else {
            Err(LoaderError::L4D2NotFound)
        }
    } else {
        let (app, library) = SteamDir::locate()
            .map_err(|_| LoaderError::L4D2NotFound)?
            .find_app(550)
            .map_err(|_| LoaderError::L4D2NotFound)?
            .ok_or(LoaderError::L4D2NotFound)?;
        Ok(library.resolve_app_dir(&app))
    }
}

// fn deletion () {
//     // Locate the Left 4 Dead 2 directory
//     if var_os("DEBUG").is_some() {println!("Locating L4D2 directory...");}
//     let l4d2_dir = l4d2_path().expect("Failed to locate Left 4 Dead 2 directory");
//     let gameinfo_path = l4d2_dir.join("left4dead2/gameinfo.txt");

//     if !gameinfo_path.exists() {
//         eprintln!("Error: Unable to locate gameinfo.txt file. Is the game installation broken?");
//         exit(2);
//     }
//         // Create the new addon directory
//         if var_os("DEBUG").is_some() {println!("Creating addon directory: {}", del_name);}
//     let addon_dir = l4d2_dir.join(format!("{}", del_name));
//     let mut addon_dir_existed = false;
//     if addon_dir.exists() {
//         addon_dir_existed = true;
//             if var_os("DEBUG").is_some() {println!("Warning: An addon directory with the same name already exists! Assuming this is an update.");}
//     }
// }
// fn gameinfo_reset()  {
//     if &gameinfo_backup_path.exists() {
//         copy(&gameinfo_backup_path, &gameinfo_path)?;
//     Ok(());
//     }
// }
