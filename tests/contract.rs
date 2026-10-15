use cpuz_linux::{
    bench::{self, Mode, Progress},
    cli, cpu,
};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

#[test]
fn cli_is_bounded_and_explicit_modes_are_headless() {
    for args in [
        vec!["--seconds", "0"],
        vec!["--seconds", "NaN"],
        vec!["--threads", "0"],
        vec!["--threads", "257"],
        vec!["--mode", "bad"],
    ] {
        let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
        assert!(cli::parse(&args, true).is_err());
    }
    let options = cli::parse(&["--mode".into(), "multi".into()], true).unwrap();
    assert_eq!(options.mode, Mode::Multi);
    assert!(!options.tui);
    assert!(cli::parse(&[], true).unwrap().tui);
    assert!(!cli::parse(&[], false).unwrap().tui);
    let automatic = cli::parse(&[], false).unwrap().threads;
    assert_eq!(
        cli::parse(
            &[
                "--threads".into(),
                "4".into(),
                "--threads".into(),
                "auto".into()
            ],
            false
        )
        .unwrap()
        .threads,
        automatic
    );
}

#[test]
fn cpu_parser_handles_missing_and_duplicate_records() {
    let text = "processor : 0\nmodel name : Example CPU\nphysical id : 0\ncore id : 0\n\nprocessor : 1\nmodel name : Example CPU\nphysical id : 0\ncore id : 0\n";
    let cpu = cpu::parse_cpuinfo(text, 1);
    assert_eq!(cpu.model, "Example CPU");
    assert_eq!(cpu.logical_total, 2);
    assert_eq!(cpu.logical_available, 1);
    assert_eq!(cpu.physical_cores, Some(1));
    assert_eq!(cpu::parse_cpuinfo("", 1).model, "Unknown CPU");
}

#[test]
fn multi_counts_real_workers_and_uses_common_elapsed_time() {
    let progress = Arc::new(Progress::default());
    let cancel = Arc::new(AtomicBool::new(false));
    let run =
        bench::run_phase(Mode::Multi, 2, Duration::from_millis(150), cancel, progress).unwrap();
    assert_eq!(run.threads, 2);
    assert_eq!(run.worker_pixels.len(), 2);
    assert!(run.worker_pixels.iter().all(|p| *p > 0));
    assert_eq!(run.pixels, run.worker_pixels.iter().sum::<u64>());
    assert_eq!(run.pixels % 1024, 0);
    assert_eq!(run.score, run.pixels as f32 / run.elapsed_ms.max(1) as f32);
}

#[test]
fn cancellation_does_not_wait_for_requested_duration() {
    let cancel = Arc::new(AtomicBool::new(false));
    let worker_cancel = cancel.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = bench::run_phase(
            Mode::Multi,
            2,
            Duration::from_secs(30),
            worker_cancel,
            Arc::new(Progress::default()),
        );
        tx.send(result).unwrap();
    });
    std::thread::sleep(Duration::from_millis(50));
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    let run = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("cancellation hung")
        .unwrap();
    assert!(run.cancelled);
}
