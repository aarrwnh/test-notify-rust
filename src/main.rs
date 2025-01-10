// #![allow(dead_code, unused_imports)]

use std::fmt::Write as _;
use std::io::Write as _;
use std::rc::Rc;
use std::thread::sleep;
use std::{collections::HashSet, error::Error, time::Duration};

use chrono::{DateTime, Utc};

use rss_alert::{config, item};

static PUBDATE_OFFSET: i64 = 60 * 60 * 12;

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

    let mut entries = 0;
    let mut cache = HashSet::new();
    let mut out = String::new();
    let mut stdout = std::io::stdout();

    loop {
        let n = Utc::now();
        let cutoff = n.timestamp() - PUBDATE_OFFSET;

        for feed in &feeds_urls {
            let items = match item::fetch_items(feed) {
                Ok(x) => x,
                Err(err) => {
                    // probably just a network error
                    eprintln!("\x1b[31m-< {err}\x1b[0m");
                    continue;
                }
            };

            sleep(Duration::from_millis(100));

            print!(".");
            stdout.flush()?;

            for el in items.iter().filter(|x| x.timestamp() > cutoff) {
                if cache.insert(Rc::clone(el)) && entries > 0 {
                    let pub_date = DateTime::from_timestamp(el.timestamp(), 0)
                        .map(|dt| dt.format("%H:%M"))
                        .expect("item publication date");
                    out.write_str(&format!("{} | {} | {}\n", pub_date, el.link(), el.title()))?;
                    el.show_toast(config.toast_duration);
                }
            }
        }

        print!("end\x1b[1G\x1b[2K{out}");
        stdout.flush()?;

        out.clear();

        cache.retain(|x| x.timestamp() > cutoff);
        entries = cache.len();

        sleep(config.cycle_interval);
    }
}
