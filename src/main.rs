#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::fs;
use std::process::Command;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use slint::{ModelRc, SharedString, VecModel};

mod ipcalc;

slint::include_modules!();

const DEFAULT_IPV4: &str = "192.168.1.1/22";
const DEFAULT_IPV6: &str = "fd00::1/64";
const DEFAULT_RANGE: &str = "1-20";

#[derive(Debug)]
struct UiState {
    ipv4_input: String,
    ipv4_range: String,
    ipv6_input: String,
    ipv6_range: String,
}

fn main() -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;
    let state = Rc::new(RefCell::new(UiState {
        ipv4_input: DEFAULT_IPV4.to_owned(),
        ipv4_range: DEFAULT_RANGE.to_owned(),
        ipv6_input: DEFAULT_IPV6.to_owned(),
        ipv6_range: DEFAULT_RANGE.to_owned(),
    }));

    app.set_dark_mode(true);
    app.set_ipv6_mode(false);
    app.set_input_value(DEFAULT_IPV4.into());
    app.set_range_value(DEFAULT_RANGE.into());
    update_output(&app);

    {
        let app_weak = app.as_weak();
        app.on_recalculate(move || {
            if let Some(app) = app_weak.upgrade() {
                update_output(&app);
            }
        });
    }

    {
        let app_weak = app.as_weak();
        let state = Rc::clone(&state);
        app.on_select_version(move |is_ipv6| {
            if let Some(app) = app_weak.upgrade() {
                {
                    let mut state = state.borrow_mut();
                    let current_input = app.get_input_value().to_string();
                    let current_range = app.get_range_value().to_string();

                    if app.get_ipv6_mode() {
                        state.ipv6_input = current_input;
                        state.ipv6_range = current_range;
                    } else {
                        state.ipv4_input = current_input;
                        state.ipv4_range = current_range;
                    }

                    let (next_input, next_range) = if is_ipv6 {
                        (state.ipv6_input.clone(), state.ipv6_range.clone())
                    } else {
                        (state.ipv4_input.clone(), state.ipv4_range.clone())
                    };

                    app.set_ipv6_mode(is_ipv6);
                    app.set_input_value(next_input.into());
                    app.set_range_value(next_range.into());
                }

                update_output(&app);
            }
        });
    }

    {
        let app_weak = app.as_weak();
        app.on_toggle_theme(move || {
            if let Some(app) = app_weak.upgrade() {
                app.set_dark_mode(!app.get_dark_mode());
            }
        });
    }

    {
        let app_weak = app.as_weak();
        app.on_open_range_list(move || {
            if let Some(app) = app_weak.upgrade() {
                open_range_list(&app);
            }
        });
    }

    app.run()
}

fn update_output(app: &AppWindow) {
    let version = if app.get_ipv6_mode() {
        ipcalc::IpVersion::Ipv6
    } else {
        ipcalc::IpVersion::Ipv4
    };

    match ipcalc::calculate(app.get_input_value().as_str(), version) {
        Ok(calculation) => {
            let rows: Vec<OutputLine> = calculation
                .lines
                .into_iter()
                .map(|line| OutputLine {
                    label: SharedString::from(line.label),
                    value: SharedString::from(line.value),
                    tone: line.tone.as_i32(),
                    value_offset: output_value_offset(line.label),
                })
                .collect();
            app.set_has_error(false);
            app.set_error_message(SharedString::new());
            app.set_output_lines(model_from_rows(rows));
        }
        Err(error) => {
            app.set_has_error(true);
            app.set_error_message(error.into());
            app.set_output_lines(model_from_rows(Vec::new()));
        }
    }
}

fn open_range_list(app: &AppWindow) {
    let range_input = app.get_range_value().to_string();

    if range_input.trim().is_empty() {
        app.set_has_error(true);
        app.set_error_message("Enter an IP range, for example 1-20.".into());
        return;
    }

    let version = if app.get_ipv6_mode() {
        ipcalc::IpVersion::Ipv6
    } else {
        ipcalc::IpVersion::Ipv4
    };

    match ipcalc::calculate_range_addresses(app.get_input_value().as_str(), &range_input, version) {
        Ok(addresses) => match open_addresses_in_notepad(&addresses) {
            Ok(()) => update_output(app),
            Err(error) => {
                app.set_has_error(true);
                app.set_error_message(error.into());
            }
        },
        Err(error) => {
            app.set_has_error(true);
            app.set_error_message(error.into());
        }
    }
}

fn open_addresses_in_notepad(addresses: &[String]) -> Result<(), String> {
    let mut path = std::env::temp_dir();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "System clock is before the Unix epoch.".to_owned())?
        .as_millis();

    path.push(format!("ipcalc-range-{}-{stamp}.txt", std::process::id()));
    fs::write(&path, addresses.join("\r\n"))
        .map_err(|error| format!("Could not write the range list: {error}"))?;

    Command::new("notepad.exe")
        .arg(&path)
        .spawn()
        .map_err(|error| format!("Could not open Notepad: {error}"))?;

    Ok(())
}

fn model_from_rows(rows: Vec<OutputLine>) -> ModelRc<OutputLine> {
    ModelRc::from(Rc::new(VecModel::from(rows)))
}

fn output_value_offset(label: &str) -> f32 {
    const MONO_CHAR_WIDTH_PX: f32 = 8.8;

    (label.chars().count() as f32 + 3.0) * MONO_CHAR_WIDTH_PX
}
