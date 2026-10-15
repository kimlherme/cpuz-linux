use crate::{
    bench::{self, Mode, Progress, RunResult},
    cli::{MAX_THREADS, Options},
    cpu::CpuInfo,
};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Gauge, Paragraph, Wrap},
};
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

const ACCENT: Color = Color::Rgb(95, 213, 190);
const GOLD: Color = Color::Rgb(238, 190, 113);
const DIM: Color = Color::Rgb(147, 160, 177);
const BG: Color = Color::Rgb(17, 24, 35);

struct ActiveRun {
    cancel: Arc<AtomicBool>,
    progress: Arc<Progress>,
    receiver: mpsc::Receiver<Result<Vec<RunResult>, String>>,
    handle: Option<JoinHandle<()>>,
}

impl ActiveRun {
    fn start(mode: Mode, options: &Options) -> io::Result<Self> {
        let cancel = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(Progress::default());
        let (tx, receiver) = mpsc::channel();
        let worker_cancel = cancel.clone();
        let worker_progress = progress.clone();
        let seconds = options.seconds;
        let threads = options.threads;
        let handle = thread::Builder::new()
            .name("cpuz-coordinator".into())
            .spawn(move || {
                let result = bench::run(
                    mode,
                    threads,
                    Duration::from_secs_f64(seconds),
                    worker_cancel,
                    worker_progress,
                );
                let _ = tx.send(result);
            })?;
        Ok(Self {
            cancel,
            progress,
            receiver,
            handle: Some(handle),
        })
    }
}

impl Drop for ActiveRun {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

struct Screen {
    options: Options,
    cpu: CpuInfo,
    single: Option<RunResult>,
    multi: Option<RunResult>,
    active: Option<ActiveRun>,
    status: String,
}

impl Screen {
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(Block::new().style(Style::default().bg(BG)), frame.area());
        if frame.area().width < 64 || frame.area().height < 22 {
            frame.render_widget(Paragraph::new("CPUZ Linux\n\nAmplie o terminal para pelo menos 64 × 22.\nQ / Esc: sair · C: cancelar").style(Style::default().fg(ACCENT)).wrap(Wrap { trim: true }), frame.area());
            return;
        }
        let area = frame.area().inner(Margin {
            horizontal: 2,
            vertical: 1,
        });
        let rows = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Min(4),
            Constraint::Length(4),
        ])
        .split(area);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(
                        " CPUZ ",
                        Style::default()
                            .fg(BG)
                            .bg(ACCENT)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " LINUX",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("   /   laboratório de desempenho", Style::default().fg(DIM)),
                ]),
                Line::from(Span::styled(
                    " 17.01-style · f32 escalar · pontuação experimental",
                    Style::default().fg(DIM),
                )),
            ]),
            rows[0],
        );
        let physical = self
            .cpu
            .physical_cores
            .map_or_else(|| "—".into(), |v| v.to_string());
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    &self.cpu.model,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(format!(
                    "{}  ·  {} CPUs disponíveis / {} detectadas  ·  {} núcleos físicos",
                    self.cpu.architecture,
                    self.cpu.logical_available,
                    self.cpu.logical_total,
                    physical
                )),
            ])
            .style(Style::default().fg(DIM))
            .block(panel(" PROCESSADOR "))
            .wrap(Wrap { trim: true }),
            rows[1],
        );
        let cards = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .spacing(1)
            .split(rows[2]);
        self.result_card(frame, cards[0], Mode::Single, &self.single);
        self.result_card(frame, cards[1], Mode::Multi, &self.multi);
        let ratio = self
            .single
            .as_ref()
            .zip(self.multi.as_ref())
            .filter(|(s, m)| !s.cancelled && !m.cancelled && s.score > 0.0)
            .map(|(s, m)| format!("{:.2}×", m.score / s.score))
            .unwrap_or_else(|| "—".into());
        let middle = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(rows[3]);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(
                        format!(" {} s/fase", self.options.seconds),
                        Style::default().fg(GOLD),
                    ),
                    Span::raw("   ·   "),
                    Span::styled(
                        format!("{} workers multi", self.options.threads),
                        Style::default().fg(ACCENT),
                    ),
                    Span::raw(format!("   ·   razão {ratio}")),
                ]),
                Line::from(Span::styled(&self.status, Style::default().fg(DIM))),
            ])
            .style(Style::default().fg(Color::White)),
            middle[0],
        );
        let (fraction, label) = if let Some(active) = &self.active {
            let p = &active.progress;
            let started = *p.started.lock().unwrap_or_else(|e| e.into_inner());
            let seconds = p.duration_ms.load(Ordering::Relaxed) as f64 / 1000.0;
            let elapsed = started.map_or(0.0, |s| s.elapsed().as_secs_f64());
            let pixels = p.pixels.load(Ordering::Relaxed);
            let phase = if p.phase.load(Ordering::Acquire) == 1 {
                "single"
            } else {
                "multi"
            };
            (
                (elapsed / seconds.max(0.001)).clamp(0.0, 1.0),
                format!("{phase}  {elapsed:.1}/{seconds:.1}s  ·  {pixels} pixels"),
            )
        } else {
            (0.0, "Pronto para medir".into())
        };
        frame.render_widget(
            Gauge::default()
                .ratio(fraction)
                .label(label)
                .gauge_style(Style::default().fg(ACCENT).bg(Color::Rgb(35, 47, 62))),
            middle[1],
        );
        frame.render_widget(Paragraph::new("S single   M multi   B ambos   +/− workers   [/] segundos\nC cancelar   Q / Esc sair   ·   resultados variam com frequência e carga").style(Style::default().fg(DIM)).block(panel(" CONTROLES ")), rows[4]);
    }

    fn result_card(
        &self,
        frame: &mut Frame,
        area: ratatui::layout::Rect,
        mode: Mode,
        result: &Option<RunResult>,
    ) {
        let accent = if mode == Mode::Single { ACCENT } else { GOLD };
        let (score, details) = match result {
            Some(r) => (
                format!("{:.1}", r.score),
                format!(
                    "{} threads · {} ms{}",
                    r.threads,
                    r.elapsed_ms,
                    if r.cancelled { " · parcial" } else { "" }
                ),
            ),
            None => ("—".into(), "Aguardando execução".into()),
        };
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("  {score}"),
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    format!("  {details}"),
                    Style::default().fg(DIM),
                )),
                Line::from(Span::styled(
                    "  pixels / milissegundo",
                    Style::default().fg(DIM),
                )),
            ])
            .block(panel(if mode == Mode::Single {
                " SINGLE THREAD "
            } else {
                " MULTI THREAD "
            })),
            area,
        );
    }
}

