extern crate winapi;

use std::ffi::CString;
use std::ptr::null_mut;
use winapi::shared::minwindef::DWORD;
use winapi::shared::winerror::WAIT_TIMEOUT;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::CloseHandle;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::memoryapi::VirtualAllocEx;
use winapi::um::processthreadsapi::{CreateRemoteThread, GetExitCodeThread, OpenProcess};
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winbase::INFINITE;
use winapi::um::winnt::{HANDLE, MEM_COMMIT, PAGE_READWRITE, PROCESS_ALL_ACCESS};

pub(crate) unsafe fn open_process(pid: DWORD) -> Result<HANDLE, String> {
    let process = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
    if process.is_null() {
        Err("Failed to open the target process.".to_string())
    } else {
        Ok(process)
    }
}

unsafe fn alloc_memory<T: Sized>(
    process: HANDLE,
    data: &[T],
) -> Result<*mut winapi::ctypes::c_void, String> {
    assert_ne!(std::mem::size_of::<T>(), 0);
    let size = data.len() * std::mem::size_of::<T>();
    let addr = VirtualAllocEx(process, null_mut(), size, MEM_COMMIT, PAGE_READWRITE);
    if addr.is_null() {
        return Err("Failed to allocate memory in the target process.".to_string());
    }

    if winapi::um::memoryapi::WriteProcessMemory(
        process,
        addr,
        data.as_ptr() as *const _,
        size,
        null_mut(),
    ) == 0
    {
        return Err(format!(
            "Failed to write into the target process memory. Error code {}",
            GetLastError()
        ));
    }
    Ok(addr)
}

pub(crate) unsafe fn inject_dll(process: HANDLE, dll_path: &str) -> Result<HANDLE, String> {
    // let dll_path_cstring = CString::new(dll_path.to_string()).expect("CString::new failed");
    let to_utf_16 = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let dll_path_wstr = to_utf_16(dll_path);

    unsafe {
        let addr = match alloc_memory(process, dll_path_wstr.as_slice()) {
            Ok(addr) => addr,
            Err(s) => return Err(s),
        };
        let kernel32 = CString::new("kernel32.dll").expect("CString::new failed");
        let loadlibraryw = CString::new("LoadLibraryW").expect("CString::new failed");

        let h_kernel32 = GetModuleHandleA(kernel32.as_ptr());
        if h_kernel32.is_null() {
            return Err("Failed to get the handle of kernel32.dll.".to_string());
        }

        let h_loadlibraryw =
            winapi::um::libloaderapi::GetProcAddress(h_kernel32, loadlibraryw.as_ptr());
        if h_loadlibraryw.is_null() {
            return Err("Failed to get the address of LoadLibraryA.".to_string());
        }

        let handle = CreateRemoteThread(
            process,
            null_mut(),
            0,
            Some(std::mem::transmute(h_loadlibraryw)),
            addr as *mut _,
            0,
            null_mut(),
        );
        if handle.is_null() {
            return Err("Failed to create a remote thread in the target process.".to_string());
        }
        while WaitForSingleObject(handle, 0) == WAIT_TIMEOUT {
            WaitForSingleObject(handle, INFINITE);
        }
        let mut exit_code: HANDLE = std::ptr::null_mut();
        let ret = if GetExitCodeThread(handle, ((&mut exit_code) as *mut _) as *mut _) == 0 {
            Err("GetExitCodeThread returns false".to_string())
        } else {
            if exit_code == std::ptr::null_mut() {
                Err("Failed to load dll".to_string())
            } else {
                Ok(exit_code)
            }
        };
        CloseHandle(handle);
        ret
    }
}
