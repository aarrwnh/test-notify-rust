use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fmt::Debug,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

pub struct Config {
    /// Path pointing to file containing list of feeds
    pub file_path: PathBuf,
    /// Delay amount of seconds between each [`Toast::new`] call
    /// TODO? this should grab value from somewhere in the registry
    pub toast_duration: Duration,
    /// Interval between each update cycle
    pub cycle_interval: Duration,
}

impl Config {
    pub fn new(cycle_interval: u64, toast_duration: u64) -> Self {
        let args = Self::get_args();
        let file_path = PathBuf::from(args.get("path").expect("--path is required"));

        if !file_path.exists() {
            panic!("file does not exists");
        }

        Self {
            file_path,
            toast_duration: Self::parse_number(toast_duration, args.get("toast")),
            cycle_interval: Self::parse_number(cycle_interval, args.get("interval")),
        }
    }

    pub fn default() -> Self {
        Config::new(600, 5)
    }

    fn parse_number(default: u64, x: Option<&String>) -> Duration {
        Duration::from_secs(match x {
            Some(v) => u64::from_str(v).expect("tried to parse a number"),
            None => default,
        })
    }

    fn get_args() -> HashMap<String, String> {
        std::env::args()
            .skip(1)
            .filter_map(|s| {
                s.split_once('=')
                    .map(|(a, b)| (a.strip_prefix("--").unwrap().to_owned(), b.to_owned()))
            })
            .collect()
    }

    pub fn parse_feeds(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let buf = read_file(&self.file_path)?;
        Ok(parse_feeds_var(&buf))
    }
}

fn read_file<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn Error>> {
    let mut file = std::fs::File::open(path.as_ref())?;
    let mut buf = String::new();
    let _ = file.read_to_string(&mut buf);
    Ok(buf.trim().to_owned())
}

// s = "var1={1|2|3}&var2={4|5}"
// result = [ f"var1={v1}&var2{v2}" for v1 in [1,2,3] for v2 in [4,5] ]
fn parse_feeds_var(s: &str) -> Vec<String> {
    let mut feeds = Vec::new();
    let mut temp = Vec::new();
    let mut args = BTreeMap::new();
    for line in s.split('\n') {
        if line.trim().starts_with("#") {
            continue;
        }

        match line.find('{') {
            Some(first) => {
                let mut start = 0;
                for i in first..line.len() {
                    if &line[i..=i] == "{" {
                        temp.push(&line[start..i]);
                        if let Some(j) = line[i..].find('}') {
                            let v = line[i + 1..i + j].split('|').collect::<Vec<_>>();
                            args.insert(temp.len(), v); // key=index in template
                            temp.push(""); // empty placeholder
                            start = i + j + 1;
                        }
                    }
                }
                temp.push(&line[start..]); // push leftover str

                let entries = args.keys().collect::<Vec<_>>();
                let arrays = args.values().collect::<Vec<_>>();
                for r in combinations(&arrays) {
                    for (pos, replacement) in r.into_iter().enumerate() {
                        temp[*entries[pos]] = replacement;
                    }
                    feeds.push(temp.join(""));
                }
            }
            None => feeds.push(line.to_owned()),
        }
        temp.clear();
        args.clear();
    }
    feeds
}

fn get_combinations<T: Copy + Debug>(n: usize, arrays: &[&Vec<T>], divisors: &[usize]) -> Vec<T> {
    arrays
        .iter()
        .enumerate()
        .map(|(i, arr)| arr[(n / divisors[i]) % arr.len()])
        .collect()
}

fn combinations<T: Copy + Debug>(arrays: &[&Vec<T>]) -> Vec<Vec<T>> {
    let mut divisors = vec![0; arrays.len()];
    let mut count = 1;

    for i in (0..arrays.len()).rev() {
        divisors[i] = match divisors.get(i + 1) {
            Some(v) => v * arrays[i + 1].len(),
            None => 1,
        };
        count *= match arrays[i].len() {
            0 => 1,
            x => x,
        };
    }

    (0..count)
        .map(|i| get_combinations(i, arrays, &divisors))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn expand_vars() {
        let a = parse_feeds_var("{A|B|C}_{D|E}\n# {DD|CC}");
        assert_eq!(a, vec!["A_D", "A_E", "B_D", "B_E", "C_D", "C_E"]);
    }
}