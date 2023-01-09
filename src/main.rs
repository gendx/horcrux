use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use horcrux::field::Field;
use horcrux::gf2n::{GF128, GF16, GF256, GF32, GF64, GF8};
use horcrux::shamir::{CompactShamir, RandomShamir, Shamir};
use rand::thread_rng;
use regex::Regex;
use std::str::FromStr;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

mod mnemonic;
use crate::mnemonic::{Bip39, Bip39Shamir};

#[derive(Copy, Clone)]
enum FormatType {
    Hex,
    Bip39,
}

impl FromStr for FormatType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hex" => Ok(FormatType::Hex),
            "bip39" => Ok(FormatType::Bip39),
            _ => Err(()),
        }
    }
}

fn main() {
    let matches = App::new("Horcrux")
        .version("0.1.0")
        .author("G. Endignoux <ggendx@gmail.com>")
        .about("Split your secrets a.k.a. Shamir's secret sharing")
        .arg(
            Arg::with_name("type")
                .long("type")
                .takes_value(true)
                .possible_values(&["compact", "random"])
                .default_value("compact")
                .help("Type of shares"),
        )
        .arg(
            Arg::with_name("bitsize")
                .long("bitsize")
                .short("b")
                .takes_value(true)
                .possible_values(&["8", "16", "32", "64", "128", "256"])
                .default_value("256")
                .help("Size of the secret in bits"),
        )
        .arg(
            Arg::with_name("nshares")
                .long("nshares")
                .short("n")
                .takes_value(true)
                .required(true)
                .help("Total number of shares (1 <= n <= 255)"),
        )
        .arg(
            Arg::with_name("threshold")
                .long("threshold")
                .short("t")
                .takes_value(true)
                .required(true)
                .help("Minimum number of shares required to reconstruct the secret (1 <= t <= n)"),
        )
        .arg(
            Arg::with_name("format-type")
                .long("format-type")
                .takes_value(true)
                .help("Field value formatting type possible values include: [hex|bip39] [default: hex]")
        )
        .subcommand(
            SubCommand::with_name("split")
                .about("Splits a secret into shares")
                .arg(
                    Arg::with_name("secret")
                        .long("secret")
                        .takes_value(true)
                        .help("Name of a file containing a secret to split [default: generate a random secret instead]"),
                ),
        )
        .subcommand(
            SubCommand::with_name("reconstruct")
                .about("Reconstruct a secret from shares")
                .arg(
                    Arg::with_name("shares")
                        .long("shares")
                        .takes_value(true)
                        .required(true)
                        .help("Name of a file containing the shares to reconstruct from"),
                )
                .arg(
                    Arg::with_name("at")
                        .long("at")
                        .takes_value(true)
                        .help("Where to reconstruct at [default: reconstruct the secret]"),
                ),
        )
        .setting(AppSettings::SubcommandRequired)
        .get_matches();

    let bitsize_str = matches.value_of("bitsize").unwrap();
    let shares_str = matches.value_of("nshares").unwrap();
    let threshold_str = matches.value_of("threshold").unwrap();

    let bitsize = bitsize_str
        .parse::<usize>()
        .expect("--bitsize must be an integer");
    let shares = shares_str
        .parse::<usize>()
        .expect("--shares must be an integer");
    let threshold = threshold_str
        .parse::<usize>()
        .expect("--threshold must be an integer");

    assert!(
        shares != 0 && shares <= 255,
        "--shares must be between 1 and 255"
    );
    assert!(
        threshold != 0 && threshold <= shares,
        "--threshold must be between 1 and --shares"
    );

    let format_type = match matches.value_of("format-type") {
        Some(format_type_str) => format_type_str
            .parse::<FormatType>()
            .expect("--format-type must be one of the following: [hex|bip39]"),
        None => FormatType::Hex,
    };

    let format_error = || panic!("bip39 encoding is only available for 128 and 256 bit keys");

    match bitsize {
        8 => match format_type {
            FormatType::Hex => dispatch_shamir_type::<GF8>(matches, threshold, shares, format_type),
            FormatType::Bip39 => format_error()
        },
        16 => match format_type {
            FormatType::Hex => dispatch_shamir_type::<GF16>(matches, threshold, shares, format_type),
            FormatType::Bip39 => format_error()
        },
        32 => match format_type {
            FormatType::Hex => dispatch_shamir_type::<GF32>(matches, threshold, shares, format_type),
            FormatType::Bip39 => format_error()
        },
        64 => match format_type {
            FormatType::Hex => dispatch_shamir_type::<GF64>(matches, threshold, shares, format_type),
            FormatType::Bip39 => format_error()
        },
        128 => match format_type {
            FormatType::Hex => dispatch_shamir_type::<GF128>(matches, threshold, shares, format_type),
            FormatType::Bip39 => dispatch_mnemonic_shamir_type::<GF128>(matches, threshold, shares, format_type),
        },
        256 => match format_type {
            FormatType::Hex => dispatch_shamir_type::<GF256>(matches, threshold, shares, format_type),
            FormatType::Bip39 => dispatch_mnemonic_shamir_type::<Bip39<GF256>>(matches, threshold, shares, format_type),
        },
        _ => panic!("Unsupported bitsize: {}", bitsize),
    }
}

