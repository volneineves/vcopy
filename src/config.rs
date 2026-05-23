use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{io::{self, Write}, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub language: String,
    #[serde(default)]
    pub shortcut: ShortcutConfig,
    #[serde(default)]
    pub history: HistoryConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShortcutConfig {
    #[serde(default = "default_key")]
    pub key: String,
    #[serde(default = "detect_terminal")]
    pub terminal: String,
    #[serde(default = "default_width")]
    pub width: u16,
    #[serde(default = "default_height")]
    pub height: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    #[serde(default = "default_history_limit")]
    pub limit: usize,
    #[serde(default)]
    pub clear_on_restart: bool,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            key: default_key(),
            terminal: detect_terminal(),
            width: default_width(),
            height: default_height(),
        }
    }
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            limit: default_history_limit(),
            clear_on_restart: false,
        }
    }
}

impl HistoryConfig {
    pub fn effective_limit(&self) -> usize {
        if self.limit == 0 {
            default_history_limit()
        } else {
            self.limit
        }
    }
}

fn default_key() -> String { "super+v".into() }
fn default_width() -> u16 { 100 }
fn default_height() -> u16 { 28 }
fn default_history_limit() -> usize { 50 }
fn detect_terminal() -> String {
    for t in ["kgx", "kitty", "alacritty", "foot", "wezterm", "gnome-terminal", "konsole", "xfce4-terminal", "xterm"] {
        if which(t) { return t.into(); }
    }
    "xterm".into()
}

impl Config {
    pub fn load_or_setup() -> Result<Self> {
        let path = config_path();
        if path.exists() {
            let raw = std::fs::read_to_string(&path)?;
            return Ok(toml::from_str(&raw)?);
        }

        let detected = crate::i18n::detect_system_lang();
        let s = crate::i18n::t();
        println!(
            "\n\x1b[1;32mVCopy\x1b[0m — {}\n{} {}\n",
            s.setup_title, s.setup_detected, crate::i18n::lang_label(detected),
        );
        print!("{}", s.setup_prompt);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let lang = match input.trim() {
            "1" => "en",
            "2" => "pt",
            "3" => "es",
            "" => detected,
            _ => { eprintln!("{}", crate::i18n::t().setup_invalid); detected }
        };

        crate::i18n::init(lang);

        let cfg = Config {
            language: lang.to_string(),
            shortcut: ShortcutConfig::default(),
            history: HistoryConfig::default(),
        };
        cfg.save()?;
        println!("{} {}\n", crate::i18n::t().setup_saved, crate::i18n::lang_label(lang));

        offer_install()?;
        Ok(cfg)
    }

    pub fn load() -> Result<Self> {
        let path = config_path();
        if path.exists() {
            return Ok(toml::from_str(&std::fs::read_to_string(&path)?)?);
        }
        Ok(Config {
            language: "en".into(),
            shortcut: ShortcutConfig::default(),
            history: HistoryConfig::default(),
        })
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path();
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn set_language(lang: &str) -> Result<()> {
        let mut cfg = Self::load()?;
        cfg.language = lang.to_string();
        cfg.save()
    }
}

fn offer_install() -> Result<()> {
    let s = crate::i18n::t();
    print!("{}", s.install_prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    match input.trim().to_lowercase().as_str() {
        "" | "y" | "s" => { crate::service::install()?; println!("{}", s.install_yes); }
        _ => println!("{}", s.install_skipped),
    }
    Ok(())
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vcopy")
        .join("config.toml")
}

fn which(bin: &str) -> bool {
    std::process::Command::new("which").arg(bin).output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
