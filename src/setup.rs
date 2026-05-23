use crate::config::{Config, ShortcutConfig, config_path};
use crate::storage::Storage;
use anyhow::{Context, Result};
use std::io::{self, Write};
use std::process::Command;

const TERMINALS: &[&str] = &[
    "kgx", "kitty", "alacritty", "foot", "wezterm",
    "gnome-terminal", "konsole", "xfce4-terminal", "xterm",
];

/// Interactive config wizard for shortcut + terminal size.
pub fn run_wizard() -> Result<()> {
    let mut cfg = Config::load()?;

    println!("\n\x1b[1;32mVCopy\x1b[0m — shortcut config\n");

    // ── terminal ─────────────────────────────────────────────────────────────
    let available: Vec<&str> = TERMINALS.iter().copied().filter(|t| which(t)).collect();

    println!("Available terminals:");
    for (i, t) in available.iter().enumerate() {
        let marker = if *t == cfg.shortcut.terminal { " \x1b[32m← current\x1b[0m" } else { "" };
        println!("  {}) {}{}", i + 1, t, marker);
    }
    print!("Terminal [Enter = keep '{}']: ", cfg.shortcut.terminal);
    io::stdout().flush()?;
    let input = readline()?;
    if !input.is_empty() {
        if let Ok(n) = input.parse::<usize>() {
            if n >= 1 && n <= available.len() {
                cfg.shortcut.terminal = available[n - 1].to_string();
            }
        } else {
            cfg.shortcut.terminal = input;
        }
    }

    // ── shortcut key ─────────────────────────────────────────────────────────
    print!("Shortcut key [Enter = keep '{}']: ", cfg.shortcut.key);
    io::stdout().flush()?;
    let input = readline()?;
    if !input.is_empty() {
        cfg.shortcut.key = input;
    }

    // ── terminal size ─────────────────────────────────────────────────────────
    print!("Window width  [Enter = keep {}]: ", cfg.shortcut.width);
    io::stdout().flush()?;
    let input = readline()?;
    if let Ok(n) = input.parse::<u16>() { cfg.shortcut.width = n; }

    print!("Window height [Enter = keep {}]: ", cfg.shortcut.height);
    io::stdout().flush()?;
    let input = readline()?;
    if let Ok(n) = input.parse::<u16>() { cfg.shortcut.height = n; }

    // ── history ───────────────────────────────────────────────────────────────
    print!("History limit [Enter = keep {}]: ", cfg.history.limit);
    io::stdout().flush()?;
    let input = readline()?;
    if let Ok(n) = input.parse::<usize>() {
        if n > 0 {
            cfg.history.limit = n;
        }
    }

    print!(
        "Clear history on restart? [y/N, Enter = keep {}]: ",
        if cfg.history.clear_on_restart { "yes" } else { "no" }
    );
    io::stdout().flush()?;
    let input = readline()?.to_lowercase();
    match input.as_str() {
        "y" | "yes" | "s" | "sim" => cfg.history.clear_on_restart = true,
        "n" | "no" | "nao" | "não" => cfg.history.clear_on_restart = false,
        "" => {}
        _ => {}
    }

    cfg.save()?;
    Storage::new()?.prune(cfg.history.effective_limit())?;
    println!("\n\x1b[32mSaved\x1b[0m → {}\n", config_path().display());

    // ── generate launch script ────────────────────────────────────────────────
    let script_path = generate_popup_script(&cfg.shortcut)?;
    println!("\x1b[1mLaunch script:\x1b[0m {}", script_path.display());

    // ── WM instructions ───────────────────────────────────────────────────────
    print_wm_instructions(&cfg.shortcut, &script_path);

    Ok(())
}

/// Generates `~/.local/bin/vcopy-popup` with the correct terminal + size flags.
fn generate_popup_script(s: &ShortcutConfig) -> Result<std::path::PathBuf> {
    let bin = std::env::current_exe()?;
    let cmd = terminal_launch_cmd(&s.terminal, s.width, s.height, &bin.display().to_string());

    let dir = dirs::home_dir().unwrap_or_default().join(".local/bin");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("vcopy-popup");

    std::fs::write(&path, format!("#!/bin/sh\nexec {cmd}\n"))?;

    // make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    }

    Ok(path)
}

fn terminal_launch_cmd(terminal: &str, w: u16, h: u16, bin: &str) -> String {
    match terminal {
        "kgx" | "console" | "gnome-console" => format!(
            "kgx --gapplication-app-id org.gnome.Console.VCopy --title vcopy -e sh -c '{bin}; status=$?; kill -TERM \"$PPID\" 2>/dev/null; exit \"$status\"'"
        ),
        "kitty" => format!(
            "kitty --class vcopy --title vcopy \
             --override initial_window_width={w}c \
             --override initial_window_height={h}c {bin}"
        ),
        "alacritty" =>
            format!("alacritty --option window.dimensions.columns={w} --option window.dimensions.lines={h} -e {bin}"),
        "foot" =>
            format!("foot --window-size-chars={w}x{h} {bin}"),
        "wezterm" =>
            format!("wezterm start --no-auto-connect -- {bin}"),
        "gnome-terminal" =>
            format!("gnome-terminal --geometry={w}x{h} -- {bin}"),
        "konsole" =>
            format!("konsole --geometry {w}x{h} -e {bin}"),
        "xfce4-terminal" =>
            format!("xfce4-terminal --geometry={w}x{h} -e {bin}"),
        _ =>
            format!("{terminal} -geometry {w}x{h} -e {bin}"),
    }
}

