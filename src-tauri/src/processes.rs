use std::{path::Path, sync::Mutex};

use serde::Serialize;
use sysinfo::{Pid, Process, ProcessesToUpdate, System};
use tauri::State;

use crate::suspicious;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcInfo {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub exe: Option<String>,
    pub cmd: String,
    /// % trên toàn bộ CPU (đã chia số lõi), giống Task Manager.
    pub cpu: f32,
    pub memory: u64,
    pub flags: Vec<&'static str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    cpu: f32,
    mem_used: u64,
    mem_total: u64,
    processes: Vec<ProcInfo>,
}

pub fn proc_info(p: &Process, sys: &System) -> ProcInfo {
    let cores = sys.cpus().len().max(1) as f32;
    let windir = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    let name = p.name().to_string_lossy().into_owned();
    let exe = p.exe().map(|e| e.to_string_lossy().into_owned());
    let missing = exe.as_deref().is_some_and(|e| !Path::new(e).exists());
    ProcInfo {
        pid: p.pid().as_u32(),
        parent_pid: p.parent().map(|x| x.as_u32()),
        flags: suspicious::flags(&name, exe.as_deref(), missing, &windir),
        cmd: p
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" "),
        cpu: p.cpu_usage() / cores,
        memory: p.memory(),
        name,
        exe,
    }
}

#[tauri::command(async)]
pub fn snapshot(sys: State<Mutex<System>>) -> Snapshot {
    let mut sys = sys.lock().unwrap();
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let processes = sys
        .processes()
        .values()
        .map(|p| proc_info(p, &sys))
        .collect();
    Snapshot {
        cpu: sys.global_cpu_usage(),
        mem_used: sys.used_memory(),
        mem_total: sys.total_memory(),
        processes,
    }
}

#[tauri::command(async)]
pub fn kill_process(pid: u32, sys: State<Mutex<System>>) -> Result<(), String> {
    let sys = sys.lock().unwrap();
    let p = sys
        .process(Pid::from_u32(pid))
        .ok_or("Không tìm thấy tiến trình")?;
    if pid <= 4 || pid == std::process::id() || suspicious::is_critical(&p.name().to_string_lossy())
    {
        return Err("Đây là tiến trình hệ thống quan trọng, không được dừng".into());
    }
    if p.kill() {
        Ok(())
    } else {
        Err("Không dừng được (có thể cần chạy app bằng quyền Administrator)".into())
    }
}
