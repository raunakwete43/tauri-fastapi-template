// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
};
use tauri::RunEvent;

// Load FastAPI configuration from JSON file
fn load_fastapi_config() -> (String, Vec<String>) {
    let config_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fastapi-config.json");
    let config_str = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("[tauri] Failed to read FastAPI config: {}", e);
            return (String::new(), Vec::new());
        }
    };

    let config: Value = match serde_json::from_str(&config_str) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("[tauri] Failed to parse FastAPI config: {}", e);
            return (String::new(), Vec::new());
        }
    };

    let os_key = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };

    let os_config = &config[os_key];

    let cmd = os_config["command"].as_str().unwrap_or("").to_string();

    let args = os_config["args"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|arg| arg.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_else(Vec::new);

    (cmd, args)
}

fn spawn_fastapi() -> Option<Child> {
    let (cmd, args) = load_fastapi_config();
    if cmd.is_empty() {
        eprintln!("[tauri] FastAPI command is empty. Check configuration.");
        return None;
    }

    println!("[tauri] Starting FastAPI server: {} {:?}", cmd, args);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::JobObjects::*;

        let mut command = Command::new(&cmd);
        command
            .args(&args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .creation_flags(0);

        match command.spawn() {
            Ok(child) => {
                // Assign process to a job object so it's killed if parent dies.
                unsafe {
                    let job = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
                    if !job.is_null() {
                        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                        info.BasicLimitInformation.LimitFlags = 0x00002000; // JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
                        SetInformationJobObject(
                            job,
                            JobObjectExtendedLimitInformation,
                            &mut info as *mut _ as *mut _,
                            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                        );
                        use std::os::windows::io::AsRawHandle;
                        let _ = AssignProcessToJobObject(job, child.as_raw_handle() as _);
                    }
                }
                println!("[tauri] FastAPI process started.");
                Some(child)
            }
            Err(e) => {
                eprintln!("[tauri] Failed to start FastAPI process: {}", e);
                None
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        match Command::new(cmd)
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(child) => {
                println!("[tauri] FastAPI process started.");
                Some(child)
            }
            Err(e) => {
                eprintln!("[tauri] Failed to start FastAPI process: {}", e);
                None
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let fastapi_process = Arc::new(Mutex::new(None));
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![])
        .setup({
            let fastapi_process = fastapi_process.clone();
            move |_app| {
                *fastapi_process.lock().unwrap() = spawn_fastapi();
                Ok(())
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(move |_app_handle, event| {
            if let RunEvent::Exit = event {
                if let Some(mut child) = fastapi_process.lock().unwrap().take() {
                    let _ = child.kill();
                    println!("[tauri] FastAPI process killed.");
                }
            }
        });
}