mod clipboard;
mod sync;

use std::{thread::sleep, time::Duration};
use sync::{get_clipboards, keep_synced};

fn main() {
    let mut clipboards = get_clipboards().unwrap();

    loop {
        if let Err(error) = keep_synced(&clipboards) {
            println!("Error while syncing clipboards: {error}");
            clipboards = get_clipboards().unwrap();
            sleep(Duration::from_millis(1000));
        }
    }
}
