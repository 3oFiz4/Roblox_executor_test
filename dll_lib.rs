use libloading::{Library, Symbol};
use std::ffi::{c_char, CString};
use std::sync::OnceLock;

static DLL: OnceLock<Result<Library, String>> = OnceLock::new();

fn get_or_load_dll() -> Result<&'static Library, &'static str> {
    let res = DLL.get_or_init(|| {
        unsafe {
            let mut candidates: Vec<std::path::PathBuf> = Vec::new();

            if let Ok(env_path) = std::env::var("RBX_DLL_PATH") {
                candidates.push(std::path::PathBuf::from(env_path));
            }

            if let Ok(exe) = std::env::current_exe() {
                if let Some(exe_dir) = exe.parent() {
                    candidates.push(exe_dir.join("rbx.dll"));
                    candidates.push(exe_dir.join("rbx-1.dll"));

                    let mut curr = exe_dir.to_path_buf();
                    for _ in 0..5 {
                        if let Some(parent) = curr.parent() {
                            candidates.push(parent.join("rbx.dll"));
                            candidates.push(parent.join("rbx-1.dll"));
                            candidates.push(parent.join("src").join("rbx.dll"));
                            candidates.push(parent.join("src").join("rbx-1.dll"));
                            curr = parent.to_path_buf();
                        } else {
                            break;
                        }
                    }
                }
            }

            candidates.push(std::path::PathBuf::from("rbx.dll"));
            candidates.push(std::path::PathBuf::from("rbx-1.dll"));
            candidates.push(std::path::PathBuf::from("src/rbx.dll"));
            candidates.push(std::path::PathBuf::from("src/rbx-1.dll"));

            let mut last_err = String::from("No candidate DLL found");
            for path in &candidates {
                if path.exists() {
                    let load_target = if let Ok(canon) = path.canonicalize() {
                        let s = canon.to_string_lossy().to_string();
                        if s.starts_with(r"\\?\") {
                            std::path::PathBuf::from(&s[4..])
                        } else {
                            canon
                        }
                    } else {
                        path.clone()
                    };

                    match Library::new(&load_target) {
                        Ok(lib) => {
                            if let Ok(init) = lib.get::<extern "C" fn()>(b"initialize") {
                                init();
                            }
                            return Ok(lib);
                        }
                        Err(e) => {
                            last_err = format!("Failed to load {:?}: {}", path, e);
                        }
                    }
                }
            }

            Err(format!("API Load fail: {}", last_err))
        }
    });

    match res {
        Ok(lib) => Ok(lib),
        Err(e) => Err(e.as_str()),
    }
}

pub fn attach() -> Result<(), String> {
    let lib = get_or_load_dll().map_err(|e| e.to_string())?;
    unsafe {
        let func: Symbol<extern "C" fn()> = lib.get(b"attach").map_err(|e| e.to_string())?;
        func();
    }
    Ok(())
}

pub fn is_attached() -> bool {
    let Ok(lib) = get_or_load_dll() else {
        return false;
    };
    unsafe {
        match lib.get::<extern "C" fn() -> u8>(b"isAttached") {
            Ok(func) => func() == 1,
            Err(_) => false,
        }
    }
}

pub fn execute(script: &str) -> Result<(), String> {
    let lib = get_or_load_dll().map_err(|e| e.to_string())?;
    let c_string = CString::new(script).map_err(|e| e.to_string())?;
    unsafe {
        let func: Symbol<extern "C" fn(*const c_char)> = lib.get(b"execute").map_err(|e| e.to_string())?;
        func(c_string.as_ptr());
    }
    Ok(())
}

#[cfg(windows)]
#[repr(C)]
struct PROCESSENTRY32W {
    dw_size: u32,
    cnt_usage: u32,
    th32_process_id: u32,
    th32_default_heap_id: usize,
    th32_module_id: u32,
    cnt_threads: u32,
    th32_parent_process_id: u32,
    pc_pri_class_base: i32,
    dw_flags: u32,
    sz_exe_file: [u16; 260],
}

#[cfg(windows)]
unsafe extern "system" {
    fn CreateToolhelp32Snapshot(dw_flags: u32, th32_process_id: u32) -> *mut std::ffi::c_void;
    fn Process32FirstW(h_snapshot: *mut std::ffi::c_void, lppe: *mut PROCESSENTRY32W) -> i32;
    fn Process32NextW(h_snapshot: *mut std::ffi::c_void, lppe: *mut PROCESSENTRY32W) -> i32;
    fn CloseHandle(h_object: *mut std::ffi::c_void) -> i32;
}

pub fn is_process_running(proc_name: &str) -> bool {
    #[cfg(windows)]
    unsafe {
        const TH32CS_SNAPPROCESS: u32 = 0x00000002;
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot.is_null() || snapshot == -1isize as *mut std::ffi::c_void {
            return false;
        }

        let mut entry = std::mem::zeroed::<PROCESSENTRY32W>();
        entry.dw_size = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut found = false;
        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let len = entry
                    .sz_exe_file
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.sz_exe_file.len());
                let name = String::from_utf16_lossy(&entry.sz_exe_file[..len]);
                if name.eq_ignore_ascii_case(proc_name) {
                    found = true;
                    break;
                }
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
        found
    }
    #[cfg(not(windows))]
    {
        false
    }
}

pub fn is_roblox_running() -> bool {
    is_process_running("RobloxPlayerBeta.exe") || is_process_running("Windows10Universal.exe")
}
