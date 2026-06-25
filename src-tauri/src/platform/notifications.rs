use notify_rust::Notification;

pub fn notify(title: &str, body: &str) {
    if let Err(e) = Notification::new()
        .appname("日报助手")
        .summary(title)
        .body(body)
        .show()
    {
        log::warn!("Failed to show notification: {e}");
    }
}
