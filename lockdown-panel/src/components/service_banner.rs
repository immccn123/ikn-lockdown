use iced::{
    widget::{button, container, horizontal_space, row},
    Alignment, Element, Length, Padding,
};

use crate::{app::Message, service::ServiceStatus};

pub fn service_banner(
    service_status: ServiceStatus,
    restart_required: bool,
) -> Element<'static, Message> {
    let message = if !restart_required {
        match service_status {
            ServiceStatus::Running => "服务已启动",
            ServiceStatus::Error => "服务启动失败",
            ServiceStatus::Stopped => "服务已停止",
            ServiceStatus::Starting => "服务启动中",
        }
    } else {
        "重启服务以应用更改"
    };

    let mut element = row![container(message).padding(Padding::from([6, 0]))]
        .spacing(8)
        .padding(Padding::from([12, 20]))
        .align_y(Alignment::Center);

    match service_status {
        ServiceStatus::Stopped | ServiceStatus::Error => {
            element = element.push(horizontal_space()).push(
                button("启动服务")
                    .on_press(Message::RestartService)
                    .style(button::success),
            );
        }
        ServiceStatus::Starting => {}
        _ if restart_required => {
            element = element.push(horizontal_space()).push(
                button("重启服务")
                    .on_press(Message::RestartService)
                    .style(button::success),
            );
        }
        _ => {}
    }

    container(element)
        .width(Length::Fill)
        .style(container::rounded_box)
        .into()
}
