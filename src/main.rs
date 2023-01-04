use std::borrow::Cow;

use slight::{Flags, Slight};

use clap::Parser;

/// Utility to control backlight brightness smoothly
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Change brightness of device with given id (use --list to find one)
    #[clap(short, long)]
    id: Option<String>,

    /// to_value: 10, by_value: +-10, by_percent: +-10.0%
    #[clap(allow_hyphen_values(true))]
    input: Option<String>,

    /// List all available devices or the one with given id
    #[clap(short, long, conflicts_with("input"))]
    list: Option<Option<String>>,

    /// Use exponential range with given exponent (or default = 4.0)
    #[clap(short, long)]
    exponent: Option<Option<f32>>,

    /// Write to stdout instead of sysfs
    #[clap(short, long)]
    stdout: bool,

    /// Being verbose about what is going on
    #[clap(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    if let Some(list) = args.list {
        if let Some(id) = list {
            Slight::print_device(id.into())
        } else {
            Slight::print_devices()
        }
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });
        return;
    }

    let mut slight = Slight::new(
        args.id.map(Cow::from),
        args.exponent,
        args.input,
        Flags {
            stdout: args.stdout,
            ..Flags::default()
        },
    )
    .unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    slight.set_brightness().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
}
