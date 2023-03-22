#[cfg(not(windows))]
use clap::ValueEnum;
use clap::{ArgGroup, Parser};
use std::num::NonZeroUsize;
use std::time::Duration;

const DEFAULT_DELAY: &str = "30s";

#[cfg(not(windows))]
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AllocationMode {
    Available,
    Free,
}

/// Monitors memory for bit-flips (won't work on ECC memory).
/// The chance of detection scales with the physical size of your DRAM modules
/// and the percentage of them you allocate to this program.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(group(
    ArgGroup::new("detector memory size")
        .required(true)
        .args(&["memory_to_monitor", "use_all"])
))]
pub struct Cli {
    #[arg(short, long, value_parser(parse_size_string))]
    /// The size of the memory to monitor for bit flips, understands e.g. 200, 5kB, 2GB and 3MB.
    /// If no suffix is given the program will assume that the given number is the number of bytes to monitor.
    pub memory_to_monitor: Option<NonZeroUsize>,

    // There is a difference between free and available memory,
    // and on most operating systems we can detect this difference.
    // This option lets the user specify which alternative they mean.
    #[cfg(all(not(windows), not(freebsd)))]
    #[arg(long, value_enum, value_name = "ALLOCATION_MODE")]
    /// Allocate as much memory as possible to the detector.
    /// If "free" is specified the program will allocate all currently unused memory,
    /// while if "available" is specified the program will also try to eject things that sit in memory
    /// but haven't been used in a while.
    pub use_all: Option<AllocationMode>,

    // On Windows and FreeBSD there is no way to differentiate free and available memory,
    // so we just allocate as much as the OS gives us.
    #[cfg(any(windows, freebsd))]
    #[arg(long)]
    /// Allocate as much memory as possible to the detector.
    pub use_all: bool,

    #[arg(short, value_parser = parse_delay_string, default_value = DEFAULT_DELAY)]
    /// The delay in between each integrity check.
    pub delay_between_checks: Duration,

    #[arg(long)]
    /// Run the integrity check in parallel.
    pub parallel: bool,

    #[arg(short, long)]
    /// Print extra information.
    pub verbose: bool,
}

/// Parses a string describing a number of bytes into an integer.
/// The string can use common SI prefixes as well, like '4GB' or '30kB'.
pub fn parse_size_string(size_string: &str) -> Result<NonZeroUsize, String> {
    match size_string.parse() {
        // The input was a number, interpret it as the number of bytes if nonzero.
        Ok(t) => NonZeroUsize::new(t).ok_or_else(|| "zero is not a valid value".to_owned()),
        // The input was more than just an integer
        Err(_) => {
            let (number, suffix) = match size_string
                .chars()
                .position(|c| !c.is_ascii_digit() && c != '.')
            {
                Some(index) => Ok(size_string.split_at(index)),
                None => Err("you need to specify a suffix to use non-integer numbers".to_owned()),
            }?;

            let mut num_bytes: f64 = number
                .parse()
                .map_err(|_| format!("could not interpret '{number}' as a number"))?;

            let mut chars: Vec<char> = suffix.chars().collect();
            let original_suffix_len = chars.len();

            if original_suffix_len > 2 {
                return Err("the suffix is too long, it can be at most two letters".to_owned());
            }

            match chars.pop() {
                Some(ending) => {
                    if ending == 'B' || (ending == 'b' && original_suffix_len == 2) {
                        if let Some(si_prefix) = chars.pop() {
                            num_bytes *= parse_si_prefix(si_prefix)?;
                        }
                        if ending == 'b' {
                            num_bytes /= 8.0;
                        }
                    } else {
                        return Err("the suffix must end with either 'B' or 'b' and be two characters long".to_owned());
                    }
                }
                // No suffix
                None => (),
            }

            NonZeroUsize::new(num_bytes as usize).ok_or_else(|| "too small".to_owned())
        }
    }
}

fn parse_si_prefix(c: char) -> Result<f64, String> {
    if c == 'k' {
        Ok(1e3)
    } else if c == 'M' {
        Ok(1e6)
    } else if c == 'G' {
        Ok(1e9)
    } else if c == 'T' {
        Ok(1e12)
    } else if c == 'P' {
        // Values higher than this one should not be needed, but are included for completeness.
        Ok(1e15)
    } else if c == 'E' {
        Ok(1e18)
    } else if c == 'Z' {
        Ok(1e21)
    } else if c == 'Y' {
        Ok(1e24)
    } else {
        Err(format!("'{c}' is an unsupported si prefix"))
    }
}

fn parse_delay_string(s: &str) -> Result<Duration, String> {
    match s.parse::<humantime::Duration>() {
        Ok(d) => Ok(d.into()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod test {
    use super::parse_size_string;

    #[test]
    fn check_memory_parsing() {
        for s in (0..10).map(|i| 2_usize.pow(i)) {
            assert_eq!(parse_size_string(&format!("{s}")).unwrap().get(), s);
            assert_eq!(
                parse_size_string(&format!("{s}kB")).unwrap().get(),
                s * 1000
            );
            assert_eq!(
                parse_size_string(&format!("{s}MB")).unwrap().get(),
                s * 1000000
            );
            assert_eq!(
                parse_size_string(&format!("{s}GB")).unwrap().get(),
                s * 1000000000
            );
            assert_eq!(
                parse_size_string(&format!("{s}TB")).unwrap().get(),
                s * 1000000000000
            );
            assert_eq!(
                parse_size_string(&format!("{s}PB")).unwrap().get(),
                s * 1000000000000000
            );
        }
    }
}
