use anyhow::{Context, Result};
use std::fs;
use std::ptr::addr_of;

use windows::Win32::UI::Shell::{FOLDERID_ProgramData, SHGetKnownFolderPath};

#[inline]
fn read_data_file() -> Result<Vec<String>> {
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

fn main() -> Result<()> {
    let file_paths = read_data_file()?;
    let mut handles = Vec::new();
    for file_path in &file_paths {
        let handle = unsafe {
            match windows::Win32::Storage::FileSystem::CreateFileW(
                &windows::core::HSTRING::from(file_path),
                (windows::Win32::Foundation::GENERIC_READ
                    | windows::Win32::Foundation::GENERIC_WRITE)
                    .0,
                windows::Win32::Storage::FileSystem::FILE_SHARE_READ,
                None,
                windows::Win32::Storage::FileSystem::OPEN_EXISTING,
                windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
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
                        return Err(err)
                            .with_context(|| format!("Error Opening File: {}", file_path));
                    }
                }
            }
        };

        handles.push(handle);
    }

    loop {
        std::thread::park();
    }
}