fn print_wm_instructions(s: &ShortcutConfig, script: &std::path::Path) {
    let wm = detect_wm();
    let key = wm_key_format(&s.key, wm);
    let script_str = script.display().to_string();

    println!("\n\x1b[1mWindow manager:\x1b[0m {wm}");

    match wm {
        "sway" | "i3" => {
            let cfg_path = if wm == "sway" {
                "~/.config/sway/config"
            } else {
                "~/.config/i3/config"
            };

            let line = format!("bindsym {key} exec --no-startup-id {script_str}");

            match try_add_to_wm_config(wm, &line) {
                Ok(path) => println!("\x1b[32mAdded to\x1b[0m {}", path.display()),
                Err(_) => {
                    println!("Add to \x1b[1m{cfg_path}\x1b[0m:");
                    println!("\n  \x1b[36m{line}\x1b[0m\n");
                    println!("Then reload: \x1b[1msway reload\x1b[0m / \x1b[1mi3-msg reload\x1b[0m");
                }
            }
        }
        "gnome" => match configure_gnome_shortcut(s, script) {
            Ok(()) => println!("\x1b[32mRegistered GNOME shortcut\x1b[0m {key} → {script_str}"),
            Err(err) => {
                println!("Could not register GNOME shortcut automatically: {err}");
                println!("Run in Settings → Keyboard → Custom Shortcuts:");
                println!("  Name:    \x1b[36mVCopy\x1b[0m");
                println!("  Command: \x1b[36m{script_str}\x1b[0m");
                println!("  Key:     \x1b[36m{}\x1b[0m", s.key);
            }
        },
        "kde" => {
            println!("System Settings → Shortcuts → Custom Shortcuts:");
            println!("  Command: \x1b[36m{script_str}\x1b[0m");
            println!("  Key:     \x1b[36m{}\x1b[0m", s.key);
        }
        _ => {
            println!("Bind \x1b[1m{}\x1b[0m → \x1b[36m{script_str}\x1b[0m in your WM config.", s.key);
        }
    }
}

fn configure_gnome_shortcut(s: &ShortcutConfig, script: &std::path::Path) -> Result<()> {
    let binding = gnome_key_format(&s.key);
    let script_str = script.display().to_string();
    let key_path = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/vcopy/";
    let schema = format!(
        "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{key_path}"
    );

    run_command(
        "gsettings",
        &[
            "set",
            "org.gnome.settings-daemon.plugins.media-keys",
            "custom-keybindings",
            &format!("['{key_path}']"),
        ],
    )?;
    run_command("gsettings", &["set", &schema, "name", "VCopy"])?;
    run_command("gsettings", &["set", &schema, "command", &script_str])?;
    run_command("gsettings", &["set", &schema, "binding", &binding])?;

    // Clipboard Indicator can otherwise capture Super+V before custom shortcuts.
    let _ = run_command(
        "dconf",
        &[
            "write",
            "/org/gnome/shell/extensions/clipboard-indicator/toggle-menu",
            "@as []",
        ],
    );
    let _ = run_command(
        "dconf",
        &[
            "write",
            "/org/gnome/shell/extensions/clipboard-indicator/enable-keybindings",
            "false",
        ],
    );

    Ok(())
}

fn try_add_to_wm_config(wm: &str, line: &str) -> Result<std::path::PathBuf> {
    let config_dir = dirs::config_dir().unwrap_or_default();
    let path = match wm {
        "sway" => config_dir.join("sway/config"),
        _      => config_dir.join("i3/config"),
    };

    anyhow::ensure!(path.exists(), "config not found");

    let existing = std::fs::read_to_string(&path)?;

    // Avoid duplicate entries.
    if existing.contains("vcopy-popup") {
        println!("(shortcut already present in config — skipped)");
        return Ok(path);
    }

    let mut f = std::fs::OpenOptions::new().append(true).open(&path)?;
    writeln!(f, "\n# vcopy shortcut\n{line}")?;
    Ok(path)
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn detect_wm() -> &'static str {
    if std::env::var("SWAYSOCK").is_ok() { return "sway"; }
    if std::env::var("I3SOCK").is_ok()   { return "i3"; }
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
    if desktop.contains("gnome")                         { return "gnome"; }
    if desktop.contains("kde") || desktop.contains("plasma") { return "kde"; }
    if desktop.contains("xfce")                          { return "xfce"; }
    "unknown"
}

/// Converts "super+v" to the WM-native format (e.g. "Mod4+v" for i3/sway).
fn wm_key_format(key: &str, wm: &str) -> String {
    match wm {
        "sway" | "i3" => key
            .replace("super", "Mod4")
            .replace("ctrl", "Ctrl")
            .replace("alt", "Mod1")
            .replace("shift", "Shift"),
        _ => key.to_string(),
    }
}

fn gnome_key_format(key: &str) -> String {
    key.split('+')
        .filter(|part| !part.is_empty())
        .map(|part| match part.to_lowercase().as_str() {
            "super" => "<Super>".to_string(),
            "ctrl" | "control" => "<Control>".to_string(),
            "alt" => "<Alt>".to_string(),
            "shift" => "<Shift>".to_string(),
            other => other.to_string(),
        })
        .collect()
}

fn run_command(program: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("failed to start {program}"))?;

    anyhow::ensure!(status.success(), "{program} exited with {status}");
    Ok(())
}

fn which(bin: &str) -> bool {
    std::process::Command::new("which").arg(bin).output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn readline() -> Result<String> {
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(s.trim().to_string())
}
