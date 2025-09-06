// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod device_hotplug;
mod library;
mod db;

fn main() {
  app_lib::run();
}
