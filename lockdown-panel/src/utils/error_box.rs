use crate::error::LockdownError;

pub fn error_box(error: &LockdownError) {
    let message = format!("Error: {}\nInfo:\n{:?}", error, error);

    let message = windows::core::HSTRING::from(message);
    let title = windows::core::HSTRING::from("Error Occurred");

    // 使用 MessageBoxW 弹出对话框
    unsafe {
        use windows::core::*;
        use windows::Win32::Foundation::*;
        use windows::Win32::UI::WindowsAndMessaging::*;

        MessageBoxW(
            HWND(std::ptr::null_mut()),
            PCWSTR(message.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK,
        );
    }
}
