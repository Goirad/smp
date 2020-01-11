#[macro_use]
extern crate clap;

use std::io;
use std::io::prelude::*;

use clap::App;

mod filter;
mod plot;
mod streaming_variance;

use filter::filter;
use plot::plot;
use streaming_variance::StreamingVariance;

fn main() {
    let yml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yml).get_matches();
    let plot_matches = matches.subcommand_matches("plot");
    let filter_matches = matches.subcommand_matches("filter");

    let mut min = std::f64::MAX;
    let mut max = std::f64::MIN;
    let mut sum = 0.0;
    let mut count = 0;
    let mut ov = StreamingVariance::new();
    let mut vals = Vec::new();

    if let Some(filter_matches) = filter_matches {
        filter(filter_matches);
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
            if matches.is_present("standard-deviation")
                || matches.is_present("mean")
                || matches.is_present("basic")
            {
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
        if matches.is_present("standard-deviation") {
            println!("standard deviation: {:.3}", ov.sample_variance().sqrt());
        }
        if let Some(plot_matches) = plot_matches {
            plot(plot_matches, vals, min, max);
        }
    }
}
