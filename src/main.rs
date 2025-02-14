#![allow(non_snake_case)]
use colored::Colorize;
use helptext::{Help, sections};
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
    let mut del_name = String::new();
    let gameinfo_orig_md5 = format!("586b3b0b39bc44ddfb07792b1932c479");
    let mut list = false;
    let mut verbose = false;
    let mut install = true;

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
                        exit(22)
                    }
                }
                "-l" | "--list" => {
                    install = false;
                    list = true;
                },
                "-n" | "--name" => {
                    if let Some(value) = iter.next() {
                        name = value.clone();
                    } else {
                        eprintln!("{} No value provided for {}", "Error:".red(), arg);
                        exit(22)
                    }
                }
                //"-R" | "--reset" => gameinfo_reset(),
                "-u" | "--uninstall" => {
                    if let Some(value) = iter.next() {
                        install = false;
                        del_name = value.clone();
                    } else {
                        eprintln!("Error: No addon name provided for {}", arg);
                        exit(22)
                    }
                },
                "-v" | "--verbose" => {
                    verbose = true;
                }
                "-h" => print_short_help(true),
                "--help" => print_long_help(true),
                _ => println!("Invalid argument: {}", args[1]),
            }
        }
    } else {
        println!("{} No options were passed!", "Error:".red());
        exit(22)
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
    if !addon_path.is_file() && install {
        eprintln!("{} Invalid addon file path!", "Error:".red());
        exit(2) // 'No such file or directory'
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
        exit(2)
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

    // List the installed custom addons
    if list {
        let mut SearchPaths = false;
        if lines
            .iter()
            .any(|line| line.contains("SearchPaths"))
            {
                println!("{}", "Installed addons:".bold());
                for line in lines.iter() {
                    if line.contains("SearchPaths") {
                        SearchPaths = true;
                    } else if SearchPaths && line.contains("}") {
                        SearchPaths = false;
                    }
                    if SearchPaths && line.contains("Game")
                    && !excl_addons.iter().any(|&s| line.contains(s)) {
                        println!("{}", line.trim_start_matches('\t').trim_start_matches("Game").replace("\t\t\t\t", "\t"));
                    }
                }
                return Ok(());
            }
            exit(0)
    }

    // Delete the selected addon
    if !del_name.is_empty() && !excl_addons.iter().any(|&s| del_name.contains(s))
    {
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
                println!("{} Removing line \n{} \nfrom gameinfo.txt", "[D]".blue(), index);
            }
        }
        else {
            eprintln!("{} {} not found in the gameinfo.txt file!", "Error:".red(), del_name);
            exit(22)
        }
        let del_addon_dir = l4d2_dir.join(format!("{}", del_name));
        if del_addon_dir.exists()
        {
            if del_addon_dir.is_dir()
            {
                std::fs::remove_dir_all(&del_addon_dir)?;
                println!("Uninstalled {} successfully.", del_name.italic());
                exit(0)
            }
            else {
                if var_os("DEBUG").is_some() || verbose {
                    println!("{} {} appears to not be a directory! (filesystem damaged? installation failed?)", "[D]".blue(), del_addon_dir.display());
                }
            }
        }
        exit(0)
    }
    else {
        if excl_addons.iter().any(|&s| del_name.contains(s))
        {
            eprintln!("{} Core game components cannot be uninstalled!", "Error:".red());
            exit(22)
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

    // The line to insert
    let new_line = format!("\t\t\tGame\t\t\t\t{}", name);


    if lines
        .iter()
        .any(|line| line.contains("Game") && line.contains(&name))
        && addon_dir_existed
    {
        println!("Updated {} successfully.", name.italic());
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


const HELP: Help = Help(sections!(
    [{env!("CARGO_PKG_NAME")} " " {env!("CARGO_PKG_VERSION")}]
    ["Use " c:"-h" " for short descriptions and " c:"--help" " for more details."]
    []
    "USAGE" {
        [{env!("CARGO_PKG_NAME")} " [OPTIONS] <ARGS>"]
    }
    "OPTIONS" {
        table Auto {
            "-a, -f, --addon, --file <PATH>" => {
                ["VPK Addon file path"]
            }
            "-l, --list" => {
                ["List currently installed addons"]
            }
            "-n, --name <NAME>" => {
                ["Name for the addon that will be installed/updated"]
                Long ["Name of the addon that will be installed/updated," "a directory that will be named after," "and an entry in the gameinfo.txt."
                "You can name it whatever you like, to later know what that addon is."]
            }
            "-u, --uninstall <NAME>" => {
                ["Uninstall the already installed addon"]
			}
            "-h, --help" => {
                ["Print help information"]
                Long ["Use " c:"-h" " for short descriptions and " c:"--help" " for more details."]
            }
			"-v, --verbose" => {
				["Enable verbose output"]
                Long ["Enable verbose output, showing more details about the operations being performed."]
			}
            // "-V, --version" => {
            //     ["Print version information"]
            // }
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
        [g:{env!("CARGO_PKG_NAME")} " " c:"-f /home/user/Downloads/ion_vocalizer.vpk -n vocalizer"]
		[g:{env!("CARGO_PKG_NAME")} " " c:"-u vocalizer"]
    }
));

fn print_short_help(use_colors: bool) {
	#![allow(unused_must_use)]
    HELP.write(
        &mut std::io::stdout().lock(),
        false,  // don't show long help
        use_colors,
    );
}

fn print_long_help(use_colors: bool) {
	#![allow(unused_must_use)]
    HELP.write(
        &mut std::io::stdout().lock(),
        true,  // show long help
        use_colors,
    );
}