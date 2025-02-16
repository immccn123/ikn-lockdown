use std::{
    ffi::OsString,
    fs,
    ptr::addr_of,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Context;
use windows::{
    core::HSTRING,
    Win32::{
        self,
        UI::Shell::{FOLDERID_ProgramData, SHGetKnownFolderPath},
    },
};
use windows_service::{
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult, ServiceStatusHandle},
};

#[inline]
fn read_data_file() -> anyhow::Result<Vec<String>> {
    let folder_id = FOLDERID_ProgramData;
    let program_data = unsafe {
        SHGetKnownFolderPath(
            addr_of!(folder_id),
            windows::Win32::UI::Shell::KNOWN_FOLDER_FLAG(0),
            None,
        )
        .with_context(|| format!("Error Getting ProgramData Path"))?
        .to_string()?
    };

    let data_file = std::path::Path::new(&program_data)
        .join("Lockdown.Service")
        .join("locked_files.txt");

    Ok(fs::read_to_string(data_file)
        .with_context(|| "Error Reading Configuration File")?
        .split_ascii_whitespace()
        .filter_map(|s| {
            let trimed = s.trim();
            if s.len() == 0 {
                None
            } else {
                Some(trimed.to_string())
            }
        })
        .collect())
}

pub fn run(_arguments: &Vec<OsString>) -> Result<(), anyhow::Error> {
    let status_handle: Arc<Mutex<Option<ServiceStatusHandle>>> = Arc::new(Mutex::new(None));
    let status_handle_inner = status_handle.clone();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            ServiceControl::Stop => {
                let status_handle_inner = status_handle_inner.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(800));
                    if let Ok(Some(h)) = status_handle_inner.lock().as_deref() {
                        let _ = h.set_service_status(ServiceStatus {
                            service_type: ServiceType::OWN_PROCESS,
                            current_state: ServiceState::Stopped,
                            controls_accepted: ServiceControlAccept::empty(),
                            exit_code: ServiceExitCode::Win32(0),
                            checkpoint: 0,
                            wait_hint: Duration::default(),
                            process_id: None,
                        });
                    }
                });

                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    status_handle
        .lock()
        .unwrap()
        .replace(service_control_handler::register(
            "lockdown-service",
            event_handler,
        )?);

    let file_paths = read_data_file()?;
    let mut handles = Vec::new();
    for file_path in &file_paths {
        let handle = unsafe {
            match Win32::Storage::FileSystem::CreateFileW(
                &HSTRING::from(file_path),
                (Win32::Foundation::GENERIC_READ | Win32::Foundation::GENERIC_WRITE).0,
                Win32::Storage::FileSystem::FILE_SHARE_READ,
                None,
                Win32::Storage::FileSystem::OPEN_EXISTING,
                Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
                None,
            ) {
                Ok(x) => {
                    println!("File Locked: {}", file_path);
                    Some(x)
                }
                Err(err) => {
                    if err.code() == windows::core::HRESULT(0x80070002u32 as i32) {
                        eprintln!("File not found: {file_path}");
                        None
                    } else {
                        continue;
                    }
                }
            }
        };

        handles.push(handle);
    }

    let next_status = ServiceStatus {
        // Should match the one from system service registry
        service_type: ServiceType::OWN_PROCESS,
        // The new state
        current_state: ServiceState::Running,
        // Accept stop events when running
        controls_accepted: ServiceControlAccept::STOP,
        // Used to report an error when starting or stopping only, otherwise must be zero
        exit_code: ServiceExitCode::Win32(0),
        // Only used for pending states, otherwise must be zero
        checkpoint: 0,
        // Only used for pending states, otherwise must be zero
        wait_hint: Duration::default(),
        process_id: None,
    };

    status_handle
        .lock()
        .unwrap()
        .unwrap()
        .set_service_status(next_status)?;

    loop {
        std::thread::park();
    }
}
