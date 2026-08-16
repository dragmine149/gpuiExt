pub use crate::writer::Writer;
use gpui::{
    App, AppContext, AsyncApp, Context, Entity, Global, Length, ParentElement, SharedString,
    StyleRefinement, Styled, Task, WeakEntity, Window,
};
use gpui_component::{
    ActiveTheme,
    group_box::{GroupBox, GroupBoxVariants},
    h_flex,
};

use log::trace;
use std::sync::mpsc;
pub mod notify;
pub mod writer;

/// A trait to skip some of the announces of creating a new entity for a struct all the time.
///
/// # Usage
/// ```rs
/// struct A {};
/// impl GPUIStructHelper for A {
///     fn new(window, cx, data) -> Self {
///         Self {}
///     }
/// }
///
/// fn some_fn(window, cx) -> Entity<A> {
///     A::view(window, cx, None)
/// }
/// ```
pub trait GPUIStructHelper<D>
where
    Self: 'static + Sized,
{
    /// Turn itself into a usable entity.
    fn view(window: &mut Window, cx: &mut App, data: Option<D>) -> Entity<Self>
    where
        Self: Sized,
    {
        cx.new(|cx| Self::new(window, cx, data))
    }
    /// Create the struct itself.
    fn new(window: &mut Window, cx: &mut Context<Self>, data: Option<D>) -> Self;
}

/// Taken from gpui-component story/lib.rs
///
/// Returns a [gpui_component::group_box::GroupBox] template to section off the children.
/// Title is shown outside the section
pub fn section(title: impl Into<SharedString>, cx: &mut App) -> GroupBox {
    let title = title.into();
    GroupBox::new()
        .w_full()
        .id(title.clone())
        .outline()
        .title(h_flex().justify_between().w_full().gap_4().child(title))
        .content_style(
            StyleRefinement::default()
                .rounded(cx.theme().radius_lg)
                .overflow_x_hidden()
                .items_center()
                .justify_center(),
        )
}

/// Helper trait for assigning structs as global structs.
///
/// Also see [writer::Writer]
///
/// # Usage
/// ```rs
/// #[derive(Default)]
/// struct SomeStruct {
///     data: bool,
/// }
/// impl GlobalExt for SomeStruct {}
///
/// fn main(cx) {
///     SomeStruct::init_default(cx);
///
///     SomeStruct::get_mut(cx).data = true;
///     println!("{}", SomeStruct::get(cx).data); // true
/// }
/// ```
pub trait GlobalExt: Global + Sized {
    /// Initialise the global with the currently created struct.
    ///
    /// This requires the struct to be created first as we don't know if there is a `new` function.
    ///
    /// Also see [GlobalExt::init_default]
    fn init(self, cx: &mut App) {
        cx.set_global(self);
    }
    /// Initialise the global with a default version of the given struct.
    ///
    /// Also see [GlobalExt::init]
    fn init_default(cx: &mut App)
    where
        Self: Default,
    {
        cx.set_global(Self::default());
    }
    /// Same as [GlobalExt::get] but returns a clone of the data instead.
    ///
    /// Note: This clones the whole struct, Preferably use [GlobalExt::get] instead and clone the individual items
    fn get_copy(cx: &App) -> Self
    where
        Self: Clone,
    {
        cx.global::<Self>().clone()
    }
    /// Get a reference to the value stored in global.
    fn get(cx: &App) -> &Self {
        cx.global::<Self>()
    }
    /// Get a mutable reference to the value stored in global.
    fn get_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }
}

/// Move an event from an outsider thread into the inside (gpui)
///
/// # Parameters
/// - cx: Context of the entity for access to said entity. Better than `&mut [gpui::app]`
/// - receiver: The receiver to forward the messages from.
/// - f: The callback function, exactly the same as [gpui::Context::spawn] but with the addition of [async_channel::Receiver] dedicated to the input receiver.
///
/// # Returns
/// Same thing as [gpui::Context::spawn] does, and is what expected by the inner function.
///
/// # Usage
/// ```rs
/// // Somewhere in your codebase
/// let (rx, tx) = channel::<String>();
///
/// // in a gpui struct initialise (new/view) function
/// fn new(window: &mut Window, cx: &mut Context<Self>, rx: Receiver<T>) -> Self {
///     thread_to_main(cx, rx, async |this, cx, rx| {
///         while let Ok(msg) = rx.recv().await {
///             println!("msg: {}", msg);
///         }
///     }).detach();
///     Self { ... }
/// }
///
/// // In a separate thread somewhere
/// while true {
///     tx.send("Hello".to_string());
/// }
/// ```
pub fn thread_to_main<AsyncFn, R, T, Cont>(
    cx: &mut Context<Cont>,
    receiver: mpsc::Receiver<T>,
    f: AsyncFn,
) -> Task<R>
where
    AsyncFn:
        AsyncFnOnce(WeakEntity<Cont>, &mut AsyncApp, async_channel::Receiver<T>) -> R + 'static,
    R: 'static,
    T: Send + 'static,
    Cont: 'static,
{
    trace!("Setup connections");
    let (tx, rx) = async_channel::unbounded::<T>();
    cx.background_spawn(async move {
        loop {
            tx.send(
                receiver
                    .recv()
                    .unwrap_or_else(|err| panic!("Failed to get receiver message {}", err)),
            )
            .await
            .expect("Failed to send receiver message");
        }
    })
    .detach();

    trace!("Returning spawn obj");
    cx.spawn(async move |this, cx| f(this, cx, rx).await)
}

/// Use a percentage in terms of length. Shorthand for `Length::Definite(gpui::DefiniteLength::Fraction())`
///
/// value is in terms of percentage, hence is valid between 0 and 100. value will also be clamped if it's too high.
/// # Parameters
/// - value: A percentage between `0.0` and `100.0`. Will be clamped between those ranges before being defined.
pub fn percent(value: f32) -> Length {
    Length::Definite(gpui::DefiniteLength::Fraction(
        value.clamp(0.0, 100.0) / 100.0,
    ))
}

/// Function for loading a theme. Will also update the config at the same time.
///
/// # Parameters
/// - cx: App context, used for accessing globals.
/// - theme_name: The name of the theme to load
/// - update_fn: Callback function to apply the new theme name to your config.
pub fn load_theme<F>(cx: &mut App, theme_name: &SharedString, update_fn: F)
where
    F: Fn(&SharedString, &mut App),
{
    if let Some(theme) = gpui_component::ThemeRegistry::global(cx)
        .themes()
        .get(theme_name)
        .cloned()
    {
        let glob_theme = gpui_component::Theme::global_mut(cx);
        glob_theme.apply_config(&theme);
        update_fn(theme_name, cx);
    }
}
