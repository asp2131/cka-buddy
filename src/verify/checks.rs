use std::process::Command;

use anyhow::Result;

use crate::content::models::Step;

pub enum VerifyOutcome {
    Pass,
    Fail(String),
    NoChecks,
}

pub fn run_verify(step: &Step) -> Result<VerifyOutcome> {
    let checks = if step.success_check_commands.is_empty() {
        &step.verify_commands
    } else {
        &step.success_check_commands
    };

    if checks.is_empty() {
        return Ok(VerifyOutcome::NoChecks);
    }

    let mut collected_stdout = String::new();
    for cmd in checks {
        let output = Command::new("/bin/zsh").arg("-lc").arg(cmd).output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let msg = if stderr.trim().is_empty() {
                format!("verify command failed: {}", cmd)
            } else {
                format!("{} ({})", cmd, stderr.trim())
            };
            return Ok(VerifyOutcome::Fail(msg));
        }
        collected_stdout.push_str(&String::from_utf8_lossy(&output.stdout));
        collected_stdout.push('\n');
    }

    for expected in &step.success_contains {
        if !collected_stdout.contains(expected) {
            return Ok(VerifyOutcome::Fail(format!(
                "missing expected output: {}",
                expected
            )));
        }
    }

    Ok(VerifyOutcome::Pass)
}
