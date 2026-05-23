use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn install() -> Result<()> {
    if has_systemd() {
        install_systemd().or_else(|_| install_xdg_autostart())
    } else {
        install_xdg_autostart()
    }
}

pub fn start() -> Result<()> {
    start_detached()?;
    println!("vcopy daemon started.");
    Ok(())
}

pub fn stop() -> Result<()> {
    stop_running();
    println!("vcopy daemon stopped.");
    Ok(())
}

pub fn restart() -> Result<()> {
    stop_running();
    start_detached()?;
    println!("vcopy daemon restarted.");
    Ok(())
}

pub fn uninstall() -> Result<()> {
    if has_systemd() {
        uninstall_systemd().or_else(|_| uninstall_xdg_autostart())
    } else {
        uninstall_xdg_autostart()
    }
}

pub fn status() -> Result<()> {
    let output = std::process::Command::new("/usr/bin/pgrep")
        .args(["-a", "-f", "[v]copy.*daemon"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let pids = String::from_utf8_lossy(&o.stdout);
            print!("vcopy daemon running — {}", pids);
        }
        _ => println!("vcopy daemon is not running"),
    }
    Ok(())
}

// ── systemd ───────────────────────────────────────────────────────────────────

fn install_systemd() -> Result<()> {
    let unit_path = systemd_service_path()?;
    std::fs::create_dir_all(unit_path.parent().unwrap())?;
    std::fs::write(&unit_path, systemd_unit())?;
    systemctl(&["daemon-reload"])?;
    systemctl(&["enable", "--now", "vcopy"])?;
    println!("vcopy daemon enabled via systemd.");
    Ok(())
}

fn uninstall_systemd() -> Result<()> {
    systemctl(&["disable", "--now", "vcopy"])?;
    let path = systemd_service_path()?;
    if path.exists() { std::fs::remove_file(&path)?; }
    systemctl(&["daemon-reload"])?;
    println!("vcopy daemon removed from systemd.");
    Ok(())
}

fn systemctl(args: &[&str]) -> Result<()> {
    let ok = std::process::Command::new("systemctl")
        .arg("--user")
        .args(args)
        .status()
        .context("systemctl not found")?
        .success();
    if !ok { anyhow::bail!("systemctl --user {} failed", args.join(" ")); }
    Ok(())
}

fn systemd_unit() -> String {
    format!(
        "[Unit]\n\
         Description=VCopy clipboard history daemon\n\
         After=graphical-session.target\n\
         PartOf=graphical-session.target\n\
         \n\
         [Service]\n\
         ExecStart={bin} daemon\n\
         Restart=on-failure\n\
         RestartSec=3\n\
         \n\
         [Install]\n\
         WantedBy=graphical-session.target\n",
        bin = bin_path().unwrap_or_default().display(),
    )
}

fn systemd_service_path() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("cannot determine config dir")?
        .join("systemd/user/vcopy.service"))
}

// ── XDG autostart (OpenRC / any DE) ──────────────────────────────────────────

fn install_xdg_autostart() -> Result<()> {
    let path = xdg_autostart_path()?;
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(&path, xdg_desktop_entry())?;
    println!("vcopy daemon added to XDG autostart (~/.config/autostart/vcopy.desktop).");
    println!("It will start automatically on your next login.");

    // Also start the daemon right now so the user doesn't need to log out.
    start_detached()?;
    println!("vcopy daemon started for the current session.");
    Ok(())
}

fn uninstall_xdg_autostart() -> Result<()> {
    let path = xdg_autostart_path()?;
    if path.exists() { std::fs::remove_file(&path)?; }

    // Kill running instances.
    stop_running();
    println!("vcopy daemon disabled and stopped.");
    Ok(())
}

fn xdg_desktop_entry() -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=VCopy\n\
         Comment=Clipboard history daemon\n\
         Exec={bin} daemon\n\
         Hidden=false\n\
         NoDisplay=true\n\
         X-GNOME-Autostart-enabled=true\n",
        bin = bin_path().unwrap_or_default().display(),
    )
}

fn xdg_autostart_path() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("cannot determine config dir")?
        .join("autostart/vcopy.desktop"))
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn has_systemd() -> bool {
    std::process::Command::new("systemctl")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn start_detached() -> Result<()> {
    let bin = bin_path()?;
    std::process::Command::new(&bin)
        .arg("daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("failed to start vcopy daemon")?;
    Ok(())
}

fn stop_running() {
    let _ = std::process::Command::new("/usr/bin/pkill")
        .args(["-f", "[v]copy.*daemon"])
        .status();
}

fn bin_path() -> Result<PathBuf> {
    std::env::current_exe().context("cannot determine vcopy binary path")
}
