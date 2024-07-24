extern crate winapi;

use core::str;
use injection::open_process;
// use winapi::shared::ntdef::HANDLE;
// use processes::find_process_id_by_name;
use std::{env, process::Command, thread, time::Duration};

mod injection;

unsafe fn hook() {
    // Use tasklist instead of Windows API to get pid, to avoid being detected as virus.
    let pid = match Command::new("tasklist")
        .args(["/fo", "csv", "/nh"])
        .output()
    {
        Ok(s) => {
            let mut pids: Vec<u32> = Vec::new();
            for line in s.stdout.split(|x| *x == b'\n' || *x == b'\r') {
                for i in [b"\"th123.exe\",", b"th123.exe,"] as [&[u8]; 2] {
                    if line.starts_with(i) {
                        let stripped = match &line[i.len()..] {
                            s => &s[(s[0] == b'"') as usize..],
                        };
                        let pid_str = stripped.split(|x| !x.is_ascii_digit()).next().unwrap();
                        let pid: u32 =
                            u32::from_str_radix(str::from_utf8(pid_str).unwrap(), 10).unwrap();
                        pids.push(pid);
                    }
                }
            }
            match pids.len() {
                0 => {
                    eprintln!("Cannot found th123.exe process!");
                    return;
                }
                1 => pids[0],
                n => {
                    eprintln!(
                        "Found {} th123.exe processes. Please use SWRSToys instead.",
                        n
                    );
                    return;
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to run tasklist to get pid of th123.exe: {}", e);
            return;
        }
    };

    let process = match unsafe { open_process(pid) } {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to open process: {}. Error: {}", pid, e);
            return;
        }
    };

    let path = match env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to get current dir. Error: {}", e);
            return;
        }
    };

    let path = path.join("giuroll_loader_dll.dll");
    let path = match path.to_str() {
        Some(path) => path,
        None => {
            eprintln!("Failed to get path string, for encoding issue. How can it even happen???");
            return;
        }
    };

    match unsafe { injection::inject_dll(process, path) } {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to load DLL into process: {}. Error: {}", pid, e);
            return;
        }
    };
    println!(
        "Successfully loaded giuroll_loader_dll.dll into process: {}",
        pid
    );
}

fn main() {
    unsafe { hook() };
    println!("The loader will exit 10 seconds later.");
    thread::sleep(Duration::from_secs(10));
}
