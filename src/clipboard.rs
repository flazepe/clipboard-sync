use anyhow::Result;
use std::{
    cell::RefCell,
    fmt::{Display, Result as FmtResult},
    io::Read,
    rc::Rc,
};
use terminal_clipboard::Clipboard as TerminalClipboard;
use wl_clipboard_rs::{
    copy::{MimeType as CopyMimeType, Options, Source},
    paste::{ClipboardType, Error as PasteError, MimeType as PasteMimeType, Seat, get_contents},
};

pub trait Clipboard: Display {
    fn get(&self) -> Result<String>;
    fn set(&self, value: &str) -> Result<()>;
}

pub struct WlClipboard;

impl Display for WlClipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> FmtResult {
        write!(f, "Wayland")
    }
}

impl Clipboard for WlClipboard {
    fn get(&self) -> Result<String> {
        let result = get_contents(
            ClipboardType::Regular,
            Seat::Unspecified,
            PasteMimeType::Text,
        );

        match result {
            Ok((mut pipe, _)) => {
                let mut contents = vec![];
                pipe.read_to_end(&mut contents)?;
                Ok(String::from_utf8_lossy(&contents).to_string())
            }
            Err(PasteError::NoSeats)
            | Err(PasteError::ClipboardEmpty)
            | Err(PasteError::NoMimeType) => Ok("".to_string()),
            Err(err) => Err(err)?,
        }
    }

    fn set(&self, value: &str) -> Result<()> {
        let opts = Options::new();

        Ok(opts.copy(
            Source::Bytes(value.to_string().into_bytes().into()),
            CopyMimeType::Text,
        )?)
    }
}

pub struct X11Clipboard {
    backend: X11Backend,
}

impl Display for X11Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> FmtResult {
        write!(f, "X11")
    }
}

impl X11Clipboard {
    pub fn new() -> Result<Self> {
        Ok(Self {
            backend: X11Backend::new()?,
        })
    }
}

impl Clipboard for X11Clipboard {
    fn get(&self) -> Result<String> {
        self.backend
            .0
            .try_borrow()?
            .get_string()
            .map_err(anyhow::Error::msg)
    }

    fn set(&self, value: &str) -> Result<()> {
        self.backend
            .0
            .try_borrow_mut()?
            .set_string(value)
            .map_err(anyhow::Error::msg)
    }
}

pub struct X11Backend(Rc<RefCell<terminal_clipboard::X11Clipboard>>);

impl X11Backend {
    fn new() -> Result<Self> {
        let backend = terminal_clipboard::X11Clipboard::new().map_err(anyhow::Error::msg)?;
        Ok(Self(Rc::new(RefCell::new(backend))))
    }
}
