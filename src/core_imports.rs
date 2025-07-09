// SPDX-License-Identifier: LGPL-3.0-only
pub use clap::Parser;
pub use colored::Colorize;
pub use helptext::{Help, sections};
// use path_dedot::ParseDot;
pub use md5::{Digest, Md5};
pub use std::{
	env::var_os,
	error::Error,
	fmt::{self, Debug},
	fs::{File, copy, create_dir_all, read_to_string, remove_file, write},
	io::{BufReader, Read, Write},
	path::{Path, PathBuf},
	process::exit,
};
pub use steamlocate::SteamDir;
