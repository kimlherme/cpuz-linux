use serde::Serialize;
use std::{collections::HashSet, fs, thread};

#[derive(Clone, Debug, Serialize)]
pub struct CpuInfo {
    pub model: String,
    pub architecture: String,
    pub logical_total: usize,
    pub logical_available: usize,
    pub physical_cores: Option<usize>,
}

pub fn detect() -> CpuInfo {
    let available = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    parse_cpuinfo(
        &fs::read_to_string("/proc/cpuinfo").unwrap_or_default(),
        available,
    )
}

pub fn parse_cpuinfo(text: &str, available: usize) -> CpuInfo {
    let mut model = None;
    let mut logical = 0;
    let mut cores = HashSet::new();
    for record in text.split("\n\n") {
        let mut socket = None;
        let mut core = None;
        for line in record.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "model name" | "Hardware" if model.is_none() && !value.is_empty() => {
                    model = Some(value.to_owned())
                }
                "processor" => logical += 1,
                "physical id" => socket = Some(value.to_owned()),
                "core id" => core = Some(value.to_owned()),
                _ => {}
            }
        }
        if let (Some(socket), Some(core)) = (socket, core) {
            cores.insert((socket, core));
        }
    }
    CpuInfo {
        model: model.unwrap_or_else(|| "CPU desconhecida".into()),
        architecture: std::env::consts::ARCH.into(),
        logical_total: logical.max(available),
        logical_available: available.max(1),
        physical_cores: (!cores.is_empty()).then_some(cores.len()),
    }
}
