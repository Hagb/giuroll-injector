extern crate winapi;

use injection::open_process;
use processes::find_process_id_by_name;
use std::{env, thread, time::Duration};

mod injection;
mod processes;

fn main() {
    'inject: {
        let pid = match find_process_id_by_name("th123.exe") {
            Some(pid) => pid,
            None => {
                eprintln!("Cannot find th123.exe");
                break 'inject;
            }
        };

        let process = match unsafe { open_process(pid) } {
            Ok(h) => h,
            Err(e) => {
                eprintln!("Failed to open process: {}. Error: {}", pid, e);
                break 'inject;
            }
        };

        let path = match env::current_dir() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Failed to get current dir. Error: {}", e);
                break 'inject;
            }
        };

        let path = path.join("giuroll_loader.dll");
        let path = match path.to_str() {
            Some(path) => path,
            None => {
                eprintln!(
                    "Failed to get path string, for encoding issue. How can it even happen???"
                );
                break 'inject;
            }
        };

        match unsafe { injection::inject_dll(process, path) } {
            Ok(h) => h,
            Err(e) => {
                eprintln!("Failed to inject DLL into process: {}. Error: {}", pid, e);
                break 'inject;
            }
        };
        println!(
            "Successfully injected giuroll_loader.dll into process: {}",
            pid
        );
    }
    println!("The injector will exit 10 seconds later.");
    thread::sleep(Duration::from_secs(10));
}
