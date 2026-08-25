// Don't pop up a console window alongside the app on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    gdrive_ui_lib::run()
}
