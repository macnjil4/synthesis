use std::process::Command;

fn synthesis_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_synthesis"))
}

#[test]
fn help_flag_prints_usage() {
    let output = synthesis_bin()
        .arg("--help")
        .output()
        .expect("failed to run synthesis");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("A Rust audio synthesizer"));
    assert!(stdout.contains("--waveform"));
    assert!(stdout.contains("--frequency"));
    assert!(stdout.contains("--amplitude"));
    assert!(stdout.contains("--duration"));
    assert!(stdout.contains("--gui"));
}

#[test]
fn invalid_waveform_is_rejected() {
    let output = synthesis_bin()
        .args(["--waveform", "noise"])
        .output()
        .expect("failed to run synthesis");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid value"),
        "expected error about invalid value, got: {stderr}"
    );
}
