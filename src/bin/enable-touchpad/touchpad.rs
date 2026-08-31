//! Touchpad detection, enable and disable via Windows PowerShell
//! (`Get-PnpDevice` / `Enable-PnpDevice` / `Disable-PnpDevice`).
//!
//! Demo-grade on purpose: it spawns `powershell.exe` instead of calling
//! `CfgMgr32` so that this crate needs no `unsafe` code. Device enable/disable
//! requires an elevated process; failure surfaces as `Err` and the UI shows
//! a hint. Device names are matched case-insensitively against both English
//! ("touchpad") and Chinese ("触摸板") friendly names.

use std::process::Command;

/// Friendly-name regex used on both sides of the toggle.
const MATCH: &str = "touchpad|触摸板";
/// Prefix making PowerShell output UTF-8 and non-interactive.
const PS_PREFIX: &str = "$ErrorActionPreference='SilentlyContinue';[Console]::OutputEncoding=[System.Text.Encoding]::UTF8;";

/// Snapshot of the detected touchpad device(s) and their aggregate state.
#[derive(Debug, Clone)]
pub struct TouchpadReport {
    /// Human-readable aggregate state (启用 / 已禁用 / …).
    pub status_text: String,
    /// Raw `Status|Problem|FriendlyName` rows, one per detected device.
    pub lines: Vec<String>,
}

/// PowerShell snippet listing touchpad `PnP` devices as `Status|Problem|Name`.
fn query_snippet() -> String {
    let mut s = String::from(PS_PREFIX);
    s.push_str("$d=Get-PnpDevice -PresentOnly|?{$_.FriendlyName -match '");
    s.push_str(MATCH);
    s.push_str("'}|%{$_.Status+'|'+$_.Problem+'|'+$_.FriendlyName};$d");
    s
}

/// Run a PowerShell script and return its UTF-8 stdout or the error text.
fn run_powershell(script: &str) -> Result<String, String> {
    let out = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| format!("无法启动 powershell: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let detail = stderr.trim();
        let detail = if detail.is_empty() {
            stdout.trim()
        } else {
            detail
        };
        let detail = if detail.is_empty() {
            "powershell 失败(可能需要以管理员身份运行)"
        } else {
            detail
        };
        return Err(detail.to_string());
    }
    Ok(stdout)
}

/// Parse `Status|Problem|Name` rows into an aggregate report.
pub fn parse_report(stdout: &str) -> TouchpadReport {
    let mut enabled = 0usize;
    let mut disabled = 0usize;
    let mut lines = Vec::new();
    for row in stdout.lines() {
        let row = row.trim();
        if row.is_empty() {
            continue;
        }
        let mut parts = row.splitn(3, '|');
        let status = parts.next().unwrap_or("").trim();
        let problem = parts.next().unwrap_or("").trim();
        if status.eq_ignore_ascii_case("OK") {
            enabled += 1;
        } else if status.eq_ignore_ascii_case("Error")
            && problem.eq_ignore_ascii_case("CM_PROB_DISABLED")
        {
            disabled += 1;
        }
        lines.push(row.to_string());
    }
    let status_text = match (enabled, disabled) {
        (0, 0) => "未检测到触摸板".to_string(),
        (0, _) => "已禁用".to_string(),
        (_, 0) => "启用".to_string(),
        _ => "部分启用/异常".to_string(),
    };
    TouchpadReport { status_text, lines }
}

/// Query the current touchpad state without changing anything.
pub fn query() -> Result<TouchpadReport, String> {
    let stdout = run_powershell(&query_snippet())?;
    Ok(parse_report(&stdout))
}

/// Enable (`true`) or disable (`false`) the detected touchpad device(s),
/// then re-query so the caller gets the post-toggle state.
pub fn set_enabled(enable: bool) -> Result<TouchpadReport, String> {
    let verb = if enable { "Enable" } else { "Disable" };
    let mut script = String::from(PS_PREFIX);
    script.push_str("$d=Get-PnpDevice -PresentOnly|?{$_.FriendlyName -match '");
    script.push_str(MATCH);
    script.push_str("'}|");
    script.push_str(verb);
    script.push_str("-PnpDevice -Confirm:$false;");
    script.push_str("Start-Sleep -Milliseconds 900;");
    script.push_str("$e=Get-PnpDevice -PresentOnly|?{$_.FriendlyName -match '");
    script.push_str(MATCH);
    script.push_str("'}|%{$_.Status+'|'+$_.Problem+'|'+$_.FriendlyName};$e");
    let stdout = run_powershell(&script)?;
    Ok(parse_report(&stdout))
}
