use crate::data::TransferData;
use gpui::{App, AsyncApp, WeakEntity, AppContext, Context, Entity, IntoElement, ParentElement, Render, Window, div, AnyWindowHandle};
use gpui_component::{Root, notification::Notification};

pub struct Home {
    data: TransferData,
    main_window: AnyWindowHandle,
}
impl Home {
    pub fn view(window: &mut Window, cx: &mut App, data: Option<TransferData>) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, data))
    }
    fn new(window: &mut Window, cx: &mut Context<Self>, data: Option<TransferData>) -> Self {
        Self { 
        	data: data.unwrap(),
        	main_window: window.window_handle()
        }
    }
}

impl Home {
	  /// Even even more shorthand for notification.
    ///
	  /// Unlike normal, this takes in an [gpui::AsyncApp] in order to dispay the notification.
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
    fn notify(&mut self, notification: Notification, cx: &mut Context<Self>) -> anyhow::Result<()> {
        cx.update_window(self.main_window, |_, win, cx| {
            win.push_notification(notification, cx);
        })
    } 
}

impl Render for Home {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let notifications = Root::render_notification_layer(window, cx);
        div().children(notifications)
    }
}
