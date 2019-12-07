extern crate clap;

use std::io;
use std::io::prelude::*;

use clap::{Arg, App, SubCommand, ArgMatches};

fn main() {
    let matches = App::new("Streaming Math Processor")
                    .version("0.1.0")
                    .author("Dario Gonzalez <goiradio1@gmail.com>")
                    .about("Command line utility for math and stats")
                    .arg(Arg::with_name("min")
                            .long("min")
                            .help("print min value"))
                    .arg(Arg::with_name("max")
                            .long("max")
                            .help("print max value"))
                    .arg(Arg::with_name("count")
                            .long("count")
                            .help("the number of values processed"))
                    .arg(Arg::with_name("mean")
                            .long("mean")
                            .help("print mean value"))
                    .arg(Arg::with_name("sum")
                            .long("sum")
                            .help("print the sum of input values"))
                    .arg(Arg::with_name("standard_deviation")
                            .long("standard-deviation")
                            .visible_alias("sig")
                            .help("print the sample standard deviation"))
                    .arg(Arg::with_name("basic")
                            .long("basic")
                            .help("prints the count, min, max, and mean"))
                    .subcommand(SubCommand::with_name("plot")
                                .about("plots the data visually")
                                .arg(Arg::with_name("log-x")
                                        .long("log-x")
                                        .help("use a log scale for the x axis"))
                                .arg(Arg::with_name("log-x-rev")
                                        .long("log-x-rev")
                                        .help("use a reverse log scale for the x axis"))
                                .arg(Arg::with_name("log-y")
                                        .long("log-y")
                                        .help("use log scale for the y (count) axis")))
                    .subcommand(SubCommand::with_name("filter")
                                .about("filters streamed numbers")
                                .arg(Arg::with_name("less-than")
                                        .long("less-than")
                                        .visible_alias("lt")
                                        .takes_value(true)
                                        .value_name("VALUE")
                                        .allow_hyphen_values(true)
                                        .help("only pass on numbers less than VALUE"))
                                .arg(Arg::with_name("greater-than")
                                        .long("greater-than")
                                        .visible_alias("gt")
                                        .takes_value(true)
                                        .value_name("VALUE")
                                        .allow_hyphen_values(true)
                                        .help("only pass on numbers greater than VALUE")))
                    .get_matches();
    let plot_matches = matches.subcommand_matches("plot");
    let filter_matches = matches.subcommand_matches("filter");

    let mut min = std::f64::MAX;
    let mut max = std::f64::MIN;
    let mut sum = 0.0;
    let mut count = 0;
    let mut ov = OnlineVariance::new();
    let mut vals = Vec::new();

    if let Some(filter_matches) = filter_matches {
        let upper_limit = filter_matches.value_of("less-than").map(|raw| raw.parse().expect("could not parse option --less-than"));
        let lower_limit = filter_matches.value_of("greater-than").map(|raw| raw.parse().expect("could not parse option --greater-than"));
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
                        continue
                    }
                }
                if let Some(upper_limit) = upper_limit {
                    if next > upper_limit {
                        continue
                    }
                }
                println!("{}", line);
            }
        }
    } else {
        for line in io::stdin().lock().lines() {
            let next: f64 = match line.expect("could not read a line from stdin").parse() {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("could not parse line: {}", e);
                    continue;
                }
            };
            if plot_matches.is_some() {
                vals.push(next);
            }
            if matches.is_present("standard_deviation") || matches.is_present("mean") {
                ov.update(next);
            }
            // The assumption is these are so cheap it isn't worth it to gate them on flags
            count += 1;
            min = min.min(next);
            max = max.max(next);
            sum += next;
        }
        if matches.is_present("count") || matches.is_present("basic") {
            println!("count: {}", count);
        }
        if matches.is_present("min") || matches.is_present("basic") {
            println!("min:   {:.3}", min);
        }
        if matches.is_present("mean") || matches.is_present("basic") {
            println!("mean:  {:.3}", ov.mean());
        }
        if matches.is_present("max") || matches.is_present("basic") {
            println!("max:   {:.3}", max);
        }
        if matches.is_present("sum") {
            println!("sum: {}", sum);
        }
        if matches.is_present("standard_deviation") {
            println!("standard deviation: {:.3}", ov.sample_variance().sqrt());
        }
        if let Some(plot_matches) = plot_matches {
            plot(plot_matches, vals, min, max);
        }
    }
}

