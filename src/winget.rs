//! Interfaccia con winget (versione più recente e download degli installer).

use std::path::Path;
use std::process::{Command, Stdio};

pub trait Winget {
    /// Ultima versione disponibile per l'id.
    fn latest_version(&self, id: &str) -> Result<String, String>;
    /// Scarica l'installer dell'id dentro `dir`.
    fn download(&self, id: &str, dir: &Path) -> Result<(), String>;
}

pub struct RealWinget;

fn run(args: &[&str]) -> Result<String, String> {
    let out = Command::new("winget")
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("winget non disponibile: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    if out.status.success() {
        Ok(text)
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        Err(format!(
            "winget codice {:?}: {}{}",
            out.status.code(),
            text.trim(),
            err.trim()
        ))
    }
}

impl Winget for RealWinget {
    fn latest_version(&self, id: &str) -> Result<String, String> {
        let out = run(&[
            "show",
            "--id",
            id,
            "--exact",
            "--source",
            "winget",
            "--accept-source-agreements",
            "--disable-interactivity",
        ])?;
        parse_show_version(&out).ok_or_else(|| format!("versione non trovata per {id}"))
    }

    fn download(&self, id: &str, dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let d = dir.to_string_lossy().to_string();
        run(&[
            "download",
            "--id",
            id,
            "--exact",
            "--source",
            "winget",
            "--download-directory",
            &d,
            "--accept-package-agreements",
            "--accept-source-agreements",
            "--disable-interactivity",
        ])
        .map(|_| ())
    }
}

/// Estrae `Versione: x` / `Version: x` dall'output di `winget show`.
pub fn parse_show_version(out: &str) -> Option<String> {
    out.lines().find_map(|line| {
        let line = line.trim();
        let (key, val) = line.split_once(':')?;
        let key = key.trim().to_lowercase();
        let val = val.trim();
        ((key == "version" || key == "versione") && !val.is_empty()).then(|| val.to_string())
    })
}

#[cfg(test)]
pub mod fake {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Default)]
    pub struct FakeWinget {
        pub versions: HashMap<String, String>,
        pub downloaded: RefCell<Vec<String>>,
        /// Nome del file da creare nella cartella al download.
        pub file_name: String,
        pub fail_download: bool,
    }

    impl Winget for FakeWinget {
        fn latest_version(&self, id: &str) -> Result<String, String> {
            self.versions
                .get(id)
                .cloned()
                .ok_or_else(|| "sconosciuto".into())
        }
        fn download(&self, id: &str, dir: &Path) -> Result<(), String> {
            if self.fail_download {
                return Err("rete assente".into());
            }
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            std::fs::write(dir.join(&self.file_name), b"MZ fake installer")
                .map_err(|e| e.to_string())?;
            self.downloaded.borrow_mut().push(id.to_string());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_english_and_italian_output() {
        let en = "Found Git [Git.Git]\nVersion: 2.47.1\nPublisher: The Git Development Community\n";
        let it = "Trovato VLC [VideoLAN.VLC]\nVersione: 3.0.21\n";
        assert_eq!(parse_show_version(en).as_deref(), Some("2.47.1"));
        assert_eq!(parse_show_version(it).as_deref(), Some("3.0.21"));
        assert_eq!(parse_show_version("niente"), None);
        assert_eq!(parse_show_version("Version:\n"), None);
    }
}
