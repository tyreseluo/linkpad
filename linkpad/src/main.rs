// This cfg option hides the command prompt console window on Windows.
// TODO: move this into Makepad itself as an addition to the `MAKEPAD` env var.
#![cfg_attr(
    all(feature = "hide_windows_console", target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    linkpad::app::app_main();
}
