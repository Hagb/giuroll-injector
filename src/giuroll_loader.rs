use std::ffi::{CStr, CString};
use std::{os::windows::ffi::OsStringExt, path::Path, ptr::null_mut, thread};

use winapi::shared::minwindef::FARPROC;
use winapi::{
    shared::minwindef::{DWORD, HINSTANCE, LPVOID},
    um::{
        errhandlingapi::GetLastError,
        libloaderapi::{GetModuleFileNameW, GetProcAddress, LoadLibraryW},
        winnt::DLL_PROCESS_ATTACH,
        winuser::{MessageBoxA, MB_ICONERROR},
    },
};

#[no_mangle]
pub unsafe extern "stdcall" fn DllMain(module: HINSTANCE, reason: DWORD, _: LPVOID) -> i32 {
    match reason {
        DLL_PROCESS_ATTACH => {
            println!("loader DllMain");
            let mut dat = [0u16; 1025];
            GetModuleFileNameW(module, dat.as_mut_ptr(), dat.len().try_into().unwrap());
            let null_pos = dat.iter().position(|x| *x == 0).unwrap();
            let path_ = std::ffi::OsString::from_wide(&dat[0..null_pos]);
            let giuroll_path = Path::new(&path_).with_file_name("giuroll.dll");
            if !giuroll_path.exists() {
                eprintln!("{:?} not exists", giuroll_path);
                return 0;
            }
            let giuroll_path = giuroll_path
                .to_str()
                .unwrap()
                .encode_utf16()
                .chain([0])
                .collect::<Vec<u16>>();
            thread::spawn(move || {
                let title = CString::new("Failed to load Giuroll!").unwrap();
                println!("loader thread sppawn");
                let giuroll = LoadLibraryW(giuroll_path.as_ptr());
                if giuroll == null_mut() {
                    MessageBoxA(
                        std::ptr::null_mut(),
                        title.as_ptr(),
                        CString::new(format!(
                            "Failed to load giuroll.dll! Error code {}",
                            GetLastError()
                        ))
                        .unwrap()
                        .as_ptr(),
                        MB_ICONERROR,
                    );
                    return;
                }
                let initialize = match GetProcAddress(
                    giuroll,
                    CString::new("InitializeByLoader").unwrap().as_ptr() as *const i8,
                ) as usize
                {
                    0 => GetProcAddress(giuroll, CString::new("Initialize").unwrap().as_ptr()),
                    x => x as FARPROC,
                };
                if initialize == std::ptr::null_mut() {
                    MessageBoxA(
                        std::ptr::null_mut(),
                        title.as_ptr(),
                        CString::new(format!(
                            "Failed to get InitializeByLoader or Initialize function of Giuroll. Error code {}",
                            GetLastError()
                        ))
                        .unwrap()
                        .as_ptr(),
                        MB_ICONERROR,
                    );
                    return;
                }
                let initialize =
                    std::mem::transmute::<_, extern "C" fn(HINSTANCE) -> bool>(initialize);
                if !initialize(giuroll) {
                    MessageBoxA(
                        std::ptr::null_mut(),
                        title.as_ptr(),
                        CString::new("Giuroll failed to initialize")
                            .unwrap()
                            .as_ptr(),
                        MB_ICONERROR,
                    );
                }
            });
        }
        _ => {}
    }
    1
}

unsafe fn load_by_swrstoys() {
    MessageBoxA(
        std::ptr::null_mut(),
        CString::new("Error").unwrap().as_ptr(),
        CString::new(
            "giuroll_loader_dll.dll should not be loaded by SWRSToys. Load giuroll.dll instead.",
        )
        .unwrap()
        .as_ptr(),
        MB_ICONERROR,
    );
    panic!("giuroll_loader_dll.dll should not be loaded by SWRSToys");
}

#[no_mangle]
pub unsafe extern "C" fn Initialize(_module: HINSTANCE) -> bool {
    load_by_swrstoys();
    false
}

#[no_mangle]
pub unsafe extern "cdecl" fn CheckVersion(_a: *const [u8; 16]) -> bool {
    load_by_swrstoys();
    false
}

#[no_mangle]
pub unsafe extern "C" fn getPriority() -> i32 {
    load_by_swrstoys();
    1000
}
