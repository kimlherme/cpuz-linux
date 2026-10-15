use crate::bench::Mode;
use serde::Serialize;

pub const MAX_THREADS: usize = 256;

#[derive(Clone, Debug, Serialize)]
pub struct Options {
    pub mode: Mode,
    pub seconds: f64,
    pub threads: usize,
    pub tui: bool,
    pub json: bool,
    pub help: bool,
}

pub fn parse(args: &[String], tty: bool) -> Result<Options, String> {
    let mut options = Options {
        mode: Mode::Single,
        seconds: 10.0,
        threads: std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1)
            .min(MAX_THREADS),
        tui: args.is_empty() && tty,
        json: false,
        help: false,
    };
    let mut force_tui = false;
    let mut force_headless = false;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => options.help = true,
            "--tui" => force_tui = true,
            "--no-tui" => force_headless = true,
            "--json" => {
                options.json = true;
                force_headless = true;
            }
            "--mode" => {
                options.mode = match args.next().map(String::as_str) {
                    Some("single") => Mode::Single,
                    Some("multi") => Mode::Multi,
                    Some("both") => Mode::Both,
                    _ => return Err("--mode must be single, multi or both".into()),
                }
            }
            "--seconds" => {
                options.seconds = args
                    .next()
                    .ok_or("missing value for --seconds")?
                    .parse()
                    .map_err(|_| "invalid duration")?
            }
            "--threads" => {
                let value = args.next().ok_or("missing value for --threads")?;
                if value == "auto" {
                    options.threads = std::thread::available_parallelism()
                        .map(usize::from)
                        .unwrap_or(1)
                        .min(MAX_THREADS);
                } else {
                    options.threads = value.parse().map_err(|_| "invalid thread count")?;
                }
            }
            _ => return Err(format!("unknown option: {arg}")),
        }
    }
    if options.help {
        return Ok(options);
    }
    if !options.seconds.is_finite() || !(0.05..=3600.0).contains(&options.seconds) {
        return Err("--seconds must be between 0.05 and 3600".into());
    }
    if !(1..=MAX_THREADS).contains(&options.threads) {
        return Err("--threads must be between 1 and 256".into());
    }
    if force_tui && force_headless {
        return Err("--tui cannot be combined with --json or --no-tui".into());
    }
    if force_tui && !tty {
        return Err("--tui requires stdin and stdout connected to a terminal".into());
    }
    options.tui = force_tui || (options.tui && !force_headless);
    Ok(options)
}

pub const HELP: &str = r#"CPUZ Linux 0.2.0 — experimental CPU benchmark

Usage: cpuz-linux [options]

No arguments in a terminal: open the TUI.
Without a terminal: run single for 10 seconds.

  --tui                 Open the interactive interface
  --no-tui              Use plain text output
  --mode single|multi|both
  --seconds N           Duration per phase (0.05..3600; default 10)
  --threads auto|N      Multi workers (1..256; default available CPUs)
  --json                JSON output; disables the TUI
  -h, --help            Show this help

TUI: S single · M multi · B both · +/- workers · [/] duration
     C cancel · Q/Esc quit · Ctrl+C cancel and quit

Score = counted pixels / elapsed integer milliseconds.
Based on the 17.01.SSE x64 workload; CPU-Z equivalence is not validated.
Use a release build. Not affiliated with CPUID.
"#;
