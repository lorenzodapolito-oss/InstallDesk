//! Controllo aggiornamenti, download verificato e creazione degli script.

use crate::detect;
use crate::scan::Item;
use crate::scripts;
use crate::version;
use crate::winget::Winget;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    UpToDate,
    Outdated { latest: String },
    Unknown(String),
}

pub fn check(w: &dyn Winget, item: &Item) -> Status {
    let Some(id) = &item.winget_id else {
        return Status::Unknown("non nel catalogo".into());
    };
    match w.latest_version(id) {
        Err(e) => Status::Unknown(e),
        Ok(latest) => match &item.version {
            None => Status::Outdated { latest },
            Some(local) if version::is_newer(&latest, local) => Status::Outdated { latest },
            Some(_) => Status::UpToDate,
        },
    }
}

fn installers_in(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                .map(|e| e.path())
                .filter(|p| {
                    let n = p.to_string_lossy().to_lowercase();
                    [
                        ".exe",
                        ".msi",
                        ".msix",
                        ".appx",
                        ".msixbundle",
                        ".appxbundle",
                    ]
                    .iter()
                    .any(|x| n.ends_with(x))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Scarica la nuova versione in `<radice>\<Programma>\`.
/// Il download passa da una cartella temporanea: se il file non sembra un installer
/// viene scartato. Le versioni precedenti vengono spostate in `Vecchie\`, mai cancellate.
pub fn download_update(
    w: &dyn Winget,
    root: &Path,
    program: &str,
    winget_id: &str,
) -> Result<PathBuf, String> {
    let folder = root.join(scripts::safe_name(program));
    let tmp = folder.join(".nuovo");
    let _ = std::fs::remove_dir_all(&tmp);
    w.download(winget_id, &tmp)?;
    let fresh: Vec<PathBuf> = installers_in(&tmp);
    let Some(file) = fresh
        .iter()
        .find(|p| detect::looks_like_installer(p))
        .cloned()
    else {
        let _ = std::fs::remove_dir_all(&tmp);
        return Err("il file scaricato non è un installer valido".into());
    };
    let name = file.file_name().ok_or("nome file mancante")?.to_os_string();
    let dest = folder.join(&name);
    if dest.exists() {
        let _ = std::fs::remove_dir_all(&tmp);
        return Ok(dest);
    }
    let old_dir = folder.join("Vecchie");
    for old in installers_in(&folder) {
        std::fs::create_dir_all(&old_dir).map_err(|e| e.to_string())?;
        if let Some(n) = old.file_name() {
            std::fs::rename(&old, old_dir.join(n))
                .map_err(|e| format!("{}: {e}", old.display()))?;
        }
    }
    std::fs::rename(&file, &dest).map_err(|e| format!("{}: {e}", dest.display()))?;
    let _ = std::fs::remove_dir_all(&tmp);
    Ok(dest)
}

/// Crea un `.cmd` accanto a ogni installer e `Installa-tutto.cmd` nella radice.
/// Restituisce il numero di script singoli creati.
pub fn write_scripts(root: &Path, items: &[Item]) -> Result<usize, String> {
    let mut n = 0;
    for it in items {
        let stem = scripts::safe_name(&format!("Installa {}", it.program));
        let dir = it.path.parent().unwrap_or(root);
        let content = scripts::single(&it.program, &it.file_name, it.kind, &it.args);
        scripts::write(&dir.join(format!("{stem}.cmd")), &content)?;
        n += 1;
    }
    let jobs: Vec<scripts::Job<'_>> = items
        .iter()
        .map(|i| scripts::Job {
            display: &i.program,
            rel_path: &i.rel_path,
            kind: i.kind,
            args: &i.args,
        })
        .collect();
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    scripts::write(
        &root.join("Installa-tutto.cmd"),
        &scripts::install_all(&jobs),
    )?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::detect::Kind;
    use crate::scan;
    use crate::winget::fake::FakeWinget;

    fn dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("idesk_mgr_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn item(version: Option<&str>, id: Option<&str>) -> Item {
        Item {
            program: "Git".into(),
            winget_id: id.map(str::to_string),
            path: PathBuf::from("Git/Git-2.0.exe"),
            file_name: "Git-2.0.exe".into(),
            rel_path: "Git\\Git-2.0.exe".into(),
            version: version.map(str::to_string),
            kind: Kind::Inno,
            args: "/VERYSILENT".into(),
        }
    }

    fn fake(version: &str) -> FakeWinget {
        let mut f = FakeWinget {
            file_name: "Git-2.47.1-64-bit.exe".into(),
            ..Default::default()
        };
        f.versions.insert("Git.Git".into(), version.into());
        f
    }

    #[test]
    fn status_cases() {
        let w = fake("2.47.1");
        assert_eq!(
            check(&w, &item(Some("2.40.0"), Some("Git.Git"))),
            Status::Outdated {
                latest: "2.47.1".into()
            }
        );
        assert_eq!(
            check(&w, &item(Some("2.47.1"), Some("Git.Git"))),
            Status::UpToDate
        );
        assert_eq!(
            check(&w, &item(None, Some("Git.Git"))),
            Status::Outdated {
                latest: "2.47.1".into()
            }
        );
        assert!(matches!(
            check(&w, &item(Some("1"), None)),
            Status::Unknown(_)
        ));
        assert!(matches!(
            check(&w, &item(Some("1"), Some("Altro.Id"))),
            Status::Unknown(_)
        ));
    }

    #[test]
    fn download_moves_old_versions_to_backup_folder() {
        let root = dir("dl");
        std::fs::create_dir_all(root.join("Git")).unwrap();
        std::fs::write(root.join("Git").join("Git-2.40.0.exe"), b"MZ old").unwrap();
        let path = download_update(&fake("2.47.1"), &root, "Git", "Git.Git").unwrap();
        assert_eq!(path, root.join("Git").join("Git-2.47.1-64-bit.exe"));
        assert!(path.exists());
        assert!(root
            .join("Git")
            .join("Vecchie")
            .join("Git-2.40.0.exe")
            .exists());
        assert!(!root.join("Git").join("Git-2.40.0.exe").exists());
        assert!(!root.join("Git").join(".nuovo").exists());
    }

    #[test]
    fn download_is_idempotent() {
        let root = dir("idem");
        let w = fake("2.47.1");
        let a = download_update(&w, &root, "Git", "Git.Git").unwrap();
        let b = download_update(&w, &root, "Git", "Git.Git").unwrap();
        assert_eq!(a, b);
        assert!(!root.join("Git").join("Vecchie").exists());
    }

    #[test]
    fn invalid_download_is_rejected_and_old_file_kept() {
        let root = dir("bad");
        std::fs::create_dir_all(root.join("Git")).unwrap();
        std::fs::write(root.join("Git").join("Git-2.40.0.exe"), b"MZ old").unwrap();
        struct Html;
        impl Winget for Html {
            fn latest_version(&self, _: &str) -> Result<String, String> {
                Ok("9".into())
            }
            fn download(&self, _: &str, d: &Path) -> Result<(), String> {
                std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
                std::fs::write(d.join("x.exe"), b"<html>errore</html>").map_err(|e| e.to_string())
            }
        }
        assert!(download_update(&Html, &root, "Git", "Git.Git").is_err());
        assert!(root.join("Git").join("Git-2.40.0.exe").exists());
        assert!(!root.join("Git").join(".nuovo").exists());
    }

    #[test]
    fn download_error_is_reported() {
        let root = dir("err");
        let mut w = fake("2.47.1");
        w.fail_download = true;
        assert!(download_update(&w, &root, "Git", "Git.Git")
            .unwrap_err()
            .contains("rete"));
    }

    #[test]
    fn scripts_are_written_next_to_installers_and_in_root() {
        let root = dir("scr");
        std::fs::create_dir_all(root.join("Git")).unwrap();
        std::fs::write(root.join("Git").join("Git-2.47.1.exe"), b"MZ Inno Setup").unwrap();
        let items = scan::scan(&root, &AppConfig::default());
        assert_eq!(write_scripts(&root, &items).unwrap(), 1);
        let single = std::fs::read_to_string(root.join("Git").join("Installa Git.cmd")).unwrap();
        assert!(single.contains("/VERYSILENT"));
        let all = std::fs::read_to_string(root.join("Installa-tutto.cmd")).unwrap();
        assert!(all.contains("Git\\Git-2.47.1.exe"));
    }
}
