use anyhow::Result;
use std::{
    fmt::{Display, Result as FmtResult},
    io::Read,
    time::Duration,
};
use wl_clipboard_rs::{
    copy::{MimeType as CopyMimeType, Options, Source},
    paste::{ClipboardType, MimeType as PasteMimeType, Seat, get_contents},
};

pub struct ClipboardContents {
    pub contents: Vec<u8>,
    pub mime_type: String,
}

impl ClipboardContents {
    pub fn new<T: Display>(contents: Vec<u8>, mime_type: T) -> Self {
        Self {
            contents,
            mime_type: mime_type.to_string(),
        }
    }
}

impl Default for ClipboardContents {
    fn default() -> Self {
        Self::new(vec![], "text/plain")
    }
}

pub trait Clipboard: Display {
    fn get(&self) -> Result<ClipboardContents>;
    fn set(&self, value: &ClipboardContents) -> Result<()>;
}

pub struct WlClipboard;

impl Display for WlClipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> FmtResult {
        write!(f, "Wayland")
    }
}

impl Clipboard for WlClipboard {
    fn get(&self) -> Result<ClipboardContents> {
        let result = get_contents(
            ClipboardType::Regular,
            Seat::Unspecified,
            PasteMimeType::TextWithPriority("image/png"),
        );

        if let Ok((mut pipe, mime_type)) = result {
            let mut contents = vec![];
            pipe.read_to_end(&mut contents)?;
            Ok(ClipboardContents::new(contents, mime_type))
        } else {
            Ok(Default::default())
        }
    }

    fn set(&self, value: &ClipboardContents) -> Result<()> {
        Options::new().copy(
            Source::Bytes(value.contents.as_slice().into()),
            CopyMimeType::Specific(value.mime_type.clone()),
        )?;

        Ok(())
    }
}

pub struct X11Clipboard {
    backend: x11_clipboard::Clipboard,
}

impl Display for X11Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> FmtResult {
        write!(f, "X11")
    }
}

impl X11Clipboard {
    pub fn new() -> Result<Self> {
        Ok(Self {
            backend: x11_clipboard::Clipboard::new()?,
        })
    }
}

impl Clipboard for X11Clipboard {
    fn get(&self) -> Result<ClipboardContents> {
        let contents = self.backend.load(
            self.backend.getter.atoms.clipboard,
            self.backend.getter.atoms.utf8_string,
            self.backend.getter.atoms.property,
            Duration::from_secs(2),
        )?;

        Ok(ClipboardContents::new(contents, "text/plain"))
    }

    fn set(&self, value: &ClipboardContents) -> Result<()> {
        Ok(self.backend.store(
            self.backend.setter.atoms.clipboard,
            self.backend.setter.atoms.utf8_string,
            value.contents.as_slice(),
        )?)
    }
}
