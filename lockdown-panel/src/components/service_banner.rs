use iced::{
    widget::{button, column, text},
    Element, Padding,
};

use crate::{app::Message, service::ServiceStatus};

pub fn service_banner(service_status: ServiceStatus) -> Element<'static, Message> {
    let mut col = column![text(match service_status {
        ServiceStatus::Running => "服务已启动",
        ServiceStatus::Error => "服务启动失败",
        ServiceStatus::Stopped => "服务已停止",
        ServiceStatus::Starting => "服务启动中",
    })
    .size(20),]
    .spacing(8)
    .padding(Padding::from([12, 20]));

    match service_status {
        ServiceStatus::Stopped | ServiceStatus::Error => {
            col = col.push(button("启动服务").on_press(Message::RestartService));
        }
        _ => {}
    }

    col.into()
}
