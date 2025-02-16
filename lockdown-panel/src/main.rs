#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use iced::{Font, Task};

mod app;
mod components;
mod error;
mod macros;
mod service;
mod store;
mod utils;

fn main() -> iced::Result {
    std::panic::set_hook(Box::new(|info| {
        let panic_message = format!(
            "Panic occurred: {}\nLocation: {:?}",
            info.payload().downcast_ref::<&str>().unwrap_or(&"Unknown"),
            info.location()
        );

        // 将 panic 信息转换为 Unicode 字符串
        let message = windows::core::HSTRING::from(panic_message);
        let title = windows::core::HSTRING::from("Panic Handler");

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
    }));

    iced::application(
        app::LockdownPanel::title,
        app::LockdownPanel::update,
        app::LockdownPanel::view,
    )
    .centered()
    // .font(include_bytes!("../assets/fonts/NotoSansSC-VF_wght.ttf").as_slice())
    .font(include_bytes!("../assets/fonts/NotoSansSC-Regular.ttf").as_slice())
    .default_font(Font::with_name("Noto Sans SC"))
    .run_with(|| {
        (
            app::LockdownPanel::default(),
            Task::future(async { app::Message::PollServiceStatus }),
        )
    })
}
