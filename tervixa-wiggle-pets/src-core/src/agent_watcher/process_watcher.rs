//! OS process-table inspection (deterministic fallback telemetry).
//! Windows: NtQuerySystemInformation-free approach — spawn `tasklist` is slow;
//! instead we use the Windows API via winapi-less direct FFI on kernel32 for
//! process snapshotting. On Unix: /proc scan.

use std::collections::HashSet;

#[cfg(windows)]
pub fn find_processes(names: &[&str]) -> Vec<u32> {
    use std::os::raw::c_void;

    #[repr(C)]
    struct PROCESSENTRY32W {
        dw_size: u32,
        cnt_usage: u32,
        th32_process_id: u32,
        th32_default_heap: usize,
        th32_module_id: u32,
        cnt_threads: u32,
        th32_parent_process_id: u32,
        pc_pri_class_base: i32,
        dw_flags: u32,
        sz_exe_file: [u16; 260],
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateToolhelp32Snapshot(dw_flags: u32, th32_process_id: u32) -> *mut c_void;
        fn Process32FirstW(h: *mut c_void, entry: *mut PROCESSENTRY32W) -> i32;
        fn Process32NextW(h: *mut c_void, entry: *mut PROCESSENTRY32W) -> i32;
        fn CloseHandle(h: *mut c_void) -> i32;
    }

    const TH32CS_SNAPPROCESS: u32 = 0x2;
    const INVALID_HANDLE_VALUE: *mut c_void = -1isize as *mut c_void;

    let wanted: HashSet<String> = names.iter().map(|n| n.to_lowercase()).collect();
    let mut out = Vec::new();
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE || snap.is_null() {
            return out;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dw_size = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snap, &mut entry) != 0 {
            loop {
                let len = entry.sz_exe_file.iter().position(|c| *c == 0).unwrap_or(260);
                let exe = String::from_utf16_lossy(&entry.sz_exe_file[..len]).to_lowercase();
                if wanted.contains(exe.as_str()) {
                    out.push(entry.th32_process_id);
                }
                if Process32NextW(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
    }
    out
}

#[cfg(unix)]
pub fn find_processes(names: &[&str]) -> Vec<u32> {
    let wanted: HashSet<String> = names.iter().map(|n| n.to_lowercase()).collect();
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for e in entries.flatten() {
            let name = e.file_name();
            let Some(name) = name.to_str() else { continue };
            if !name.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            if let Ok(comm) = std::fs::read_to_string(format!("/proc/{name}/comm")) {
                let comm = comm.trim().to_lowercase();
                if wanted.contains(comm.as_str()) {
                    if let Ok(pid) = name.parse::<u32>() {
                        out.push(pid);
                    }
                }
            }
        }
    }
    out
}

/// Interrupt / stop a runaway agent: graceful ctrl-break on Windows, SIGINT on Unix.
pub fn interrupt_pid(pid: u32) -> Result<(), String> {
    interrupt_pid_inner(pid)
}

#[cfg(windows)]
fn interrupt_pid_inner(pid: u32) -> Result<(), String> {
    // Send CTRL_BREAK_EVENT to the process group via FreeConsole/AttachConsole.
    #[repr(C)]
    struct Corners([usize; 4]);
    #[link(name = "kernel32")]
    extern "system" {
        fn FreeConsole() -> i32;
        fn AttachConsole(dw_pid: u32) -> i32;
        fn SetConsoleCtrlHandler(handler: usize, add: i32) -> i32;
        fn GenerateConsoleCtrlEvent(dw_event: u32, dw_pid: u32) -> i32;
    }
    unsafe {
        SetConsoleCtrlHandler(0, 1); // ignore our own ctrl event
        FreeConsole();
        if AttachConsole(pid) == 0 {
            // Not attached (already exited / different console) — hard stop.
            return kill_pid(pid);
        }
        let ok = GenerateConsoleCtrlEvent(1, pid); // CTRL_BREAK_EVENT
        FreeConsole();
        if ok == 0 {
            return kill_pid(pid);
        }
    }
    Ok(())
}

#[cfg(windows)]
fn kill_pid(pid: u32) -> Result<(), String> {
    let status = std::process::Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("taskkill failed".into())
    }
}

#[cfg(unix)]
fn interrupt_pid_inner(pid: u32) -> Result<(), String> {
    unsafe {
        if libc_kill(pid as i32, 2) != 0 {
            return Err("SIGINT failed".into());
        }
    }
    Ok(())
}

#[cfg(unix)]
extern "C" {
    #[link_name = "kill"]
    fn libc_kill(pid: i32, sig: i32) -> i32;
}
