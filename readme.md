# gpuiExt
A custom crate i made for the functions i use all the time when working with [gpui](https://gpui.rs)

# Usage
Due to the `gpui` ecosystem not really have a dedicated release cycle, it might be more beneficial to use from git than from crates.io
```toml
[dependencies]
gpui_ext = {
	git = "https://github.com/dragmine149/gpuiExt",
	branch = "git"
}
```
As much as possible will be pinned, but some stuff just won't end up being pinned.

If you still want to use the main pinned version, then run the following
```sh
cargo add gpui_ext
```
or add the following to your `Cargo.toml` file
```toml
[dependencies]
gpui_ext = "0.1.0"
```
