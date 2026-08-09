use crate::{
    data::TransferData,
    home::Home,
    transitions::Transitions,
    writer::{Writer, config::Config},
};
use anyhow::anyhow;
use gpui::{
    App, AppContext, AssetSource, AsyncApp, Context, Entity, Global, KeyBinding, Length,
    ParentElement, SharedString, StyleRefinement, Styled, Task, TitlebarOptions, WeakEntity,
    Window, WindowOptions, actions,
};
use gpui_component::{
    ActiveTheme, Root,
    group_box::{GroupBox, GroupBoxVariants},
    h_flex,
};
use rust_embed::RustEmbed;
use std::{fs, path::PathBuf, sync::mpsc};
pub mod data;
pub(crate) mod home;
pub(crate) mod transitions;
pub(crate) mod writer;

/// A trait to skip some of the announces of creating a new entity for a struct all the time.
///
/// Usage
/// ```rs
/// struct A {};
/// impl GPUIStructHelper for A {
/// 	fn new(window, cx, data) -> Self {
/// 		Self {}
/// 	}
/// }
///
/// fn some_fn(window, cx) -> Entity<A> {
/// 	A::view(window, cx, None)
/// }
/// ```
pub(crate) trait GPUIStructHelper
where
    Self: 'static + Sized,
{
    /// Turn itself into a usable entity.
    fn view(window: &mut Window, cx: &mut App, data: Option<TransferData>) -> Entity<Self>
    where
        Self: Sized,
    {
        cx.new(|cx| Self::new(window, cx, data))
    }
    /// Create the struct itself.
    fn new(window: &mut Window, cx: &mut Context<Self>, data: Option<TransferData>) -> Self;
}

/// Taken from gpui-component story/lib.rs
///
/// Returns a [gpui_component::group_box::GroupBox] template to section off the children.
/// Title is shown outside the section
#[allow(dead_code)]
fn section(title: impl Into<SharedString>, cx: &mut App) -> GroupBox {
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
pub trait GlobalExt: Global + Sized {
    /// Initialise the global with a default version of the given struct.
    fn init_default(cx: &mut App)
    where
        Self: Default,
    {
        cx.set_global(Self::default());
    }
    /// Same as [GlobalExt::get] but returns a clone of the data instead.
    fn get_copy(cx: &App) -> Self
    where
        Self: Clone,
    {
        cx.global::<Self>().clone()
    }
    /// Get a reference to the value stored in global.
    fn get(cx: &App) -> &Self {
        &cx.global::<Self>()
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
pub(crate) fn thread_to_main<AsyncFn, R, T, Cont>(
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
    println!("Setup connections");
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

    println!("Returning spawn obj");
    cx.spawn(async move |this, cx| f(this, cx, rx).await)
}

/// Use a percentage in terms of length. Shorthand for `Length::Definite(gpui::DefiniteLength::Fraction())`
///
/// value is in terms of percentage, hence is valid between 0 and 100. value will also be clamped if it's too high.
#[allow(dead_code)]
pub(crate) fn percent(value: f32) -> Length {
    Length::Definite(gpui::DefiniteLength::Fraction(
        value.clamp(0.0, 100.0) / 100.0,
    ))
}

/// Function for loading a theme. Will also update the config at the same time.
pub(crate) fn load_theme(cx: &mut App, theme_name: &SharedString) {
    if let Some(theme) = gpui_component::ThemeRegistry::global(cx)
        .themes()
        .get(theme_name)
        .cloned()
    {
        let glob_theme = gpui_component::Theme::global_mut(cx);
        glob_theme.apply_config(&theme);
        // println!("{:?}", theme.font_family);
        // println!("{:?}", glob_theme.font_family);
        Config::get_mut(cx).active_theme = theme_name.to_owned();
        println!("Loaded new theme: {:?}", theme_name);
    }
}

/// Holds and loads custom assets.
#[derive(RustEmbed)]
#[folder = "../../assets"]
#[allow_missing = true]
#[include = "*"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }
        Self::get(path)
            .map(|f| Some(f.data))
            .ok_or_else(|| anyhow!("could not find asset at path \"{path}\""))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}

actions!([Quit]);

pub fn main(config_dir: PathBuf, data: TransferData) {
    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .with_assets(Assets)
        .run(move |cx| {
            gpui_component::init(cx);
            // TODO: Sort out themes.
            gpui_component::Theme::change(gpui_component::ThemeMode::Dark, None, cx);

            writer::init_writers(cx, &config_dir);
            Transitions::init(cx);

            let theme_folder = config_dir.join("Themes");
            if !theme_folder.exists() {
                let _ = fs::create_dir(&theme_folder);
            }

            _ = gpui_component::ThemeRegistry::watch_dir(theme_folder.clone(), cx, move |cx| {
                let theme_name = Config::get(cx).active_theme.clone();
                if theme_name.is_empty() {
                    return;
                }
                load_theme(cx, &theme_name);
            });

            // TODO: Customise keybinds? *or at least add more*
            cx.on_app_quit(|cx| {
                Config::force_save(cx);
                async {}
            })
            .detach();
            cx.bind_keys([KeyBinding::new("secondary-q", Quit, None)]);
            cx.on_action(|_: &Quit, cx| {
                for window in cx.windows() {
                    _ = window.update(cx, |_, window, _| {
                        window.remove_window();
                    })
                }
            });

            cx.open_window(
                WindowOptions {
                    app_id: Some(env!("CARGO_PKG_NAME").to_string()),
                    titlebar: Some(TitlebarOptions {
                        title: Some(
                            format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                                .into(),
                        ),
                        ..Default::default()
                    }),

                    tabbing_identifier: Some(env!("CARGO_PKG_NAME").to_string()),
                    ..Default::default()
                },
                |window, cx| {
                    let home = Home::view(window, cx, Some(data));
                    cx.new(|cx| Root::new(home, window, cx))
                },
            )
            .unwrap();
        });
}
