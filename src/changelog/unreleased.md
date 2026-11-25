The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

The sections should follow the order `Added`, `Changed`, `Deprecated`,
`Removed`, and `Fixed`.

Platform specific changed should be added to the end of the section and grouped
by platform name. Common API additions should have `, implemented` at the end
for platforms where the API was initially implemented. See the following example
on how to add them:

```md
### Added

- Add `Window::turbo()`, implemented on X11, Wayland, and Web.
- On X11, add `Window::some_rare_api`.
- On X11, add `Window::even_more_rare_api`.
- On Wayland, add `Window::common_api`.
- On Windows, add `Window::some_rare_api`.
```

When the change requires non-trivial amount of work for users to comply
with it, the migration guide should be added below the entry, like:

```md
- Deprecate `Window` creation outside of `EventLoop::run`

  This was done to simply migration in the future. Consider the
  following code:

  // Code snippet.

  To migrate it we should do X, Y, and then Z, for example:

  // Code snippet.

```

The migration guide could reference other migration examples in the current
changelog entry.

## Unreleased

### Added

- On Windows, add `WindowAttributesExtWindows::with_extend_content_into_title_bar` to extend the client area over the caption while keeping the native frame.
- On Windows, add `WindowExtWindows::set_title_bar_caption` to mark client regions of such a window as its caption: `WM_NCHITTEST` answers `HTCAPTION` there, and a right click there opens the system menu.
- On Windows, add `WindowExtWindows::set_title_bar_maximize` to mark the application's maximize button of such a window: `WM_NCHITTEST` answers `HTMAXBUTTON` there, so Windows 11 shows Snap Layouts on hover, while its mouse input still reaches the application as cursor and button events.

### Fixed

- On macOS, emit `WindowEvent::Occluded` with the current occlusion state when a fullscreen exit finishes, so `Window::fullscreen()` can be re-read after the state changes.
- On Windows, entering or leaving borderless fullscreen emits one `WindowEvent::Resized` with the final size. The style change used to report the old outer size as an intermediate client size first, so applications drew a frame at that size.
- On Wayland, the end of an interactive resize no longer emits a second `WindowEvent::Resized` with the unchanged size. The compositor clearing the `resizing` state was treated as a resize.
- On X11, `Window::show_window_menu` opens the window manager's window menu by sending `_GTK_SHOW_WINDOW_MENU`, as GTK does; it used to do nothing.
- On X11, mouse scroll button events no longer cause duplicated `WindowEvent::MouseWheel` events.
- On Wayland, an empty `Ime::Preedit` is no longer sent again when the last preedit was already empty, and IME events are no longer delivered to a window whose IME is disabled.
- On X11, `Window::set_ime_cursor_area` reports the bottom-right corner of the area as the IME position, so the candidate window no longer covers the text being typed.
- On Wayland, `Window::drag_window`, `Window::drag_resize_window` and `Window::show_window_menu` work while a finger is down on the window: they use the serial of the touch down instead of only the last pointer button serial.
