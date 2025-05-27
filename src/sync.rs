use crate::clipboard::{Clipboard, ClipboardContents, WlClipboard, X11Clipboard};
use anyhow::{Result, bail};
use std::{env::set_var, thread::sleep, time::Duration};

pub fn get_clipboards() -> Result<Vec<Box<dyn Clipboard>>> {
    let clipboards = vec![
        get_clipboard(get_wayland_clipboard)?,
        get_clipboard(get_x11_clipboard)?,
    ];

    println!("Clipboards found!");

    let initial_value = clipboards
        .iter()
        .map(|clipboard| clipboard.get().unwrap_or_default())
        .find(|value| !value.contents.is_empty())
        .unwrap_or_default();

    // println!("Initial clipboard contents: '{initial_value}'");

    for clipboard in &clipboards {
        clipboard.set(&initial_value)?;
    }

    Ok(clipboards)
}

fn get_clipboard<T: Fn(Option<u8>) -> Result<Box<dyn Clipboard>>>(
    clipboard_fn: T,
) -> Result<Box<dyn Clipboard>> {
    if let Ok(clipboard) = clipboard_fn(None) {
        return Ok(clipboard);
    }

    println!("Display environment variable not found. Attempting to detect display...");

    for i in 0..u8::MAX {
        if let Ok(clipboard) = clipboard_fn(Some(i)) {
            return Ok(clipboard);
        }
    }

    bail!("Could not get Wayland/X11 clipboard");
}

fn get_wayland_clipboard(display: Option<u8>) -> Result<Box<dyn Clipboard>> {
    if let Some(display) = display {
        unsafe {
            set_var("WAYLAND_DISPLAY", format!("wayland-{display}"));
        }
    }

    let clipboard = WlClipboard;
    clipboard.get()?;

    Ok(Box::new(clipboard))
}

fn get_x11_clipboard(display: Option<u8>) -> Result<Box<dyn Clipboard>> {
    if let Some(display) = display {
        unsafe {
            set_var("DISPLAY", format!(":{display}"));
        }
    }

    let clipboard = X11Clipboard::new()?;
    clipboard.get()?;

    Ok(Box::new(clipboard))
}

pub fn keep_synced(clipboards: &Vec<Box<dyn Clipboard>>) -> Result<()> {
    loop {
        sleep(Duration::from_millis(100));

        let new_value = await_change(clipboards)?;

        for clipboard in clipboards {
            clipboard.set(&new_value)?;
        }
    }
}

fn await_change(clipboards: &Vec<Box<dyn Clipboard>>) -> Result<ClipboardContents> {
    let initial_value = clipboards[0].get()?;

    loop {
        for clipboard in clipboards {
            let new_value = clipboard.get()?;

            if new_value.contents != initial_value.contents {
                println!("Clipboard updated from the {clipboard} clipboard");
                // println!("Clipboard contents: '{new_value}'");
                return Ok(new_value);
            }
        }
        sleep(Duration::from_millis(200));
    }
}
