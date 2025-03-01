use iced::widget::{button, checkbox, column, row, scrollable, text};
use iced::{Color, Element, Length, Padding, Task};
use std::collections::HashSet;
use std::time::Duration;
use windows::core::HSTRING;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use crate::components::service_banner;
use crate::error::UnwrapOrReport;
use crate::service::{start_service_flow, ServiceStatus};
use crate::utils::open_file_dialog;

#[derive(Debug, Clone)]
pub enum Message {
    ToggleFileSelection(usize),
    ShowFilebox,
    AddFiles(Vec<String>),
    RemoveSelectedFiles,
    RestartService,
    UpdateServiceStatus(ServiceStatus),
    PollServiceStatus,
    Error(String),
    About,

    #[allow(unused)]
    Nothing,
}

#[derive(Debug, Clone)]
pub struct LockdownPanel {
    files: Vec<String>,
    /// 被选中的文件
    selected_files: HashSet<usize>,
    service_status: ServiceStatus,
    error_message: Vec<String>,
    restart_required: bool,
}

impl LockdownPanel {
    pub fn view(&self) -> Element<Message> {
        let mut elements: Vec<Element<'_, Message>> = Vec::new();
        let file_list = self
            .files
            .iter()
            .enumerate()
            .fold(column![], |col, (index, file)| {
                col.push(
                    row![checkbox(file, self.selected_files.contains(&index))
                        .on_toggle(move |_| Message::ToggleFileSelection(index))]
                    .spacing(12),
                )
            });

        if self.service_status != ServiceStatus::Running || self.restart_required {
            elements.push(service_banner(self.service_status, self.restart_required));
        };

        if !self.error_message.is_empty() {
            elements.push(
                self.error_message
                    .iter()
                    .fold(column![], |col, message| {
                        col.push(text(message).color(Color::from_rgb(1., 0., 0.)))
                    })
                    .into(),
            )
        };

        let restart_button_text = match self.service_status {
            ServiceStatus::Running | ServiceStatus::Error => "重启服务",
            ServiceStatus::Starting => "启动中...",
            ServiceStatus::Stopped => "启动服务",
        };

        let menu = row![
            button("添加").on_press(Message::ShowFilebox),
            button("移除")
                .style(button::secondary)
                .on_press(Message::RemoveSelectedFiles),
            button(restart_button_text)
                .style(button::secondary)
                .on_press_maybe(if self.service_status == ServiceStatus::Starting {
                    None
                } else {
                    Some(Message::RestartService)
                }),
            button("关于")
                .style(button::secondary)
                .on_press(Message::About),
        ]
        .spacing(12);

        column![text("Lockdown 面板").size(20)]
            .extend(elements)
            .push(menu)
            .push(scrollable(file_list).width(Length::Fill))
            .spacing(12)
            .padding(Padding::from([6, 20]))
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleFileSelection(index) => {
                if self.selected_files.contains(&index) {
                    self.selected_files.remove(&index);
                } else {
                    self.selected_files.insert(index);
                }

                Task::none()
            }

            Message::Error(message) => {
                self.error_message.push(message);

                Task::none()
            }

            Message::ShowFilebox => Task::future(async {
                Message::AddFiles(open_file_dialog().await.unwrap_or_report())
            }),

            Message::RemoveSelectedFiles => {
                let mut keep =
                    (0..self.files.len()).map(|index| !self.selected_files.contains(&index));
                self.files.retain(|_| keep.next().unwrap());
                self.selected_files.clear();
                if let Err(e) = crate::store::write_data_file(&self.files) {
                    Task::future(async move { Message::Error(e.to_string()) })
                } else {
                    self.restart_required = true;
                    Task::none()
                }
            }

            Message::RestartService => {
                self.service_status = ServiceStatus::Starting;
                self.restart_required = false;

                Task::future(async {
                    if let Err(e) = start_service_flow(true) {
                        eprintln!("Error starting service: {}", e);
                        Message::UpdateServiceStatus(ServiceStatus::Error)
                    } else {
                        Message::UpdateServiceStatus(ServiceStatus::Running)
                    }
                })
            }

            Message::UpdateServiceStatus(status) => {
                self.service_status = status;

                Task::none()
            }

            Message::PollServiceStatus => {
                if self.service_status != ServiceStatus::Starting {
                    match crate::service::is_service_running() {
                        Ok(running) => {
                            if running {
                                self.service_status = ServiceStatus::Running;
                            } else {
                                if self.service_status != ServiceStatus::Error {
                                    self.service_status = ServiceStatus::Stopped;
                                }
                            }
                        }
                        Err(_) => self.service_status = ServiceStatus::Error,
                    }
                }

                Task::future(async {
                    std::thread::sleep(Duration::from_secs(1));

                    Message::PollServiceStatus
                })
            }

            Message::AddFiles(mut new_files) => {
                new_files.sort();
                new_files.dedup();
                for path in new_files {
                    if !self.files.contains(&path) {
                        self.files.push(path);
                    }
                }
                if let Err(e) = crate::store::write_data_file(&self.files) {
                    Task::future(async move { Message::Error(e.to_string()) })
                } else {
                    self.restart_required = true;
                    Task::none()
                }
            }

            Message::About => Task::future(async {
                unsafe {
                    ShellExecuteW(
                        None,
                        &HSTRING::from("open"),
                        &HSTRING::from("https://lockdown.imken.dev/about.html"),
                        None,
                        None,
                        SW_SHOWNORMAL,
                    );

                    Message::Nothing
                }
            }),

            Message::Nothing => Task::none(),
        }
    }

    pub fn title(&self) -> String {
        "Lockdown Panel".into()
    }
}

impl Default for LockdownPanel {
    fn default() -> Self {
        let service_status = if crate::service::is_service_running().unwrap() {
            ServiceStatus::Running
        } else {
            ServiceStatus::Stopped
        };

        Self {
            files: crate::store::read_data_file().unwrap(),
            selected_files: HashSet::new(),
            service_status,
            error_message: Vec::new(),
            restart_required: false,
        }
    }
}
