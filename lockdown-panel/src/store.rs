use std::ptr::addr_of;

use windows::Win32::UI::Shell::{FOLDERID_ProgramData, SHGetKnownFolderPath};

use crate::error::LockdownError;

pub fn read_data_file() -> Result<Vec<String>, LockdownError> {
    let folder_id = FOLDERID_ProgramData;
    let program_data = unsafe {
        SHGetKnownFolderPath(
            addr_of!(folder_id),
            windows::Win32::UI::Shell::KNOWN_FOLDER_FLAG(0),
            None,
        )?
        .to_string()?
    };

    let data_file = std::path::Path::new(&program_data)
        .join("Lockdown.Service")
        .join("locked_files.txt");

    if !std::fs::exists(&data_file)? {
        std::fs::create_dir_all(crate::service::get_program_folder())?;
        write_data_file(&Vec::new())?;
    }

    Ok(std::fs::read_to_string(data_file)?
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

pub fn write_data_file(data: &Vec<String>) -> Result<(), LockdownError> {
    let folder_id = FOLDERID_ProgramData;
    let program_data = unsafe {
        SHGetKnownFolderPath(
            addr_of!(folder_id),
            windows::Win32::UI::Shell::KNOWN_FOLDER_FLAG(0),
            None,
        )?
        .to_string()?
    };

    let data_file = std::path::Path::new(&program_data)
        .join("Lockdown.Service")
        .join("locked_files.txt");

    std::fs::write(data_file, data.join("\n"))?;

    Ok(())
}

