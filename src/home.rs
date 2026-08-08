use crate::data::TransferData;
use gpui::{App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Window, div};
use gpui_component::Root;

pub struct Home {
    data: TransferData,
}
impl Home {
    pub fn view(window: &mut Window, cx: &mut App, data: TransferData) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, data))
    }
    fn new(window: &mut Window, cx: &mut Context<Self>, data: TransferData) -> Self {
        Self { data }
    }
}
impl Render for Home {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let notifications = Root::render_notification_layer(window, cx);
        div().children(notifications)
    }
}