fn dispatch_shamir_type<F: Field + Debug + Display>(
    matches: ArgMatches,
    k: usize,
    n: usize,
    format_type: FormatType
) {
    let shamir_type = matches.value_of("type").unwrap();
    match shamir_type {
        "compact" => process_command::<F, CompactShamir>(matches, k, n, format_type),
        "random" => process_command::<F, RandomShamir>(matches, k, n, format_type),
        _ => panic!("Unsupported shamir type: {}", shamir_type),
    };
}

fn dispatch_mnemonic_shamir_type<F: Field + Debug + Display>(
    matches: ArgMatches,
    k: usize,
    n: usize,
    format_type: FormatType
) {
    let shamir_type = matches.value_of("type").unwrap();
    match shamir_type {
        "compact" => process_command::<F, Bip39Shamir<CompactShamir>>(matches, k, n, format_type),
        "random" => process_command::<F, Bip39Shamir<RandomShamir>>(matches, k, n, format_type),
        _ => panic!("Unsupported shamir type: {}", shamir_type),
    };
}

fn process_command<F: Field + Debug + Display, S: Shamir<F>>(
    matches: ArgMatches,
    k: usize,
    n: usize,
    format_type: FormatType,
) where
    S::Share: Display,
{
    match matches.subcommand() {
        ("split", Some(args)) => split::<F, S>(args, k, n, format_type),
        ("reconstruct", Some(args)) => reconstruct::<F, S>(args, k),
        (command, _) => panic!("Unsupported command: {}", command),
    };
}

fn split<F: Field + Debug + Display, S: Shamir<F>>(
    args: &ArgMatches,
    k: usize,
    n: usize,
    format_type: FormatType
)
where
    S::Share: Display,
{
    let secret = match args.value_of("secret") {
        None => {
            let mut rng = thread_rng();
            F::uniform(&mut rng)
        }
        Some(filename) => match format_type {
            FormatType::Hex => parse_ascii_secret::<F>(filename),
            FormatType::Bip39 => parse_mnemonic_secret::<F>(filename),
        },
    };
    println!("Secret = {}", secret);

    let shares = S::split(&secret, k, n);
    println!("Shares:");
    for s in &shares {
        println!("{}", s);
    }
}

fn reconstruct<F: Field + Debug + Display, S: Shamir<F>>(args: &ArgMatches, k: usize)
where
    S::Share: Display,
{
    let shares = parse_shares::<F, S>(args.value_of("shares").unwrap());
    println!("Shares:");
    for s in &shares {
        println!("{}", s);
    }

    assert!(
        shares.len() >= k,
        "Found fewer shares than the threshold, cannot reconstruct!"
    );

    match args.value_of("at") {
        Some(at) => {
            let x = S::parse_x(at).unwrap();
            let share = S::reconstruct_at(&shares, k, x);
            match share {
                Some(s) => println!("Share = {}", s),
                None => println!("Could not reconstruct the share..."),
            }
        }
        None => {
            let secret = S::reconstruct(&shares, k);
            match secret {
                Some(s) => println!("Secret = {}", s),
                None => println!("Could not reconstruct the secret..."),
            }
        }
    }
}

fn parse_ascii_secret<F: Field>(filename: &str) -> F {
    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let regex = Regex::new(r"^([0-9a-fA-F]+)\n?$").unwrap();
    let captures = match regex.captures(&contents) {
        Some(cap) => cap,
        None => panic!("Secret file must contains hexadecimal characters only",),
    };

    let bytes = match hex::decode(&captures[1]) {
        Ok(bytes) => bytes,
        Err(e) => panic!(
            "Couldn't parse secret file as hexadecimal characters: {}",
            e
        ),
    };

    match F::from_bytes(bytes.as_slice()) {
        Some(f) => f,
        None => panic!("Secret is not a valid represetation of a field element"),
    }
}

#[cfg(feature = "bip39")]
fn parse_mnemonic_secret<F: Field>(filename: &str) -> F {
    use bip39::{Mnemonic, Language};
    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let mnemonic = Mnemonic::from_phrase(&contents, Language::English).unwrap();
    match F::from_bytes(mnemonic.entropy()) {
        Some(f) => f,
        None => panic!("Secret is not a valid represetation of a field element"),
    }
}

#[cfg(not(feature = "bip39"))]
fn parse_mnemonic_secret<F: Field>(_: &str) -> F {
    panic!("bip39 mnemonics requires the bip39 feature flag")
}

fn parse_shares<F: Field + Debug + Display, S: Shamir<F>>(filename: &str) -> Vec<S::Share> {
    let file = File::open(filename).unwrap();
    BufReader::new(file)
        .lines()
        .map(|line| S::parse_share(&line.unwrap()).unwrap())
        .collect()
}
