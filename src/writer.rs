use gpui::{App, Global, Task};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::{io::Write, path::Path, sync::Arc, time::Duration};

/// Main holder for data, this has some extra information we need to store for later writing.
struct WriterHolder<Writer>
where
    Writer: std::fmt::Debug + Serialize + for<'de> Deserialize<'de> + Default + Clone + Save,
{
    data: Writer,
    write_task: Option<Task<()>>,
    path: Arc<Path>,
}

impl<Writer> Global for WriterHolder<Writer> where
    Writer:
        std::fmt::Debug + Serialize + for<'de> Deserialize<'de> + Default + Clone + Save + 'static
{
}

/// Main writer trait. This enables the given struct to write to disk upon being updated.
///
/// # Usage
/// ```rs
/// #[derive(Debug, Serialize, Deserialize, Default, Clone)]
/// pub struct Config {
///     pub active_theme: SharedString,
/// }
///
/// impl Writer for Config {
///     fn get_name() -> &'static str {
///         "Config"
///     }
/// }
/// ```
/// And then
/// ```rs
/// Config::init(cx, Path::new("some-directory"))
/// ```
/// There is no way to automatically initialise the struct. If you wanat a file to put init functions, see [init]
///
/// # File Name
/// The file will be named after whatever the [Writer::get_name] function returns + `.json`.
///
/// # Notes
/// Note that [Writer] doesn't require [crate::GlobalExt], this is because the inner data itself doesn't need to be global, only the outer [WriterHolder].
pub trait Writer
where
    Self:
        std::fmt::Debug + Clone + Save + Serialize + for<'de> Deserialize<'de> + Default + 'static,
{
    /// Initialise the writer. This attempts to read from the given path.
    fn init(cx: &mut App, path: &Path) {
        let path: Arc<Path> = path.join(format!("{}.json", Self::get_name())).into();
        cx.set_global(WriterHolder {
            data: try_read_json::<Self>(&path),
            write_task: None,
            path,
        });
    }

    /// Same as [Writer::get] but returns a clone of the data instead.
    fn get_copy(cx: &App) -> Self {
        cx.global::<WriterHolder<Self>>().data.clone()
    }

    /// Get a reference to the data stored in global.
    fn get(cx: &App) -> &Self {
        &cx.global::<WriterHolder<Self>>().data
    }

    /// Forceable save the data.
    /// In theory, [Writer::get_mut] should deal with most of the save.
    fn force_save(cx: &mut App) {
        cx.global_mut::<WriterHolder<Self>>().write_to_disk();
    }

    /// Get a mutable version of the struct.
    ///
    /// Will automatically save to disk after a given amount of time. Timeout is specified by a hard coded value.
    fn get_mut(cx: &mut App) -> &mut Self {
        if cx.global::<WriterHolder<Self>>().write_task.is_none() {
            // spawn a new task if we don't have one
            let task = cx.spawn(async |app| {
                // wait given amount of time
                app.background_executor()
                    .timer(Duration::from_secs(5))
                    .await;
                // then save it all.
                _ = app.update_global::<WriterHolder<Self>, _>(|holder, _| {
                    println!("Writing {} to disk!", Self::get_name());
                    holder.write_to_disk();
                });
            });

            let holder = cx.global_mut::<WriterHolder<Self>>();
            holder.write_task = Some(task);
            &mut holder.data
        } else {
            &mut cx.global_mut::<WriterHolder<Self>>().data
        }
    }

    /// Get the name for both the file and debugging. This requires a `&'static str` return value to not break file save location during runtime.
    ///
    /// Default value is `"Data"` but should be overwritten with something more specific.
    fn get_name() -> &'static str {
        "Data"
    }
}

/// Helper functions for code to run related to saving.
///
/// This just allows that extra bit of tweaking, like stripping all whitespace from a string, etc.
pub trait Save {
    /// Run the code before saving to disk.
    fn pre_save(&mut self) {}
}

impl<Writer> WriterHolder<Writer>
where
    Writer: std::fmt::Debug + Serialize + for<'de> Deserialize<'de> + Default + Clone + Save,
{
    /// Write the file to disk, clears the current write_task lock as well.
    fn write_to_disk(&mut self) {
        self.write_task = None;
        let mut config = self.data.clone();
        config.pre_save();
        let Ok(bytes) = serde_json::to_vec(&config) else {
            return;
        };
        _ = write_safe(&self.path, &bytes);
    }
}

/// Attempts to read the file from disk. Will provide a default struct if failed.
pub fn try_read_json<T: std::fmt::Debug + Default + for<'de> Deserialize<'de>>(path: &Path) -> T {
    let Ok(data) = std::fs::read(path) else {
        return T::default();
    };
    serde_json::from_slice(&data).unwrap_or_default()
}

/// Writes the file to disk, but does it in a way which reduces the chances of either file getting corrupted.
/// by using a new file then moving.
pub fn write_safe(path: &Path, content: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut temp = path.to_path_buf();
    temp.add_extension(format!(
        "{}",
        rand::rng().sample(rand::distr::Alphabetic) as char
    ));
    temp.add_extension("new");

    let mut temp_file = std::fs::File::create(&temp)?;

    temp_file.write_all(content)?;
    temp_file.flush()?;
    temp_file.sync_all()?;

    drop(temp_file);

    if let Err(err) = std::fs::rename(&temp, path) {
        _ = std::fs::remove_file(&temp);
        return Err(err);
    }

    Ok(())
}
