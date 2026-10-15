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
                    _ => return Err("--mode precisa ser single, multi ou both".into()),
                }
            }
            "--seconds" => {
                options.seconds = args
                    .next()
                    .ok_or("falta valor para --seconds")?
                    .parse()
                    .map_err(|_| "duração inválida")?
            }
            "--threads" => {
                let value = args.next().ok_or("falta valor para --threads")?;
                if value == "auto" {
                    options.threads = std::thread::available_parallelism()
                        .map(usize::from)
                        .unwrap_or(1)
                        .min(MAX_THREADS);
                } else {
                    options.threads = value.parse().map_err(|_| "número de threads inválido")?;
                }
            }
            _ => return Err(format!("opção desconhecida: {arg}")),
        }
    }
    if options.help {
        return Ok(options);
    }
    if !options.seconds.is_finite() || !(0.05..=3600.0).contains(&options.seconds) {
        return Err("--seconds precisa estar entre 0.05 e 3600".into());
    }
    if !(1..=MAX_THREADS).contains(&options.threads) {
        return Err("--threads precisa estar entre 1 e 256".into());
    }
    if force_tui && force_headless {
        return Err("--tui não pode ser combinado com --json ou --no-tui".into());
    }
    if force_tui && !tty {
        return Err("--tui precisa de stdin e stdout ligados a um terminal".into());
    }
    options.tui = force_tui || (options.tui && !force_headless);
    Ok(options)
}

pub const HELP: &str = "CPUZ Linux 0.2.0 — benchmark experimental de CPU\n\nUso: cpuz-linux [opções]\n\nSem argumentos em um terminal: abre a TUI.\nSem terminal: executa single por 10 segundos.\n\n  --tui                 Abre a interface interativa\n  --no-tui              Usa saída de texto\n  --mode single|multi|both\n  --seconds N           Duração por fase (0.05..3600; padrão 10)\n  --threads auto|N      Workers no multi (1..256; padrão CPUs disponíveis)\n  --json                Saída JSON; implica execução sem TUI\n  -h, --help            Mostra esta ajuda\n\nTUI: S single · M multi · B ambos · +/- threads · [/] duração\n     C cancela · Q/Esc sai · Ctrl+C cancela e sai\n\nScore = pixels contabilizados / milissegundos inteiros.\nCarga baseada no 17.01.SSE x64; equivalência com CPU-Z não validada.\nUse compilação release. Não é afiliado à CPUID.\n";