fn panel(title: &'static str) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(58, 75, 92)))
        .title(Span::styled(title, Style::default().fg(ACCENT)))
}

pub fn run(options: Options, cpu: CpuInfo) -> io::Result<()> {
    // Ratatui installs a restoring panic hook; the closure restores on errors.
    ratatui::run(|terminal| {
        let mut screen = Screen {
            options,
            cpu,
            single: None,
            multi: None,
            active: None,
            status: "Selecione S, M ou B para iniciar.".into(),
        };
        loop {
            if let Some(active) = &screen.active {
                match active.receiver.try_recv() {
                    Ok(Ok(results)) => {
                        for result in results {
                            if result.mode == Mode::Single {
                                screen.single = Some(result);
                            } else {
                                screen.multi = Some(result);
                            }
                        }
                        screen.status = if active.cancel.load(Ordering::Relaxed) {
                            "Cancelado; resultado parcial identificado.".into()
                        } else {
                            "Medição concluída. Execute novamente para comparar.".into()
                        };
                        screen.active = None;
                    }
                    Ok(Err(error)) => {
                        screen.status = error;
                        screen.active = None;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        screen.status = "Coordenador encerrou sem resultado.".into();
                        screen.active = None;
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                }
            }
            terminal.draw(|frame| screen.draw(frame))?;
            if !event::poll(Duration::from_millis(100))? {
                continue;
            }
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('q' | 'Q'))
                || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
            {
                break;
            }
            if matches!(key.code, KeyCode::Char('c' | 'C')) {
                if let Some(active) = &screen.active {
                    active.cancel.store(true, Ordering::Relaxed);
                    screen.status = "Cancelando após a unidade de trabalho atual…".into();
                }
                continue;
            }
            if screen.active.is_some() {
                continue;
            }
            let mode = match key.code {
                KeyCode::Char('s' | 'S') => Some(Mode::Single),
                KeyCode::Char('m' | 'M') => Some(Mode::Multi),
                KeyCode::Char('b' | 'B') => Some(Mode::Both),
                KeyCode::Char('+') => {
                    screen.options.threads = (screen.options.threads + 1).min(MAX_THREADS);
                    None
                }
                KeyCode::Char('-') => {
                    screen.options.threads = screen.options.threads.saturating_sub(1).max(1);
                    None
                }
                KeyCode::Char(']') => {
                    screen.options.seconds = (screen.options.seconds + 1.0).min(3600.0);
                    None
                }
                KeyCode::Char('[') => {
                    screen.options.seconds = (screen.options.seconds - 1.0).max(0.05);
                    None
                }
                _ => None,
            };
            if let Some(mode) = mode {
                screen.active = Some(ActiveRun::start(mode, &screen.options)?);
                screen.status = format!(
                    "Preparando {} · a medição começa após inicializar os workers.",
                    mode.label()
                );
            }
        }
        // Drop cancels and joins before the terminal is restored.
        drop(screen);
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_shows_cpu_and_survives_small_terminal() {
        let options = crate::cli::parse(&[], true).unwrap();
        let screen = Screen {
            options,
            cpu: crate::cpu::parse_cpuinfo("model name : Test CPU", 2),
            single: None,
            multi: None,
            active: None,
            status: "Pronto".into(),
        };
        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 26)).unwrap();
        terminal.draw(|f| screen.draw(f)).unwrap();
        let text = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(text.contains("Test CPU"));
        assert!(text.contains("SINGLE THREAD"));
        assert!(text.contains("MULTI THREAD"));
        terminal
            .resize(ratatui::layout::Rect::new(0, 0, 30, 10))
            .unwrap();
        terminal.draw(|f| screen.draw(f)).unwrap();
        if let Ok(path) = std::env::var("CPUZ_PREVIEW_PATH") {
            let preview = Screen {
                cpu: crate::cpu::detect(),
                ..screen
            };
            terminal
                .resize(ratatui::layout::Rect::new(0, 0, 100, 28))
                .unwrap();
            terminal.draw(|f| preview.draw(f)).unwrap();
            save_svg(terminal.backend().buffer(), &path);
        }
    }

    // Opt-in documentation artifact from the SAME rendering path as the TUI.
    fn save_svg(buffer: &ratatui::buffer::Buffer, path: &str) {
        use std::fmt::Write;
        let color = |color: Color| -> String {
            match color {
                Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
                Color::White => "#ffffff".into(),
                Color::Black => "#000000".into(),
                _ => "#93a0b1".into(),
            }
        };
        let width = u32::from(buffer.area.width) * 9 + 32;
        let height = u32::from(buffer.area.height) * 20 + 32;
        let mut svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\"><title>CPUZ Linux: renderização da TUI em repouso</title><rect width=\"100%\" height=\"100%\" rx=\"12\" fill=\"#111823\"/><g font-family=\"DejaVu Sans Mono,monospace\" font-size=\"14\">"
        );
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = &buffer[(x, y)];
                let px = u32::from(x) * 9 + 16;
                let py = u32::from(y) * 20 + 16;
                if cell.bg != Color::Reset && cell.bg != BG {
                    write!(
                        svg,
                        "<rect x=\"{px}\" y=\"{py}\" width=\"9\" height=\"20\" fill=\"{}\"/>",
                        color(cell.bg)
                    )
                    .unwrap();
                }
                if cell.symbol() != " " {
                    let symbol = cell
                        .symbol()
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;");
                    write!(
                        svg,
                        "<text x=\"{px}\" y=\"{}\" fill=\"{}\">{symbol}</text>",
                        py + 15,
                        color(cell.fg)
                    )
                    .unwrap();
                }
            }
        }
        svg.push_str("</g></svg>\n");
        std::fs::write(path, svg).unwrap();
    }
}
