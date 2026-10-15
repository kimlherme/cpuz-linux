use crate::noise::{Noise, SIDE};
use serde::Serialize;
use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Single,
    Multi,
    Both,
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Single => "Single",
            Self::Multi => "Multi",
            Self::Both => "Ambos",
        }
    }
}

#[derive(Default)]
pub struct Progress {
    pub pixels: AtomicU64,
    pub workers: AtomicUsize,
    pub phase: AtomicUsize,
    pub duration_ms: AtomicU64,
    pub started: Mutex<Option<Instant>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunResult {
    pub mode: Mode,
    pub threads: usize,
    pub pixels: u64,
    pub elapsed_ms: u64,
    pub score: f32,
    pub checksum: String,
    pub worker_pixels: Vec<u64>,
    pub cancelled: bool,
}

struct Gate {
    start: Option<Instant>,
    aborted: bool,
}
struct WorkerResult {
    pixels: u64,
    finished: Instant,
    checksum: u64,
}

struct CancelOnPanic(Arc<AtomicBool>);
impl Drop for CancelOnPanic {
    fn drop(&mut self) {
        if thread::panicking() {
            self.0.store(true, Ordering::Relaxed);
        }
    }
}

pub fn run_phase(
    mode: Mode,
    threads: usize,
    duration: Duration,
    cancel: Arc<AtomicBool>,
    progress: Arc<Progress>,
) -> Result<RunResult, String> {
    if mode == Mode::Both {
        return Err("run_phase aceita somente single ou multi".into());
    }
    let threads = if mode == Mode::Single { 1 } else { threads };
    if threads == 0
        || threads > crate::cli::MAX_THREADS
        || duration < Duration::from_millis(50)
        || duration > Duration::from_secs(3600)
    {
        return Err("configuração de benchmark fora dos limites".into());
    }
    let noise = Arc::new(Noise::new());
    let gate = Arc::new((
        Mutex::new(Gate {
            start: None,
            aborted: false,
        }),
        Condvar::new(),
    ));
    let (ready_tx, ready_rx) = mpsc::channel();
    let mut handles = Vec::with_capacity(threads);
    let mut startup_error = None;
    progress.pixels.store(0, Ordering::Relaxed);
    progress.workers.store(threads, Ordering::Relaxed);
    progress
        .phase
        .store(if mode == Mode::Single { 1 } else { 2 }, Ordering::Release);
    progress
        .duration_ms
        .store(duration.as_millis() as u64, Ordering::Relaxed);
    *progress.started.lock().unwrap_or_else(|e| e.into_inner()) = None;
    for worker in 0..threads {
        let noise = noise.clone();
        let gate = gate.clone();
        let ready = ready_tx.clone();
        let cancel = cancel.clone();
        let progress = progress.clone();
        let spawned = thread::Builder::new()
            .name(format!("cpuz-worker-{worker}"))
            .spawn(move || {
                let _panic_guard = CancelOnPanic(cancel.clone());
                // Allocate and fault pages before the common measured start.
                let mut buffer = vec![0_u8; SIDE * SIDE * 4];
                for byte in &mut buffer {
                    *byte = std::hint::black_box(0);
                }
                let _ = ready.send(());
                drop(ready);
                let (lock, cv) = &*gate;
                let guard = lock.lock().unwrap_or_else(|e| e.into_inner());
                let guard = cv
                    .wait_while(guard, |g| g.start.is_none() && !g.aborted)
                    .unwrap_or_else(|e| e.into_inner());
                if guard.aborted {
                    return WorkerResult {
                        pixels: 0,
                        finished: Instant::now(),
                        checksum: 0,
                    };
                }
                let start = guard.start.expect("gate released without a start");
                drop(guard);
                let mut job = worker as u64;
                let mut pixels = 0;
                while !cancel.load(Ordering::Relaxed) && start.elapsed() < duration {
                    let row = (job % SIDE as u64) as usize;
                    let frame = (job / SIDE as u64) as i32;
                    noise.row(
                        std::hint::black_box(&mut buffer[row * SIDE * 4..(row + 1) * SIDE * 4]),
                        std::hint::black_box(frame),
                        row as i32,
                    );
                    if start.elapsed() >= duration || cancel.load(Ordering::Relaxed) {
                        break;
                    }
                    pixels += SIDE as u64;
                    progress.pixels.fetch_add(SIDE as u64, Ordering::Relaxed);
                    job += threads as u64;
                }
                let finished = Instant::now();
                // Hashing and thread joins MUST NOT extend the measured interval.
                let checksum = std::hint::black_box(&buffer)
                    .iter()
                    .fold(0xcbf29ce484222325_u64, |hash, byte| {
                        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
                    });
                WorkerResult {
                    pixels,
                    finished,
                    checksum,
                }
            });
        match spawned {
            Ok(handle) => handles.push(handle),
            Err(error) => {
                startup_error = Some(format!("falha ao criar worker {worker}: {error}"));
                break;
            }
        }
    }
    drop(ready_tx);
    if startup_error.is_none() {
        for _ in 0..threads {
            if ready_rx.recv().is_err() {
                startup_error = Some("worker encerrou durante inicialização".into());
                break;
            }
        }
    }
    let start = Instant::now();
    {
        let mut state = gate.0.lock().unwrap_or_else(|e| e.into_inner());
        if startup_error.is_some() {
            state.aborted = true;
        } else {
            *progress.started.lock().unwrap_or_else(|e| e.into_inner()) = Some(start);
            state.start = Some(start);
        }
    }
    gate.1.notify_all();
    let mut results = Vec::with_capacity(threads);
    for handle in handles {
        match handle.join() {
            Ok(result) => results.push(result),
            Err(_) => {
                startup_error.get_or_insert("worker entrou em panic".into());
                cancel.store(true, Ordering::Relaxed);
            }
        }
    }
    if let Some(error) = startup_error {
        return Err(error);
    }
    let elapsed_ms = results
        .iter()
        .map(|r| r.finished.saturating_duration_since(start).as_millis() as u64)
        .max()
        .unwrap_or(0)
        .max(1);
    let pixels = results.iter().map(|r| r.pixels).sum::<u64>();
    let checksum = results.iter().enumerate().fold(0_u64, |acc, (id, r)| {
        acc.rotate_left(7) ^ r.checksum.wrapping_add(id as u64)
    });
    Ok(RunResult {
        mode,
        threads,
        pixels,
        elapsed_ms,
        score: pixels as f32 / elapsed_ms as f32,
        checksum: format!("{checksum:016x}"),
        worker_pixels: results.iter().map(|r| r.pixels).collect(),
        cancelled: cancel.load(Ordering::Relaxed),
    })
}

pub fn run(
    mode: Mode,
    threads: usize,
    duration: Duration,
    cancel: Arc<AtomicBool>,
    progress: Arc<Progress>,
) -> Result<Vec<RunResult>, String> {
    let phases: &[Mode] = match mode {
        Mode::Both => &[Mode::Multi, Mode::Single],
        Mode::Single => &[Mode::Single],
        Mode::Multi => &[Mode::Multi],
    };
    let mut results = Vec::new();
    for phase in phases {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        results.push(run_phase(
            *phase,
            threads,
            duration,
            cancel.clone(),
            progress.clone(),
        )?);
    }
    Ok(results)
}

pub fn ratio(results: &[RunResult]) -> Option<f32> {
    let single = results
        .iter()
        .find(|r| r.mode == Mode::Single && !r.cancelled)?
        .score;
    let multi = results
        .iter()
        .find(|r| r.mode == Mode::Multi && !r.cancelled)?
        .score;
    (single > 0.0).then_some(multi / single)
}
