// SPDX-License-Identifier: LGPL-3.0-only

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use fltk::{browser::{Browser, BrowserType, MultiBrowser}, enums::{Align, Color, *}, frame::Frame, group::{Flex, Pack, PackType}, prelude::*, *};
use crate::gui_theming::*;
use std::{path::{PathBuf, Path}, sync::{Arc, Mutex}};


pub fn main() {
	let a = app::App::default();
	apply_theme();
	let mut win = window::Window::default()
		.with_size(400, 600)
		.with_label(env!("CARGO_PKG_NAME"))
		.center_screen();

	let vpack = Pack::new(30,5,360,300,"");
	let mut title_text = Frame::new(10,0, 64, 20, "Installation");
	title_text.activate();
	title_text.set_label_color(Color::Light3);
	title_text.set_align(Align::Inside | Align::Left);

	let mut title_text = Frame::new(0,0, 64, 10, "");
	title_text.activate();

	let mut title_text = Frame::new(0,0, 64, 20, "Addon Name:");
	title_text.activate();
	title_text.set_align(Align::Center);

	let mut hpack = Pack::new(0,0,340,32,"");
	let name_for_selected = Arc::new(Mutex::new(RInput::new(20, 20, 340, 20, "Addon Name")));
	hpack.end();
	hpack.set_type(PackType::Horizontal);

	let mut title_text = Frame::new(0,0, 64, 20, "VPK Addon File:");
	title_text.activate();
	title_text.set_align(Align::Center);

	let mut hpack = Pack::new(0,0,340,32,"");
	let selected_file = Arc::new(Mutex::new(RInput::new(20, 20, 255, 40, "Addon File")));
	let mut btn_browse = RButton::new(0,0,82,32,"Browse...");
	
	let selected_file_clone = Arc::clone(&selected_file);
	btn_browse.set_callback(move |_| {
		if let Some(file_path) = nfc_get_file(dialog::NativeFileChooserType::BrowseFile) {
			let mut selected_file = selected_file_clone.lock().unwrap();
			selected_file.set_value(file_path.to_str().unwrap_or(""));
		}
	});

	hpack.end();
	hpack.set_type(PackType::Horizontal);
	hpack.set_spacing(2);

	let mut title_text = Frame::new(0,0, 64, 10, "");
	title_text.activate();
	
	let mut hpack = Pack::new(0,0,340,32,"");

	let mut btn_install = RButton::new(0,0,340,32,"Install");
	btn_install.set_callback(move |_| {
		let selected_file_clone = Arc::clone(&selected_file);
		let name_for_selected_clone = Arc::clone(&name_for_selected);
		let addon_file = selected_file_clone.lock().unwrap().value();
		let name = name_for_selected_clone.lock().unwrap().value();
		if let Some(_file_path) = Path::new(&addon_file).parent() {
			if !name.is_empty() {
				if name.contains(char::is_whitespace)
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
						dialog::alert(center().0 - 200, center().1 - 100, &format!("Invalid addon name!\n\tName cannot be empty, contain whitespace, or special characters\n\tthat are known to cause problems with file managers or filesystems."));
					} else {
						match crate::install_addon(&addon_file, &name, false) {
							Ok(1) => {
								dialog::message(center().0 - 200, center().1 - 100, &format!("Addon \"{}\" installed successfully!", name));
							},
							Ok(2) => {
								dialog::message(center().0 - 200, center().1 - 100, &format!("Addon \"{}\" updated successfully!", name));
							},
							Ok(other) => {
								dialog::alert(center().0 - 200, center().1 - 100, &format!("Addon \"{}\" completed with status: {}", name, other));
							},
							Err(e) => {
								dialog::alert(center().0 - 200, center().1 - 100, &format!("Failed to install addon \"{}\": {}", name, e));
							},
						}
					}
			} else {
				dialog::alert(center().0 - 200, center().1 - 100, "No addon name specified!");
			}
		} else {
			dialog::alert(center().0 - 200, center().1 - 100, "Invalid addon file path!");
		}
	});

	hpack.end();
	hpack.set_type(PackType::Horizontal);

	let mut title_text = Frame::new(0,0, 64, 10, "");
	title_text.activate();
	let mut title_text = Frame::new(0,0, 64, 10, "");
	title_text.activate();
	let mut title_text = Frame::new(0,0, 64, 10, "");
	title_text.activate();

	let mut title_text = Frame::new(10,0, 64, 20, "Currently Installed");
	title_text.activate();
	title_text.set_label_color(Color::Light3);
	title_text.set_align(Align::Inside | Align::Left);

	let mut title_text = Frame::new(0,0, 64, 10, "");
	title_text.activate();

	let mut hpack = Pack::new(0,0,340,320,"");
	let mut installed_list = Browser::new(0, 0, 340, 400, "");
	installed_list.set_color(Color::from_rgb(22, 25, 37));
	installed_list.set_selection_color(Color::from_rgb(185, 5, 224));
	installed_list.set_type(BrowserType::Hold);
	
	// Call list_addons and populate the browser
	// Capture the output from list_addons
	let mut output = Vec::new();
	if let Err(e) = crate::list_addons(true, false, &mut output) {
		installed_list.set_type(BrowserType::Normal);
		installed_list.add(&format!("Failed to list addons:"));
		installed_list.add(&format!("{}", e));
	} else {
		let output_str = String::from_utf8(output).unwrap_or_default();
		for line in output_str.lines() {
			installed_list.add(line.trim_start_matches('\t'));
		}
	}
	hpack.end();
	hpack.set_type(PackType::Horizontal);

	let mut flex2 = Flex::new(0, 0, 82, 42, "").row();
    flex2.set_margin(10);
	// let mut hpack = Pack::new(0,0,120,32,"");
	let mut spacer = frame::Frame::default().with_size(82, 32);
	spacer.activate();
	let mut spacer1 = frame::Frame::default().with_size(82, 32);
	spacer1.activate();
	let mut btn_del = RButton::new(0,0,82,32,"Uninstall");

	let mut installed_list_clone = installed_list.clone();
    btn_del.set_callback(move |_| {
        let selected = installed_list_clone.value();
        if selected > 0 {
            let addon_name = installed_list_clone.text(selected).unwrap();
            // Perform uninstall action
			if !addon_name.is_empty() {
				match crate::uninstall_addon(&addon_name, false) {
					Ok(_) => {
           				dialog::alert(center().0 - 200, center().1 - 100, &format!("Addon \"{}\" uninstalled successfully!", addon_name));
						installed_list_clone.remove(selected);
						},
					Err(e) => {
                        dialog::alert(center().0 - 200, center().1 - 100, &format!("Failed to uninstall addon \"{}\": {}", addon_name, e));
                    },
                }
			}
			else {
                dialog::alert(center().0 - 200, center().1 - 100, "No addon selected to uninstall!");
				
            }
        }
    });
	
	// hpack.end();
	// hpack.set_type(PackType::Horizontal);
	// hpack.with_align(Align::Right);
	flex2.set_spacing(2);
    flex2.end();

	vpack.end();

	win.end();
	//win.make_resizable(true);
	win.show();

	a.run().unwrap();
}

pub fn center() -> (i32, i32) {
	(
		(app::screen_size().0 / 2.0) as i32,
		(app::screen_size().1 / 2.0) as i32,
	)
}

fn nfc_get_file(mode: dialog::NativeFileChooserType) -> Option<PathBuf> {
	let mut nfc = dialog::NativeFileChooser::new(mode);
	if mode == dialog::NativeFileChooserType::BrowseSaveFile {
		nfc.set_option(dialog::NativeFileChooserOptions::SaveAsConfirm);
	} else if mode == dialog::NativeFileChooserType::BrowseFile {
		nfc.set_option(dialog::NativeFileChooserOptions::NoOptions);
		nfc.set_filter("*.vpk");
	}
	match nfc.try_show() {
		Err(e) => {
			eprintln!("{}", e);
			None
		}
		Ok(a) => match a {
			dialog::NativeFileChooserAction::Success => {
				let name = nfc.filename();
				if name.as_os_str().is_empty() {
					dialog::alert(center().0 - 200, center().1 - 100, "Please specify a file!");
					None
				} else {
					Some(name)
				}
			}
			dialog::NativeFileChooserAction::Cancelled => None,
		},
	}
}