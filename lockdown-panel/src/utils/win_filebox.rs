use crate::error::LockdownError;

pub async fn open_file_dialog() -> Result<Vec<String>, LockdownError> {
    let dialog = rfd::AsyncFileDialog::new();
    let handles = dialog.pick_files().await;
    let handles = match handles {
        None => return Ok(Vec::new()),
        Some(x) => x,
    };

    Ok(handles
        .iter()
        .map(|handle| handle.path().to_string_lossy().into()).collect())
}
