use windows::core::HRESULT;
use windows::Win32::Foundation::ERROR_CANCELLED;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
use windows::Win32::UI::Shell::{
    FileOpenDialog, IFileOpenDialog, FOS_ALLOWMULTISELECT, FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM,
    FOS_PATHMUSTEXIST, SIGDN_FILESYSPATH,
};

use crate::error::LockdownError;

pub fn open_file_dialog() -> Result<Vec<String>, LockdownError> {
    // let init_result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) };
    unsafe {
        // 创建文件打开对话框实例
        let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_ALL)?;

        // 设置对话框选项：允许多选，必须存在文件等
        dialog.SetOptions(
            FOS_ALLOWMULTISELECT | FOS_FORCEFILESYSTEM | FOS_FILEMUSTEXIST | FOS_PATHMUSTEXIST,
        )?;

        // 显示对话框并处理用户取消
        match dialog.Show(None) {
            Ok(_) => {}
            Err(e) if e.code() == HRESULT::from_win32(ERROR_CANCELLED.0) => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        }

        // 获取选中的文件项集合
        let items = dialog.GetResults()?;
        let count = items.GetCount()?;
        let mut files = Vec::with_capacity(count as usize);

        for i in 0..count {
            let item = items.GetItemAt(i)?;

            // 获取文件路径
            let path = item.GetDisplayName(SIGDN_FILESYSPATH)?;
            let path_str = path.to_string()?;
            files.push(path_str);
        }

        Ok(files)
    }
}
