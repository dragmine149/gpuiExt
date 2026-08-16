use anyhow::anyhow;
use gpui::{AppContext, AsyncApp, Context, WeakEntity};
use gpui_component::{WindowExt, notification::Notification};

/// Send a notification with only a weak version of the entity.
pub trait WeakNotify
where
    Self: Sized + 'static,
{
    /// Even even more shorthand for notification.
    ///
    /// Unlike normal, this takes in an [gpui::AsyncApp] in order to display the notification.
    ///
    /// # Usage
    /// ```rs
    /// let _ = Self::weak_notify(this, Notification::new(), cx);
    /// ```
    fn weak_notify(
        this: &WeakEntity<Self>,
        notification: Notification,
        cx: &mut AsyncApp,
    ) -> anyhow::Result<()> {
        this.update(cx, |this, cx| this.notify(notification, cx))?
    }

    /// Shorthand for notification, saves repeating it a bit.
    ///
    /// NOTE: If there is no active window, the notification will be dropped and an error will get returned.
    ///
    /// See [WeakNotify::weak_notify] for a better entry point.
    fn notify(&mut self, notification: Notification, cx: &mut Context<Self>) -> anyhow::Result<()> {
        let Some(win) = cx.active_window() else {
            return Err(anyhow!("No window is active"));
        };
        cx.update_window(win, |_, win, cx| {
            win.push_notification(notification, cx);
        })
    }
}
