// SPDX-License-Identifier: LGPL-3.0-only

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use fltk::{app::version, browser::{Browser, BrowserType}, enums::{Align, Color, Event, Shortcut}, frame::Frame, group::{Flex, Pack, PackType}, menu::SysMenuBar, prelude::*, *};
use crate::gui_theming::*;
use std::{path::{PathBuf, Path}, sync::{Arc, Mutex}};


pub fn main() {
	let a = app::App::default();
	apply_theme();
	let mut win = window::Window::default()
		.with_size(400, 685)
		.with_label(env!("CARGO_PKG_NAME"))
		.center_screen();

	let mut menubar = SysMenuBar::new(294, 0, 80, 24, "");
	menubar.add(
		"&Program/&About",
		Shortcut::None,
		menu::MenuFlag::Normal,
		|_menu| {
			let ver = env!(
				"CARGO_PKG_VERSION"
			);
			dialog::message_default(&format!("\t\tL4AddonEnforcer\n \t\tv{} | using FLTK v{} \n https://github.com/StarterX4/L4AddonEnforcer \n\nA gameinfo.txt—way addon manager for Left 4 Dead 2.", ver, version()));
		},
	);
	menubar.add(
		"&Program/&Quit\t",
		Shortcut::Ctrl | 'q',
		menu::MenuFlag::Normal,
		|_menu| {
			std::process::exit(0);
		},
	);

	let vpack = Pack::new(30,15,360,300,"");
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

	let mut flex2 = Flex::new(0, 0, 82, 42, "").row();
    flex2.set_margin(10);
	let mut spacer = frame::Frame::default().with_size(82, 32);
	spacer.activate();
	let mut spacer1 = frame::Frame::default().with_size(82, 32);
	spacer1.activate();
	let mut btn_refresh = RButton::new(0,0,82,32,"Refresh");
	flex2.set_spacing(2);
    flex2.end();

	let mut hpack = Pack::new(0,0,340,320,"");
	let mut installed_list = Browser::new(0, 0, 340, 400, "");
	installed_list.set_color(Color::from_rgb(22, 25, 37));
	installed_list.set_selection_color(Color::from_rgb(185, 5, 224));
	installed_list.set_type(BrowserType::Hold);

	// Call list_addons and populate the browser
	// Capture the output from list_addons
	let mut output = Vec::new();
	if let Err(e) = crate::list_addons::list_addons(true, false, false, &mut output) {
		installed_list.set_type(BrowserType::Normal);
		installed_list.add(&format!("Failed to list addons:"));
		installed_list.add(&format!("{}", e));
	} else {
		let output_str = String::from_utf8(output).unwrap_or_default();
		if !output_str.is_empty() {
			for line in output_str.lines() {
				installed_list.add(line.trim_start_matches('\t'));
			}
		} else {
			installed_list.set_type(BrowserType::Normal);
            installed_list.add(&format!("@c@iNo addons are currently installed."));
            installed_list.add(&format!("@c@iWould you like to install one (or more)?"));
		}
	}
	hpack.end();
	hpack.set_type(PackType::Horizontal);

	let installed_list = Arc::new(Mutex::new(installed_list));
	let installed_list_clone = Arc::clone(&installed_list);
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
						let installed_list_ref = Arc::clone(&installed_list_clone);
						match crate::install_addon::install_addon(&addon_file, &name, false) {
							Ok(1) => {
								dialog::message(center().0 - 200, center().1 - 100, &format!("Addon \"{}\" installed successfully!", name));
								refresh_installed_list(&installed_list_ref);
							},
							Ok(2) => {
								dialog::message(center().0 - 200, center().1 - 100, &format!("Addon \"{}\" updated successfully!", name));
								refresh_installed_list(&installed_list_ref);
							},
							Ok(other) => {
								dialog::alert(center().0 - 200, center().1 - 100, &format!("Addon \"{}\" completed with status: {}", name, other));
								refresh_installed_list(&installed_list_ref);
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

	let installed_list_clone = Arc::clone(&installed_list);
	btn_refresh.set_callback(move |_| {
        let installed_list_ref = Arc::clone(&installed_list_clone);
		refresh_installed_list(&installed_list_ref);
    });

	let mut flex2 = Flex::new(0, 0, 82, 42, "").row();
    flex2.set_margin(10);
	// let mut hpack = Pack::new(0,0,120,32,"");
	let mut spacer = frame::Frame::default().with_size(82, 32);
	spacer.activate();

	let mut btn_ren = RButton::new(0,0,82,32,"Rename");
	let installed_list_clone = Arc::clone(&installed_list);
	btn_ren.set_callback(move |_| {
		let installed_list = installed_list_clone.lock().unwrap();
        let selected = installed_list.value();
		drop(installed_list); // Release the lock
        if selected > 0 {
            let addon_name = installed_list_clone.lock().unwrap().text(selected).unwrap();
            // Perform uninstall action
			if !addon_name.is_empty() {
				let ctrl = controlAccentColor.to_rgb();
				let tmp = controlColor.to_rgb();
				app::background2(tmp.0, tmp.1, tmp.2);
				let inp_box = dialog::input(center().0 - 200, center().1 - 100, &format!("Enter new name for addon: {}", addon_name), "");
					if !inp_box.is_none() {
						let input = inp_box.unwrap();
						match crate::rename_addon::rename_addon(&addon_name, &input, false) {
							Ok(_) => {
								dialog::alert(center().0 - 200, center().1 - 100, &format!("Addon \"{}\" renamed successfully!", addon_name));
								refresh_installed_list(&installed_list_clone);
								},
							Err(e) => {
								dialog::alert(center().0 - 200, center().1 - 100, &format!("Failed to rename addon \"{}\": {}", addon_name, e));
							},
						}
					}
			app::background2(ctrl.0, ctrl.1, ctrl.2);
			} else {
                dialog::alert(center().0 - 200, center().1 - 100, "No addon selected to rename!");
            }
		}
    });

	let mut btn_del = RButton::new(0,0,82,32,"Uninstall");

	let installed_list_clone = Arc::clone(&installed_list);
    btn_del.set_callback(move |_| {
        let mut installed_list = installed_list_clone.lock().unwrap();
        let selected = installed_list.value();
        if selected > 0 {
            let addon_name = installed_list.text(selected).unwrap();
            // Perform uninstall action
			if !addon_name.is_empty() {
				match crate::uninstall_addon::uninstall_addon(&addon_name, false) {
					Ok(_) => {
           				dialog::alert(center().0 - 200, center().1 - 100, &format!("Addon \"{}\" uninstalled successfully!", addon_name));
						installed_list.remove(selected);
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

	let mut flex3 = Flex::new(0, 0, 82, 42, "").row();
    flex3.set_margin(10);

	let mut btn_pug = RButton::new(0,0,82,32,"PuG mode: Unknown");
	match crate::pug_mode::PuG_mode_check(false) {
		Err(_e) => {
		btn_pug.set_label("PuG mode is unavailable");
		btn_pug.deactivate();
		}, 
		Ok(1) => {
		btn_pug.set_label("PuG mode: Enabled");
		},
		Ok(2) => {
		btn_pug.set_label("PuG mode: Disabled");
		},
		Ok(_other) => {
		btn_pug.set_label("PuG mode: Unknown");
		},
	}
	let installed_list_clone = Arc::clone(&installed_list);
	let mut btn_pug_clone = btn_pug.clone();
	btn_pug.set_callback(move |_| {
		match crate::pug_mode::PuG_mode_switch(false) {
			Ok(1) => {
				   dialog::alert(center().0 - 200, center().1 - 100, &format!("PuG Mode is now enabled."));
				   btn_pug_clone.set_label("PuG mode: Enabled");
				   refresh_installed_list(&installed_list_clone);
				},
			Ok(2) => {
				dialog::alert(center().0 - 200, center().1 - 100, &format!("PuG Mode is now disabled."));
				btn_pug_clone.set_label("PuG mode: Disabled");
				refresh_installed_list(&installed_list_clone);
            },
			Ok(other) => {
				dialog::alert(center().0 - 200, center().1 - 100, &format!("Failed to switch PuG Mode: {}", other));
				btn_pug_clone.set_label("PuG mode: Unknown");
				refresh_installed_list(&installed_list_clone);
            },
			Err(e) => {
				dialog::alert(center().0 - 200, center().1 - 100, &format!("Failed to change PuG Mode: {}", e));
			},
		}
	});

	let mut btn_reset = RButton::new(0,0,82,32,"gameinfo.txt reset");
	btn_reset.set_color(Color::from_rgb(168, 6, 44));
	btn_reset.set_selection_color(Color::from_rgb(246, 25, 76));
	btn_reset.handle(move |b, ev| match ev {
		Event::Enter => {
			//b.set_frame(OS_DEFAULT_HOVERED_UP_BOX);
			b.set_color(Color::from_rgb(246, 25, 76));
			b.redraw();
			true
		}
		Event::Leave => {
			//b.set_frame(OS_DEFAULT_BUTTON_UP_BOX);
			b.set_color(Color::from_rgb(168, 6, 44));
			b.redraw();
			true
		}
		_ => false,
	});

	let installed_list_clone = Arc::clone(&installed_list);
	btn_reset.set_callback(move |_| {
		match dialog::choice2(center().0 - 200, center().1 - 100,
			"Are you sure you want to reset gameinfo.txt to default? \nThis operation cannot be undone.", "&Yes", "&No", "") {
			Some(0) => {
				match crate::gameinfo_reset(false) {
					Ok(_) => {
						dialog::alert(center().0 - 200, center().1 - 100, &format!("Succesfully reset gameinfo.txt to default."));
						refresh_installed_list(&installed_list_clone);
						},
					Err(e) => {
						dialog::alert(center().0 - 200, center().1 - 100, &format!("Failed to reset gameinfo.txt: {}", e));
					},
				}
			},
			Some(1) => (),
			Some(_) | None  => (),
		}
	});

	flex3.set_spacing(2);
    flex3.end();

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

fn refresh_installed_list(installed_list_clone: &Arc<Mutex<Browser>>) {
    let mut installed_list = installed_list_clone.lock().unwrap();
    // Call list_addons and populate the browser
    // Capture the output from list_addons
    let mut output = Vec::new();
    if let Err(e) = crate::list_addons::list_addons(true, false, false, &mut output) {
        installed_list.set_type(BrowserType::Normal);
        installed_list.add(&format!("@bFailed to list addons:"));
        installed_list.add(&format!("{}", e));
    } else {
        installed_list.clear();
		if installed_list.get_type::<BrowserType>() == BrowserType::Normal {
			installed_list.set_type(BrowserType::Hold);
		}
        let output_str = String::from_utf8(output).unwrap_or_default();
		if !output_str.is_empty() {
			for line in output_str.lines() {
				installed_list.add(line.trim_start_matches('\t'));
			}
		} else {
			installed_list.set_type(BrowserType::Normal);
            installed_list.add(&format!("@c@iNo addons are currently installed."));
            installed_list.add(&format!("@c@iWould you like to install one (or more)?"));
		}
    }
}
