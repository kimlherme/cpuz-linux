use cpuz_linux::{
    bench::{self, Progress},
    cli, cpu, tui,
};
use serde::Serialize;
use std::{
    io::{self, IsTerminal},
    process::ExitCode,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

#[derive(Serialize)]
struct Report {
    schema_version: u32,
    algorithm: &'static str,
    experimental: bool,
    cpu: cpu::CpuInfo,
    configuration: cli::Options,
    results: Vec<bench::RunResult>,
    multi_single_ratio: Option<f32>,
}

fn execute() -> Result<(), String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let options = cli::parse(
        &args,
        io::stdin().is_terminal() && io::stdout().is_terminal(),
    )?;
    if options.help {
        print!("{}", cli::HELP);
        return Ok(());
    }
    let cpu = cpu::detect();
    if options.tui {
        return tui::run(options, cpu).map_err(|e| e.to_string());
    }
    if !options.json {
        println!(
            "CPUZ Linux · experimental 17.01-style\nCPU: {}\nCPUs disponíveis: {} / {}\n",
            cpu.model, cpu.logical_available, cpu.logical_total
        );
        if cfg!(debug_assertions) {
            eprintln!("Compilação debug: use --release para medir desempenho.");
        }
    }
    let results = bench::run(
        options.mode,
        options.threads,
        Duration::from_secs_f64(options.seconds),
        Arc::new(AtomicBool::new(false)),
        Arc::new(Progress::default()),
    )?;
    let ratio = bench::ratio(&results);
    if options.json {
        let report = Report {
            schema_version: 1,
            algorithm: "17.01-style-scalar-f32",
            experimental: true,
            cpu,
            configuration: options,
            results,
            multi_single_ratio: ratio,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
        );
    } else {
        for result in &results {
            println!(
                "{}: {:.1}\nThreads: {}\nPixels contados: {}\nTempo: {} ms\nChecksum: {}\n",
                result.mode.label(),
                result.score,
                result.threads,
                result.pixels,
                result.elapsed_ms,
                result.checksum
            );
        }
        if let Some(ratio) = ratio {
            println!("Multi / single: {ratio:.2}x");
        }
        println!("Score experimental; equivalência com CPU-Z não validada.");
    }
    Ok(())
}

fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Erro: {error}\nUse --help para consultar as opções.");
            ExitCode::from(2)
        }
    }
}
