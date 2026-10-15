use std::process::Command;

#[test]
fn json_reports_both_phases_and_named_cpu() {
    let output = Command::new(env!("CARGO_BIN_EXE_cpuz-linux"))
        .args([
            "--mode",
            "both",
            "--threads",
            "2",
            "--seconds",
            "0.1",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert!(report["cpu"]["model"].as_str().unwrap().len() > 1);
    let results = report["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["mode"], "multi");
    assert_eq!(results[0]["threads"], 2);
    assert_eq!(results[1]["mode"], "single");
    assert_eq!(results[1]["threads"], 1);
    for result in results {
        let pixels = result["pixels"].as_u64().unwrap();
        let ms = result["elapsed_ms"].as_u64().unwrap();
        assert!(pixels > 0);
        assert!(
            (result["score"].as_f64().unwrap() - f64::from(pixels as f32 / ms as f32)).abs()
                < 0.001
        );
    }
    assert!(report["multi_single_ratio"].as_f64().unwrap() > 0.0);
}

#[test]
fn tui_in_pipe_fails_cleanly_without_escape_sequences() {
    let output = Command::new(env!("CARGO_BIN_EXE_cpuz-linux"))
        .arg("--tui")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.contains(&0x1b));
}
