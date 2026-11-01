//! Interfaccia di InstallDesk.

use crate::catalog::CATALOG;
use crate::config::{self, AppConfig};
use crate::manager::{self, Status};
use crate::scan::{self, Item};
use crate::theme;
use crate::winget::RealWinget;
use eframe::egui::{self, Color32, RichText};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

pub const APP_TITLE: &str = "InstallDesk";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

enum Event {
    Log(Color32, String),
    Checked(String, Status),
    Downloaded(String, Result<PathBuf, String>),
    Done,
}

struct Row {
    item: Item,
    selected: bool,
    status: Option<Status>,
}

pub struct App {
    cfg: AppConfig,
    cfg_path: PathBuf,
    app_dir: PathBuf,
    rows: Vec<Row>,
    log: Vec<(Color32, String)>,
    busy: bool,
    tx: Sender<Event>,
    rx: Receiver<Event>,
    catalog_open: bool,
    catalog_filter: String,
    catalog_sel: BTreeSet<usize>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);
        let app_dir = config::app_dir();
        let cfg_path = app_dir.join(config::CONFIG_FILE);
        let cfg = config::load(&cfg_path);
        let (tx, rx) = channel();
        let mut app = Self {
            cfg,
            cfg_path,
            app_dir,
            rows: Vec::new(),
            log: Vec::new(),
            busy: false,
            tx,
            rx,
            catalog_open: false,
            catalog_filter: String::new(),
            catalog_sel: BTreeSet::new(),
        };
        app.refresh();
        let dir = app.root().display().to_string();
        app.say(theme::LOG_INFO, format!("Cartella degli installer: {dir}"));
        app
    }

    fn root(&self) -> PathBuf {
        self.cfg.effective_dir(&self.app_dir)
    }

    fn say(&mut self, color: Color32, msg: String) {
        let t = chrono::Local::now().format("%H:%M:%S");
        self.log.push((color, format!("[{t}] {msg}")));
    }

    fn save_cfg(&mut self) {
        if let Err(e) = config::save(&self.cfg_path, &self.cfg) {
            self.say(
                theme::LOG_ERR,
                format!("Impossibile salvare le impostazioni: {e}"),
            );
        }
    }

    fn refresh(&mut self) {
        let old: Vec<(String, bool, Option<Status>)> = self
            .rows
            .drain(..)
            .map(|r| (r.item.program, r.selected, r.status))
            .collect();
        let root = self.root();
        self.rows = scan::scan(&root, &self.cfg)
            .into_iter()
            .map(|item| {
                let prev = old.iter().find(|(p, _, _)| *p == item.program);
                Row {
                    selected: prev.is_none_or(|p| p.1),
                    status: prev.and_then(|p| p.2.clone()),
                    item,
                }
            })
            .collect();
    }

    fn choose_folder(&mut self) {
        if let Some(dir) = rfd::FileDialog::new()
            .set_title("Cartella degli installer")
            .pick_folder()
        {
            self.cfg.download_dir = dir.display().to_string();
            self.save_cfg();
            self.refresh();
            self.say(
                theme::LOG_INFO,
                format!("Nuova cartella: {}", dir.display()),
            );
        }
    }

    fn open_folder(&mut self) {
        let root = self.root();
        if let Err(e) = std::fs::create_dir_all(&root) {
            self.say(theme::LOG_ERR, format!("{}: {e}", root.display()));
            return;
        }
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("explorer").arg(&root).spawn();
        }
        #[cfg(not(windows))]
        {
            let _ = std::process::Command::new("xdg-open").arg(&root).spawn();
        }
    }

    fn start_check(&mut self) {
        let jobs: Vec<Item> = self
            .rows
            .iter()
            .filter(|r| r.item.winget_id.is_some())
            .map(|r| r.item.clone())
            .collect();
        if jobs.is_empty() {
            self.say(
                theme::LOG_WARN,
                "Nessun programma riconosciuto da controllare.".into(),
            );
            return;
        }
        self.busy = true;
        self.say(
            theme::LOG_INFO,
            format!("Controllo versioni di {} programmi...", jobs.len()),
        );
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let w = RealWinget;
            for it in jobs {
                let st = manager::check(&w, &it);
                let _ = tx.send(Event::Checked(it.program.clone(), st));
            }
            let _ = tx.send(Event::Done);
        });
    }

    /// `(programma, id winget)` da scaricare.
    fn start_download(&mut self, jobs: Vec<(String, String)>) {
        if jobs.is_empty() {
            self.say(theme::LOG_WARN, "Niente da scaricare.".into());
            return;
        }
        self.busy = true;
        let root = self.root();
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let w = RealWinget;
            for (program, id) in jobs {
                let _ = tx.send(Event::Log(theme::LOG_INFO, format!("Scarico {program}...")));
                let res = manager::download_update(&w, &root, &program, &id);
                let _ = tx.send(Event::Downloaded(program, res));
            }
            let _ = tx.send(Event::Done);
        });
    }

    fn download_selected(&mut self) {
        let jobs = self
            .rows
            .iter()
            .filter(|r| r.selected && matches!(r.status, Some(Status::Outdated { .. })))
            .filter_map(|r| {
                r.item
                    .winget_id
                    .clone()
                    .map(|id| (r.item.program.clone(), id))
            })
            .collect();
        self.start_download(jobs);
    }

    fn download_from_catalog(&mut self) {
        let jobs = self
            .catalog_sel
            .iter()
            .filter_map(|&i| CATALOG.get(i))
            .map(|e| (e.name.to_string(), e.winget_id.to_string()))
            .collect();
        self.catalog_sel.clear();
        self.start_download(jobs);
    }

    fn make_scripts(&mut self) {
        let items: Vec<Item> = self
            .rows
            .iter()
            .filter(|r| r.selected)
            .map(|r| r.item.clone())
            .collect();
        if items.is_empty() {
            self.say(theme::LOG_WARN, "Nessun programma selezionato.".into());
            return;
        }
        let root = self.root();
        match manager::write_scripts(&root, &items) {
            Ok(n) => self.say(
                theme::LOG_OK,
                format!(
                    "Creati {n} script singoli e Installa-tutto.cmd in {}",
                    root.display()
                ),
            ),
            Err(e) => self.say(theme::LOG_ERR, format!("Errore: {e}")),
        }
    }

    fn pump(&mut self) {
        let mut refresh = false;
        while let Ok(ev) = self.rx.try_recv() {
            match ev {
                Event::Log(c, m) => self.say(c, m),
                Event::Checked(program, st) => {
                    let (c, msg) = match &st {
                        Status::UpToDate => (theme::LOG_OK, format!("{program}: aggiornato")),
                        Status::Outdated { latest } => {
                            (theme::LOG_WARN, format!("{program}: disponibile {latest}"))
                        }
                        Status::Unknown(e) => (theme::LOG_ERR, format!("{program}: {e}")),
                    };
                    self.say(c, msg);
                    if let Some(r) = self.rows.iter_mut().find(|r| r.item.program == program) {
                        r.status = Some(st);
                    }
                }
                Event::Downloaded(program, res) => match res {
                    Ok(p) => {
                        self.say(
                            theme::LOG_OK,
                            format!("{program}: scaricato {}", p.display()),
                        );
                        refresh = true;
                    }
                    Err(e) => self.say(theme::LOG_ERR, format!("{program}: {e}")),
                },
                Event::Done => {
                    self.busy = false;
                    refresh = true;
                    self.say(theme::LOG_INFO, "Operazione terminata.".into());
                }
            }
        }
        if refresh {
            self.refresh();
        }
    }

    fn status_text(st: &Option<Status>) -> RichText {
        match st {
            None => RichText::new("—").color(theme::MUTED),
            Some(Status::UpToDate) => RichText::new("Aggiornato").color(theme::OK),
            Some(Status::Outdated { latest }) => RichText::new(format!("Nuova: {latest}"))
                .color(theme::WARN)
                .strong(),
            Some(Status::Unknown(_)) => RichText::new("Non verificabile").color(theme::ERR),
        }
    }

    fn table(&mut self, ui: &mut egui::Ui) {
        if self.rows.is_empty() {
            ui.add_space(30.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("La cartella è vuota").size(18.0).strong());
                ui.label(
                    RichText::new("Usa «Aggiungi programmi» per scaricarne dal catalogo,\noppure copia qui i tuoi installer (anche in sottocartelle).")
                        .color(theme::MUTED),
                );
            });
            return;
        }
        let mut changed_args: Vec<(String, String)> = Vec::new();
        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            egui::Grid::new("rows").striped(true).num_columns(7).spacing([14.0, 8.0]).show(ui, |ui| {
                for h in ["", "Programma", "Versione", "Stato", "Tipo", "Opzioni silenziose", "File"] {
                    ui.label(RichText::new(h).strong().color(theme::MUTED));
                }
                ui.end_row();
                for r in &mut self.rows {
                    ui.checkbox(&mut r.selected, "");
                    ui.label(RichText::new(&r.item.program).strong());
                    ui.label(r.item.version.as_deref().unwrap_or("?"));
                    let resp = ui.label(Self::status_text(&r.status));
                    if let Some(Status::Unknown(e)) = &r.status {
                        resp.on_hover_text(e);
                    }
                    let kind = r.item.kind;
                    let k = ui.label(if kind.is_reliable() {
                        RichText::new(kind.label())
                    } else {
                        RichText::new(kind.label()).color(theme::WARN)
                    });
                    if !kind.is_reliable() {
                        k.on_hover_text("Tipo non riconosciuto: le opzioni sono un tentativo, verifica a mano.");
                    }
                    let before = r.item.args.clone();
                    ui.add_sized([300.0, 24.0], egui::TextEdit::singleline(&mut r.item.args));
                    if r.item.args != before {
                        changed_args.push((r.item.program.clone(), r.item.args.clone()));
                    }
                    ui.label(RichText::new(&r.item.rel_path).color(theme::MUTED));
                    ui.end_row();
                }
            });
        });
        if !changed_args.is_empty() {
            for (program, args) in changed_args {
                self.cfg.silent_overrides.insert(program, args);
            }
            self.save_cfg();
        }
    }

    fn catalog_window(&mut self, ctx: &egui::Context) {
        let mut open = self.catalog_open;
        egui::Window::new("Aggiungi programmi dal catalogo")
            .open(&mut open)
            .default_size([420.0, 520.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Filtra:");
                    ui.text_edit_singleline(&mut self.catalog_filter);
                });
                let f = self.catalog_filter.to_lowercase();
                egui::ScrollArea::vertical()
                    .max_height(380.0)
                    .show(ui, |ui| {
                        for (i, e) in CATALOG.iter().enumerate() {
                            if !f.is_empty() && !e.name.to_lowercase().contains(&f) {
                                continue;
                            }
                            let mut on = self.catalog_sel.contains(&i);
                            if ui
                                .checkbox(&mut on, format!("{}  ({})", e.name, e.winget_id))
                                .changed()
                            {
                                if on {
                                    self.catalog_sel.insert(i);
                                } else {
                                    self.catalog_sel.remove(&i);
                                }
                            }
                        }
                    });
                ui.separator();
                let n = self.catalog_sel.len();
                if ui
                    .add_enabled(
                        !self.busy && n > 0,
                        egui::Button::new(format!("Scarica {n} programmi")),
                    )
                    .clicked()
                {
                    self.download_from_catalog();
                }
            });
        self.catalog_open = open;
    }

    pub fn initial_window_size() -> (f32, f32) {
        (1180.0, 720.0)
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.pump();
        let ctx = ui.ctx().clone();
        egui::Panel::top("top").show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(APP_TITLE)
                        .size(22.0)
                        .strong()
                        .color(theme::PRIMARY),
                );
                ui.label(RichText::new(format!("v{APP_VERSION}")).color(theme::MUTED));
            });
            ui.horizontal(|ui| {
                ui.label("Cartella predefinita:");
                ui.label(RichText::new(self.root().display().to_string()).strong());
                if ui.button("Cambia…").clicked() {
                    self.choose_folder();
                }
                if ui.button("Apri").clicked() {
                    self.open_folder();
                }
            });
            ui.horizontal_wrapped(|ui| {
                let idle = !self.busy;
                if ui
                    .add_enabled(idle, egui::Button::new("Aggiorna elenco"))
                    .clicked()
                {
                    self.refresh();
                }
                if ui
                    .add_enabled(idle, egui::Button::new("Controlla aggiornamenti"))
                    .clicked()
                {
                    self.start_check();
                }
                if ui
                    .add_enabled(idle, egui::Button::new("Scarica aggiornamenti"))
                    .clicked()
                {
                    self.download_selected();
                }
                if ui.button("Aggiungi programmi…").clicked() {
                    self.catalog_open = true;
                }
                if ui
                    .add_enabled(idle, egui::Button::new("Crea script silenziosi"))
                    .clicked()
                {
                    self.make_scripts();
                }
                if self.busy {
                    ui.spinner();
                }
            });
            ui.add_space(4.0);
        });
        egui::Panel::bottom("log").show(ui, |ui| {
            egui::Frame::new()
                .fill(theme::LOG_BG)
                .inner_margin(6.0)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(110.0)
                        .auto_shrink([false, true])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            for (c, m) in &self.log {
                                ui.label(RichText::new(m).color(*c).monospace());
                            }
                        });
                });
        });
        egui::CentralPanel::default().show(ui, |ui| self.table(ui));
        if self.catalog_open {
            self.catalog_window(&ctx);
        }
        if self.busy {
            ctx.request_repaint_after(std::time::Duration::from_millis(150));
        }
    }
}
