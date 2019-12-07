use std::io;
use std::io::BufRead;

use clap::ArgMatches;

pub fn filter(filter_matches: &ArgMatches) {
    let upper_limit = filter_matches
        .value_of("less-than")
        .map(|raw| raw.parse().expect("could not parse option --less-than"));
    let lower_limit = filter_matches
        .value_of("greater-than")
        .map(|raw| raw.parse().expect("could not parse option --greater-than"));
    for line in io::stdin().lock().lines() {
        if let Ok(line) = line {
            let next: f64 = match line.parse() {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("could not parse line: {}", e);
                    continue;
                }
            };
            if let Some(lower_limit) = lower_limit {
                if next < lower_limit {
                    continue;
                }
            }
            if let Some(upper_limit) = upper_limit {
                if next > upper_limit {
                    continue;
                }
            }
            println!("{}", line);
        }
    }
}
