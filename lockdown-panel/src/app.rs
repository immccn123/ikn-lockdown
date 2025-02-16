use iced::widget::{button, checkbox, column, row, scrollable, text};
use iced::{Element, Padding, Task};
use std::collections::HashSet;
use std::time::Duration;

use crate::components::service_banner;
use crate::service::{start_service_flow, ServiceStatus};
use crate::utils::open_file_dialog;

#[derive(Debug, Clone)]
pub enum Message {
    ToggleFileSelection(usize),
    AddFileRequest,
    AddFiles(Vec<String>),
    RemoveSelectedFiles,
    RestartService,
    DeleteService,
    ServiceStatusUpdated(ServiceStatus),
    PollServiceStatus,
}

#[derive(Debug, Clone)]
pub struct LockdownPanel {
    files: Vec<String>,
    selected_files: HashSet<usize>, // Track selected files by index
    service_status: ServiceStatus,
}

impl LockdownPanel {
    pub fn view(&self) -> Element<Message> {
        let file_list = self
            .files
            .iter()
            .enumerate()
            .fold(column![], |col, (index, file)| {
                col.push(
                    row![checkbox(file, self.selected_files.contains(&index))
                        .on_toggle(move |_| Message::ToggleFileSelection(index)),]
                    .spacing(12),
                )
            });

        let banner = if self.service_status != ServiceStatus::Running {
            service_banner(self.service_status)
        } else {
            column![].into()
        };

        let menu = row![
            button(text("添加")).on_press(Message::AddFileRequest),
            button(text("移除")).on_press(Message::RemoveSelectedFiles),
            button(text("重启/注册服务")).on_press(Message::RestartService),
            button(text("删除服务（还没做）")).on_press(Message::DeleteService),
        ]
        .spacing(12);

        column![banner, menu, scrollable(file_list),]
            .spacing(12)
            .padding(Padding::from([12, 20]))
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
            Message::AddFileRequest => {
                Task::future(async {
                    Message::AddFiles(open_file_dialog().unwrap())
                })
            }
            Message::RemoveSelectedFiles => {
                let mut keep =
                    (0..self.files.len()).map(|index| !self.selected_files.contains(&index));
                self.files.retain(|_| keep.next().unwrap());
                self.selected_files.clear();
                self.save_files();

                Task::none()
            }
            Message::RestartService => {
                self.service_status = ServiceStatus::Starting;

                Task::future(async {
                    if let Err(e) = start_service_flow(true) {
                        println!("Error starting service: {}", e);
                        Message::ServiceStatusUpdated(ServiceStatus::Error)
                    } else {
                        Message::ServiceStatusUpdated(ServiceStatus::Running)
                    }
                })
            }
            Message::DeleteService => {
                println!("Delete Service");

                Task::none()
            }
            Message::ServiceStatusUpdated(status) => {
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
                self.save_files();

                Task::none()
            }
        }
    }

    fn save_files(&self) {
        crate::store::write_data_file(&self.files).unwrap();
        println!("Files saved: {:?}", self.files); // Placeholder for saving
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
        }
    }
}
