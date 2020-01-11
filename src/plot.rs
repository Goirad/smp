use clap::ArgMatches;

struct PlotArgs {
    min: f64,
    max: f64,
    width: u64,
    height: u64,
    num_labels: u64,
    log_x: bool,
    log_y: bool,
    log_x_rev: bool,
    plot_empty: bool,
}

impl PlotArgs {
    fn new(plot_matches: &ArgMatches, min: f64, max: f64) -> Self {
        let width = plot_matches
            .value_of("width")
            .expect("has default")
            .parse::<u64>()
            .expect("could not parse width argument");
        let height = plot_matches
            .value_of("height")
            .expect("has default")
            .parse::<u64>()
            .expect("could not parse height argument");
        let num_labels = plot_matches
            .value_of("num-labels")
            .expect("has default")
            .parse::<u64>()
            .expect("could not parse num-labels argument");
        let log_x = plot_matches.is_present("log-x");
        let log_x_rev = plot_matches.is_present("log-x-rev");
        let log_y = plot_matches.is_present("log-y");
        let plot_empty = plot_matches.is_present("omit-empty");

        PlotArgs {
            min,
            max,
            width,
            height,
            num_labels,
            log_x,
            log_y,
            log_x_rev,
            plot_empty,
        }
    }
}


pub fn plot(plot_matches: &ArgMatches, vals: Vec<f64>, min: f64, max: f64) {
    let plot_args = PlotArgs::new(plot_matches, min, max);
    do_plot(plot_args, vals);
}

fn do_plot(plot_args: PlotArgs, vals: Vec<f64>) {
    if vals.is_empty() {
        eprintln!("no values to plot");
        return
    }
    let buckets = if plot_args.log_x {
        bucketize_log(&vals, plot_args.height as usize, plot_args.min, plot_args.max)
    } else if plot_args.log_x_rev {
        bucketize_log_rev(&vals, plot_args.height as usize, plot_args.min, plot_args.max)
    } else {
        bucketize(&vals, plot_args.height as usize, plot_args.min, plot_args.max)
    };
    let bucket_max = *buckets.iter().max().unwrap();
    let tile_width = if plot_args.log_y {
        plot_args.width as f64 / (*buckets.iter().max().unwrap() as f64).log10()
    } else {
        plot_args.width as f64 / *buckets.iter().max().unwrap() as f64
    };
    let padding = format!("{:.2}", plot_args.max).len();
    print!("{1:>0$}  ", padding, "");
    let width_per_label = plot_args.width as usize / plot_args.num_labels as usize - 2;
    for i in 1..=plot_args.num_labels {
        if plot_args.log_y {
            print!(
                "{1:>0$.3} |",
                width_per_label,
                10.0f64.powf((bucket_max as f64).log10() * i as f64 / plot_args.num_labels as f64 )
            );
        } else {
            print!("{1:>0$.3} |", width_per_label, bucket_max as f64 * i as f64 / plot_args.num_labels as f64 );
        }
    }
    println!();
    for (i, bucket) in buckets.iter().enumerate() {
        if !plot_args.plot_empty || *bucket > 0 {
            if plot_args.log_x {
                print!(
                    "{1:>0$.2}: ",
                    padding,
                    (plot_args.max - plot_args.min + 1.0).powf(i as f64 / plot_args.height as f64) + plot_args.min - 1.0
                );
            } else if plot_args.log_x_rev {
                print!(
                    "{1:>0$.2}: ",
                    padding,
                    plot_args.max + 1.0 - (plot_args.max - plot_args.min + 1.0).powf(1.0 - i as f64 / plot_args.height as f64)
                );
            } else {
                print!("{1:>0$.2}: ", padding, plot_args.min + (i as f64) * (plot_args.max - plot_args.min) / plot_args.height as f64);
            }
            if *bucket > 1 {
                let tiles = if plot_args.log_y {
                    (tile_width * (*bucket as f64).log10()) as u64
                } else {
                    (tile_width * *bucket as f64).round() as u64
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
    println!("{1:>0$.2}", padding, plot_args.max as usize);
}

fn bucketize(vals: &[f64], num_buckets: usize, min: f64, max: f64) -> Vec<u64> {
    let mut buckets = vec![0; num_buckets];
    let bucket_size = (max * 1.000001 - min) / num_buckets as f64;
    for val in vals {
        let mut bucket = ((val - min) / bucket_size).floor() as usize;
        if bucket == num_buckets {
            bucket -= 1;
        }
        buckets[bucket] += 1;
    }
    buckets
}

fn bucketize_log(vals: &[f64], num_buckets: usize, min: f64, max: f64) -> Vec<u64> {
    let mut buckets = vec![0; num_buckets];
    // The boundaries of each bucket are determined as follows
    // The largest possible 9 values is -log10(1 - 1 / count)
    // The bounds of each bucket is min * ( max / min )^( bucket_number / num_buckets )
    for val in vals {
        let mut bucket =
            ((val - min + 1.0).log(max - min + 1.0) * num_buckets as f64).trunc() as usize;
        if bucket == num_buckets {
            bucket -= 1;
        }
        buckets[bucket] += 1;
    }
    buckets
}

fn bucketize_log_rev(vals: &[f64], num_buckets: usize, min: f64, max: f64) -> Vec<u64> {
    let mut buckets = vec![0; num_buckets];
    // The boundaries of each bucket are determined as follows
    // The largest possible 9 values is -log10(1 - 1 / count)
    // The bounds of each bucket is min * ( max / min )^( bucket_number / num_buckets )
    for val in vals {
        let mut bucket = ((num_buckets as f64) * (1.0 - (max + 1.0 - val).log(max - min + 1.0)))
            .trunc() as usize;
        if bucket == num_buckets {
            bucket -= 1;
        }
        buckets[bucket] += 1;
    }
    buckets
}
