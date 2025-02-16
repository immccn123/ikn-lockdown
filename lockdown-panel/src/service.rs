use std::path::Path;
use std::ptr::{addr_of, addr_of_mut};
use std::time::Duration;
use windows::core::{HSTRING, PWSTR};
use windows::Win32::System::Services::{
    ChangeServiceConfig2W, CloseServiceHandle, ControlService, CreateServiceW, DeleteService,
    OpenSCManagerW, OpenServiceW, QueryServiceStatus, StartServiceW, SC_ACTION, SC_ACTION_RESTART,
    SC_HANDLE, SC_MANAGER_ALL_ACCESS, SERVICE_AUTO_START, SERVICE_CONFIG_FAILURE_ACTIONS,
    SERVICE_CONTROL_STOP, SERVICE_ERROR_NORMAL, SERVICE_FAILURE_ACTIONSW, SERVICE_QUERY_STATUS,
    SERVICE_RUNNING, SERVICE_START, SERVICE_STATUS, SERVICE_STOP, SERVICE_WIN32_OWN_PROCESS,
};

use crate::error::LockdownError;

const SERVICE_NAME: &str = "lockdown-service";

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ServiceStatus {
    Running,
    Error,
    Stopped,
    Starting,
}

#[inline]
fn open_sc_manager() -> Result<SC_HANDLE, windows::core::Error> {
    unsafe { OpenSCManagerW(None, None, SC_MANAGER_ALL_ACCESS) }
}


pub fn get_service_executable_path() -> String {
    let common_app_data = std::env::var("ProgramData").unwrap();
    Path::new(&common_app_data)
        .join("Lockdown.Service")
        .join("lockdown-service.exe")
        .to_string_lossy()
        .into_owned()
}

pub fn get_program_folder() -> String {
    let common_app_data = std::env::var("ProgramData").unwrap();
    Path::new(&common_app_data)
        .join("Lockdown.Service")
        .to_string_lossy()
        .into_owned()
}

pub fn copy_service_exe() -> std::io::Result<()> {
    let source_path = Path::new("Assets").join("lockdown-service.exe");
    let target_path = get_service_executable_path();
    std::fs::create_dir_all(get_program_folder())?;
    std::fs::copy(source_path, target_path)?;
    Ok(())
}

pub fn service_exists() -> Result<bool, LockdownError> {
    let sc_manager = open_sc_manager()?;
    let service = unsafe {
        OpenServiceW(
            sc_manager,
            &HSTRING::from(SERVICE_NAME),
            SERVICE_QUERY_STATUS,
        )
    };
    if service.is_err() {
        unsafe { CloseServiceHandle(sc_manager)? };
        return Ok(false);
    }
    let service = service.unwrap();
    unsafe {
        CloseServiceHandle(service)?;
        CloseServiceHandle(sc_manager)?;
    };

    Ok(true)
}

pub fn check_service_location() -> bool {
    Path::new(&get_service_executable_path()).exists()
}

pub fn register_service() -> Result<(), LockdownError> {
    if !check_service_location() {
        copy_service_exe()?;
    }

    if service_exists()? {
        unregister_service()?;
    }

    let sc_manager = open_sc_manager()?;
    let service = unsafe {
        CreateServiceW(
            sc_manager,
            &HSTRING::from(SERVICE_NAME),
            &HSTRING::from(SERVICE_NAME),
            SC_MANAGER_ALL_ACCESS,
            SERVICE_WIN32_OWN_PROCESS,
            SERVICE_AUTO_START,
            SERVICE_ERROR_NORMAL,
            &HSTRING::from(get_service_executable_path()),
            None,
            None,
            None,
            None,
            None,
        )?
    };

    let mut sc_action = SC_ACTION {
        Type: SC_ACTION_RESTART,
        ..Default::default()
    };

    // Configure service failure actions
    let failure_actions = SERVICE_FAILURE_ACTIONSW {
        dwResetPeriod: 0,
        lpRebootMsg: PWSTR::null(),
        lpCommand: PWSTR::null(),
        cActions: 3,
        lpsaActions: addr_of_mut!(sc_action),
    };

    unsafe {
        ChangeServiceConfig2W(
            service,
            SERVICE_CONFIG_FAILURE_ACTIONS,
            Some(addr_of!(failure_actions) as *const _),
        )?
    }

    unsafe {
        CloseServiceHandle(service)?;
        CloseServiceHandle(sc_manager)?;
    };

    Ok(())
}

pub fn unregister_service() -> Result<(), LockdownError> {
    if service_exists()? {
        stop_service()?;
        let sc_manager = open_sc_manager()?;
        let service = unsafe {
            OpenServiceW(
                sc_manager,
                &HSTRING::from(SERVICE_NAME),
                SC_MANAGER_ALL_ACCESS,
            )?
        };
        unsafe {
            DeleteService(service)?;
            CloseServiceHandle(service)?;
            CloseServiceHandle(sc_manager)?;
        };
    }
    Ok(())
}

pub fn start_service() -> Result<(), LockdownError> {
    if !service_exists()? {
        return Ok(());
    }

    let sc_manager = open_sc_manager()?;
    let service = unsafe { OpenServiceW(sc_manager, &HSTRING::from(SERVICE_NAME), SERVICE_START)? };
    unsafe {
        StartServiceW(service, None)?;
        CloseServiceHandle(service)?;
        CloseServiceHandle(sc_manager)?;
    };

    Ok(())
}

pub fn stop_service() -> Result<(), LockdownError> {
    if !service_exists()? {
        return Ok(());
    }
    let sc_manager = open_sc_manager()?;
    let service = unsafe { OpenServiceW(sc_manager, &HSTRING::from(SERVICE_NAME), SERVICE_STOP)? };

    let mut service_status_output = SERVICE_STATUS::default();

    unsafe {
        ControlService(
            service,
            SERVICE_CONTROL_STOP,
            addr_of_mut!(service_status_output),
        )?;
        CloseServiceHandle(service)?;
        CloseServiceHandle(sc_manager)?;
    };

    Ok(())
}

pub fn is_service_running() -> Result<bool, LockdownError> {
    if !service_exists()? {
        return Ok(false);
    }

    let sc_manager = open_sc_manager()?;
    let service = unsafe {
        OpenServiceW(
            sc_manager,
            &HSTRING::from(SERVICE_NAME),
            SERVICE_QUERY_STATUS,
        )?
    };
    let mut status: SERVICE_STATUS = unsafe { std::mem::zeroed() };
    unsafe {
        QueryServiceStatus(service, &mut status)?;
        CloseServiceHandle(service)?;
        CloseServiceHandle(sc_manager)?;
    };

    Ok(status.dwCurrentState == SERVICE_RUNNING)
}

pub fn start_service_flow(should_stop_service: bool) -> Result<(), LockdownError> {
    if should_stop_service {
        if is_service_running()? {
            stop_service()?;
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    if !service_exists()? {
        register_service()?;
    }

    if !is_service_running()? {
        start_service()?;
    }

    std::thread::sleep(Duration::from_secs(1));

    Ok(())
}
