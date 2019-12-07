extern crate clap;

use std::io;
use std::io::prelude::*;

use clap::{Arg, App};

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
                    .arg(Arg::with_name("plot")
                            .long("plot")
                            .help("plots the data visually"))
                    .arg(Arg::with_name("plot-log")
                            .long("plot-log")
                            .help("plot the data visually with log scale"))
                    .arg(Arg::with_name("plot-log-rev")
                            .long("plot-log-rev")
                            .help("plot the data visually with reverse log scale"))
                    .get_matches();

    let mut min = std::f64::MAX;
    let mut max = std::f64::MIN;
    let mut sum = 0.0;
    let mut count = 0;
    let mut ov = OnlineVariance::new();
    let mut vals = Vec::new();

    for line in io::stdin().lock().lines() {
        let next: f64 = match line.expect("could not read a line from stdin").parse() {
            Ok(n) => n,
            Err(e) => {
                eprintln!("could not parse line: {}", e);
                continue;
            }
        };
        if matches.is_present("plot") || matches.is_present("plot-log") || matches.is_present("plot-log-rev") {
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
    if matches.is_present("plot") || matches.is_present("plot-log") || matches.is_present("plot-log-rev") {
        let buckets = if matches.is_present("plot") {
            bucketize(&vals, 40, min, max)
        } else if matches.is_present("plot-log") {
            bucketize_log(&vals, 40, min, max)
        } else if matches.is_present("plot-log-rev") {
            bucketize_log_rev(&vals, 40, min, max)
        } else {
            unreachable!()
        };
        let bucket_max = *buckets.iter().max().unwrap();
        let tile_width = 80.0 / *buckets.iter().max().unwrap() as f64;
        print!("{:>8}  ", "");
        for i in 1..5 {
            print!("{:>18} |", bucket_max * i / 4 );
        }
        println!();
        for (i, bucket) in buckets.iter().enumerate() {
            if matches.is_present("plot") {
                print!("{:>8.2}: ", min + ( i as f64 ) * ( max - min ) / 40.0 );
            } else if matches.is_present("plot-log") {
                print!("{:>8.2}: ", (max - min + 1.0).powf(i as f64/ 40.0) + min - 1.0);
            } else if matches.is_present("plot-log-rev") {
                print!("{:>8.2}: ", max + 1.0 - (max - min + 1.0).powf(1.0 - i as f64/ 40.0));
            } else {
                unreachable!()
            };
            if *bucket > 0 {
                for _ in 0..(tile_width * *bucket as f64).ceil() as u64 {
                    print!("#");
                }
            }
            println!();
        }
    }
}

fn bucketize(vals: &[f64], num_buckets: usize, min: f64, max: f64) -> Vec<u64> {
    let mut buckets = vec!(0; num_buckets);
    let bucket_size = ( max * 1.000001 - min ) / num_buckets as f64;
    for val in vals {
        buckets[((val - min)/bucket_size).floor() as usize] += 1;
    }
    buckets
}

fn bucketize_log(vals: &[f64], num_buckets: usize, min: f64, max: f64) -> Vec<u64> {
    let mut buckets = vec!(0; num_buckets);
    // The boundaries of each bucket are determined as follows
    // The largest possible 9 values is -log10(1 - 1 / count)
    // The bounds of each bucket is min * ( max / min )^( bucket_number / num_buckets )
    for val in vals {
        let bucket = (val - min + 1.0).log(max - min + 1.0)*0.999999*num_buckets as f64;
        buckets[bucket.floor() as usize] += 1;
    }
    buckets
}

fn bucketize_log_rev(vals: &[f64], num_buckets: usize, min: f64, max: f64) -> Vec<u64> {
    let mut buckets = vec!(0; num_buckets);
    // The boundaries of each bucket are determined as follows
    // The largest possible 9 values is -log10(1 - 1 / count)
    // The bounds of each bucket is min * ( max / min )^( bucket_number / num_buckets )
    for val in vals {
        let bucket = (num_buckets as f64) * (1.0 - (max + 1.0 - val).log(max - min + 1.0)) * 0.999999;
        buckets[bucket.floor() as usize] += 1;
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
