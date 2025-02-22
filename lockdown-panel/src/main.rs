#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use iced::{
    window::{self, icon},
    Font, Task,
};

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

        let message = windows::core::HSTRING::from(panic_message);
        let title = windows::core::HSTRING::from("Panicked");

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

    let mut window_settings = window::Settings::default();
    let icon_file = ico::IconDir::read(std::io::Cursor::new(include_bytes!(
        "../assets/lockdown.ico"
    )));
    window_settings.size = iced::Size::new(500., 500.);

    if let Ok(icon_file) = icon_file {
        let icon = icon_file.entries().first().unwrap().decode().unwrap();
        let icon = icon::from_rgba(icon.rgba_data().to_vec(), icon.width(), icon.height()).unwrap();
        window_settings.icon = Some(icon);
    } else {
        eprintln!("Could not load icon");
    }

    iced::application(
        app::LockdownPanel::title,
        app::LockdownPanel::update,
        app::LockdownPanel::view,
    )
    .centered()
    .font(include_bytes!("../assets/fonts/NotoSansSC-Regular.ttf").as_slice())
    .default_font(Font::with_name("Noto Sans SC"))
    .window(window_settings)
    .run_with(|| {
        (
            app::LockdownPanel::default(),
            Task::future(async { app::Message::PollServiceStatus }),
        )
    })
}