fn plot(plot_matches: &ArgMatches, vals: Vec<f64>, min: f64, max: f64) {
    let buckets = if plot_matches.is_present("log-x") {
        bucketize_log(&vals, 40, min, max)
    } else if plot_matches.is_present("log-x-rev") {
        bucketize_log_rev(&vals, 40, min, max)
    } else {
        bucketize(&vals, 40, min, max)
    };
    let bucket_max = *buckets.iter().max().unwrap();
    let tile_width = if plot_matches.is_present("log-y") {
        80.0 / (*buckets.iter().max().unwrap() as f64).log(10.0)
    } else {
        80.0 / *buckets.iter().max().unwrap() as f64
    };
    print!("{:>8}  ", "");
    for i in 1..5 {
        if plot_matches.is_present("log-y") {
            print!("{:>18.3} |", 10.0f64.powf(( bucket_max as f64 ).log10() * i as f64 / 4.0 ));
        } else {
            print!("{:>18.3} |", bucket_max * i / 4 );
        }
    }
    println!();
    for (i, bucket) in buckets.iter().enumerate() {
        if plot_matches.is_present("log-x") {
            print!("{:>8.2}: ", (max - min + 1.0).powf(i as f64/ 40.0) + min - 1.0);
        } else if plot_matches.is_present("log-x-rev") {
            print!("{:>8.2}: ", max + 1.0 - (max - min + 1.0).powf(1.0 - i as f64/ 40.0));
        } else {
            print!("{:>8.2}: ", min + ( i as f64 ) * ( max - min ) / 40.0 );
        };
        if *bucket > 1 {
            let tiles = if plot_matches.is_present("log-y") {
                (tile_width * (*bucket as f64).log10()) as u64
            } else {
                (tile_width * *bucket as f64).ceil() as u64
            };
            for _ in 0..tiles {
                print!("#");
            }
        } else if *bucket > 0 {
            print!("X");
        }
        println!();
    }
}

fn bucketize(vals: &[f64], num_buckets: usize, min: f64, max: f64) -> Vec<u64> {
    let mut buckets = vec!(0; num_buckets);
    let bucket_size = ( max * 1.000001 - min ) / num_buckets as f64;
    for val in vals {
        let mut bucket = ((val - min)/bucket_size).floor() as usize;
        if bucket == num_buckets {
            bucket -= 1;
        }
        buckets[bucket] += 1;
    }
    buckets
}

fn bucketize_log(vals: &[f64], num_buckets: usize, min: f64, max: f64) -> Vec<u64> {
    let mut buckets = vec!(0; num_buckets);
    // The boundaries of each bucket are determined as follows
    // The largest possible 9 values is -log10(1 - 1 / count)
    // The bounds of each bucket is min * ( max / min )^( bucket_number / num_buckets )
    for val in vals {
        let mut bucket = ((val - min + 1.0).log(max - min + 1.0) * num_buckets as f64).trunc() as usize;
        if bucket == num_buckets {
            bucket -= 1;
        }
        buckets[bucket] += 1;
    }
    buckets
}

fn bucketize_log_rev(vals: &[f64], num_buckets: usize, min: f64, max: f64) -> Vec<u64> {
    let mut buckets = vec!(0; num_buckets);
    // The boundaries of each bucket are determined as follows
    // The largest possible 9 values is -log10(1 - 1 / count)
    // The bounds of each bucket is min * ( max / min )^( bucket_number / num_buckets )
    for val in vals {
        let mut bucket = ((num_buckets as f64) * (1.0 - (max + 1.0 - val).log(max - min + 1.0))).trunc() as usize;
        if bucket == num_buckets {
            bucket -= 1;
        }
        buckets[bucket] += 1;
    }
    buckets
}
struct OnlineVariance {
    count: u64,
    mean: f64,
    variance: f64,
}

impl OnlineVariance {
    pub fn new() -> Self {
        OnlineVariance {
            count: 0,
            mean: 0.0,
            variance: 0.0,
        }
    }

    pub fn update(&mut self, new_val: f64) {
        self.count += 1;
        let delta = new_val - self.mean;
        self.mean += delta / ( self.count as f64 );
        let delta2 = new_val - self.mean;
        self.variance += delta * delta2;
    }

    pub fn mean(&self) -> f64 {
        self.mean
    }

    pub fn variance(&self) -> f64 {
        self.variance / ( self.count as f64 )
    }

    pub fn standard_deviation(&self) -> f64 {
        self.variance().sqrt()
    }

    pub fn sample_variance(&self) -> f64 {
        self.variance / ( self.count as f64 - 1.0 )
    }

    pub fn sample_standard_deviation(&self) -> f64 {
        self.sample_variance().sqrt()
    }

    pub fn count(&self) -> u64 {
        self.count
    }
}
