// #![allow(dead_code, unused_imports)]

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::{collections::HashSet, error::Error, fmt::Debug, sync::LazyLock, time::Duration};

use tauri_winrt_notification::{IconCrop, Toast};

mod config;
mod item;

static ICON_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("rss.png"));

static SOME_PADDING: i64 = 60 * 60 * 12;

fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = config::Config::default();

    let Ok(feeds_urls) = config.parse_feeds() else {
        panic!("something went wrong when parsing feeds")
    };

    println!("# TOAST_INTERVAL: {}", config.toast_duration.as_secs());
    println!("# REFRESH_INTERVAL: {}", config.cycle_interval.as_secs());
    for feed in &feeds_urls {
        println!("#  {}", urlencoding::decode(feed).expect("UTF-8"));
    }
    println!();

    let mut updatetick = 0;
    let mut cache = HashSet::new();
    let mut out = String::new();
    let mut stdout = std::io::stdout();

    loop {
        let n = now();
        println!("#{updatetick} | {} | {} ", n.to_rfc2822(), cache.len()); // TODO? date and time

        let cutoff = n.timestamp() - SOME_PADDING;
        for feed in &feeds_urls {
            let items = match item::fetch_items(feed) {
                Ok(x) => x,
                Err(err) => {
                    // probably just a network error
                    eprintln!(" \x1b[31m-< {}\x1b[0m", err);
                    continue;
                }
            };

            sleep(Duration::from_millis(100));
            print!(".");
            stdout.flush()?;

            for element in items.iter().filter(|x| x.timestamp() > cutoff) {
                // println!("item: {:?}", element.inner());
                if cache.insert(element.clone()) && updatetick > 0 {
                    out.write_str(&format!(" -> {} {}\n", element.link(), element.title()))?;
                    element.show_toast(config.toast_duration);
                }
            }
        }

        print!("end\x1b[1G\x1b[2K{}", out);
        stdout.flush()?;
        out.clear();

        cache.retain(|x| x.timestamp() > cutoff);

        updatetick += 1;
        sleep(config.cycle_interval);
    }
}

// ----------------------------------------------------------------------------------
//   - Toaster -
// ----------------------------------------------------------------------------------
pub(crate) trait Toastable: Debug {
    fn get_title(&self) -> &str;
    fn get_link(&self) -> &str;
    fn get_timestamp(&self) -> i64;

    fn show_toast(&self, wait_sec: Duration) {
        Toast::new(Toast::POWERSHELL_APP_ID)
            .title(self.get_title())
            .text1(self.get_link())
            .icon(&ICON_PATH, IconCrop::Square, "rss")
            .sound(None)
            .show()
            .expect("unable to show toast notification");
        std::thread::sleep(wait_sec);
    }
}

// ----------------------------------------------------------------------------------
// enum AppError {
//     RoxmltreError(roxmltree::Error),
//     VarError(std::env::VarError),
//     ParseInt(ParseIntError),
// }

// impl std::fmt::Debug for AppError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::RoxmltreError(e) => f.debug_tuple("RoxmltreError").field(e).finish(),
//             Self::VarError(e) => f.debug_tuple("VarError").field(e).finish(),
//             Self::ParseInt(e) => f.debug_tuple("ParseIntError").field(e).finish(),
//         }
//     }
// }

// impl From<roxmltree::Error> for AppError {
//     fn from(value: roxmltree::Error) -> Self {
//         Self::RoxmltreError(value)
//     }
// }

// impl From<std::env::VarError> for AppError {
//     fn from(value: std::env::VarError) -> Self {
//         Self::VarError(value)
//     }
// }

// impl From<ParseIntError> for AppError {
//     fn from(value: ParseIntError) -> Self {
//         Self::ParseInt(value)
//     }
// }
