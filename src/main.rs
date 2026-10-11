use std::{fs, thread};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::sync::{OnceLock};
use std::time::Duration;
use anyhow::bail;
use enigo::{Direction, Enigo, Keyboard, Settings};
use tokio::sync::Mutex;
use tokio::time;
use tokio_util::sync::CancellationToken;

mod config;

const PHANTOM_STRIKE_CAST_LOG: &str = r#"sounds\mods\armor\phantom_strike\phantom_strike_cast_01"#;

static ENIGO: OnceLock<Mutex<Enigo>> = OnceLock::new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::get();
    let enigo = Enigo::new(&Settings::default())?;
    ENIGO.set(Mutex::new(enigo)).unwrap();

    let log_file_path = config.get_console_path();
    println!("Log file path: {log_file_path:?}");

    if !fs::exists(&log_file_path)? {
        bail!("Log file does not exist!");
    }

    let file = OpenOptions::new()
        .read(true)
        .open(&log_file_path)?;
    let mut reader = BufReader::new(file);

    // ignore existing lines
    reader.seek(SeekFrom::End(0))?;
    println!("Starting monitor loop with a frequency of {} ms. Hit Control-C to exit.", config.update_frequency_ms);

    let cancel = CancellationToken::new();

    {
        let cancel_c = cancel.clone();
        ctrlc::set_handler(move || {
            cancel_c.cancel();
        })?;
    }

    let mut buf = String::new();
    loop {
        if cancel.is_cancelled() {
            break;
        }

        buf.clear();

        let bytes_read = reader.read_line(&mut buf)?;
        if bytes_read == 0 {
            time::sleep(Duration::from_millis(config.update_frequency_ms)).await;
            continue;
        }

        if buf.contains(PHANTOM_STRIKE_CAST_LOG) {
            println!("Phantom Strike cast detected!");
            tokio::spawn(async move {
                send_parry_input().await.unwrap();
            });
        }
    }

    println!("Exiting...");
    Ok(())
}

async fn send_parry_input() -> anyhow::Result<()> {
    let config = config::get();
    let delay_ms = rand::random::<u64>() % (config.parry_delay_max_ms - config.parry_delay_min_ms) + config.parry_delay_min_ms;
    time::sleep(Duration::from_millis(delay_ms)).await;

    let mut enigo = ENIGO.get().unwrap().lock().await;

    let keycode = config.parry_key_code;
    enigo.raw(keycode, Direction::Press)?;
    time::sleep(Duration::from_millis(100)).await;
    enigo.raw(keycode, Direction::Release)?;
    drop(enigo);

    Ok(())
}