//! zEXEz Sprint 02 Hotfix 02: safer prefix validation and 120 DPI default.
//! A selected executable is NEVER launched without an explicit user action.

mod engine;
mod pe;
mod runtime;

use adw::prelude::*;
use std::cell::RefCell;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;

fn make_row(title: &str, subtitle: &str) -> adw::ActionRow {
    adw::ActionRow::builder()
        .title(title)
        .subtitle(subtitle)
        .build()
}

fn build_ui(app: &adw::Application) {
    let initial = runtime::RuntimeReport::detect();
    let selection: Rc<RefCell<Option<engine::Selection>>> = Rc::new(RefCell::new(None));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("zEXEz")
        .default_width(1000)
        .default_height(720)
        .build();

    let header = adw::HeaderBar::new();
    let title = adw::WindowTitle::new("zEXEz", "Windows Application Manager for Linux");
    header.set_title_widget(Some(&title));
    let refresh = gtk::Button::from_icon_name("view-refresh-symbolic");
    refresh.set_tooltip_text(Some("Refresh Wine diagnostics"));
    header.pack_end(&refresh);

    let wine_row = make_row("Wine", &initial.wine_summary());
    let server_row = make_row("Wine server", &initial.server_summary());
    let arch_row = make_row("Architecture packages", &initial.arch_summary());
    let tricks_row = make_row("Winetricks", &initial.winetricks_summary());
    let runtime_group = adw::PreferencesGroup::builder()
        .title("Compatibility runtime")
        .description("Read-only system checks. Sprint 02 uses isolated Wine prefixes.")
        .build();
    for row in [&wine_row, &server_row, &arch_row, &tricks_row] {
        runtime_group.add(row);
    }

    let file_row = make_row("Windows program", "No file selected");
    let kind_row = make_row("Format", "Choose an EXE or MSI file to inspect");
    let prefix_row = make_row("Managed prefix", "No prefix selected");
    let status_row = make_row("Operation", "Ready · No Windows program has been launched");
    let dpi_row = make_row("Current Wine scaling", "Select an initialized prefix first");
    let program_group = adw::PreferencesGroup::builder()
        .title("Application workspace")
        .description("Inspect a file, initialize its dedicated prefix, then launch it explicitly")
        .build();
    for row in [&file_row, &kind_row, &prefix_row, &dpi_row, &status_row] {
        program_group.add(row);
    }

    let choose_button = gtk::Button::with_label("Select EXE / MSI");
    choose_button.add_css_class("suggested-action");
    let init_button = gtk::Button::with_label("Initialize Prefix");
    init_button.set_sensitive(false);
    let run_button = gtk::Button::with_label("Run Selected File");
    run_button.set_sensitive(false);

    let dpi_choices = gtk::DropDown::from_strings(&[
        "96 DPI (100%)",
        "120 DPI (125%)",
        "144 DPI (150%)",
        "168 DPI (175%)",
        "192 DPI (200%)",
    ]);
    dpi_choices.set_selected(1);
    dpi_choices.set_tooltip_text(Some("Choose a scale for this Wine prefix only"));
    let dpi_button = gtk::Button::with_label("Apply DPI");
    dpi_button.set_sensitive(false);
    let dpi_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    dpi_actions.set_halign(gtk::Align::Start);
    dpi_actions.append(&dpi_choices);
    dpi_actions.append(&dpi_button);

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.set_halign(gtk::Align::Start);
    actions.append(&choose_button);
    actions.append(&init_button);
    actions.append(&run_button);

    let heading = gtk::Label::new(Some("zEXEz · Windows Compatibility Workspace"));
    heading.add_css_class("title-2");
    heading.set_halign(gtk::Align::Start);
    let description = gtk::Label::new(Some(
        "Inspect Windows executables and manage isolated Wine prefixes. \
         Installation success and hardware access are not guaranteed.",
    ));
    description.set_wrap(true);
    description.set_halign(gtk::Align::Start);
    description.set_xalign(0.0);
    description.add_css_class("dim-label");
    let note = gtk::Label::new(Some(
        "Sprint 02 · Prefixes in XDG data directory · Logs in XDG state directory · \
         Programs run with your Linux user permissions (Wine is not a security sandbox)",
    ));
    note.add_css_class("dim-label");
    note.set_wrap(true);
    note.set_xalign(0.0);
    note.set_halign(gtk::Align::Start);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 16);
    content.set_margin_top(24);
    content.set_margin_bottom(32);
    content.set_margin_start(28);
    content.set_margin_end(28);
    for widget in [&heading, &description] {
        content.append(widget);
    }
    content.append(&program_group);
    content.append(&actions);
    content.append(&dpi_actions);
    content.append(&runtime_group);
    content.append(&note);

    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_hscrollbar_policy(gtk::PolicyType::Never);
    scrolled.set_child(Some(&content));
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&scrolled));
    window.set_content(Some(&toolbar));

    refresh.connect_clicked(move |_| {
        let report = runtime::RuntimeReport::detect();
        wine_row.set_subtitle(&report.wine_summary());
        server_row.set_subtitle(&report.server_summary());
        arch_row.set_subtitle(&report.arch_summary());
        tricks_row.set_subtitle(&report.winetricks_summary());
    });

    {
        let window = window.clone();
        let selection = selection.clone();
        let file_row = file_row.clone();
        let kind_row = kind_row.clone();
        let prefix_row = prefix_row.clone();
        let status_row = status_row.clone();
        let init_button = init_button.clone();
        let run_button = run_button.clone();
        let dpi_row = dpi_row.clone();
        let dpi_button = dpi_button.clone();
        choose_button.connect_clicked(move |_| {
            // Native chooser deliberately retained by run_async until it responds.
            #[allow(deprecated)]
            let chooser = gtk::FileChooserNative::new(
                Some("Select a Windows EXE or MSI"),
                Some(&window),
                gtk::FileChooserAction::Open,
                Some("Select"),
                Some("Cancel"),
            );
            let selection = selection.clone();
            let file_row = file_row.clone();
            let kind_row = kind_row.clone();
            let prefix_row = prefix_row.clone();
            let status_row = status_row.clone();
            let init_button = init_button.clone();
            let run_button = run_button.clone();
            let dpi_row = dpi_row.clone();
            let dpi_button = dpi_button.clone();
            #[allow(deprecated)]
            chooser.run_async(move |dialog, response| {
                if response != gtk::ResponseType::Accept {
                    return;
                }
                let Some(path) = dialog.file().and_then(|file| file.path()) else {
                    status_row.set_subtitle("Could not access selected local file");
                    return;
                };
                *selection.borrow_mut() = None;
                init_button.set_sensitive(false);
                run_button.set_sensitive(false);
                dpi_button.set_sensitive(false);
                dpi_row.set_subtitle("Select a valid EXE / MSI first");
                match engine::select_file(&path) {
                    Ok(selected) => {
                        file_row.set_subtitle(&selected.path.display().to_string());
                        kind_row.set_subtitle(&selected.kind.to_string());
                        let prefix =
                            engine::prefix_path(&selected.id).expect("generated ID is safe");
                        let ready = engine::prefix_ready(&selected.id);
                        prefix_row.set_subtitle(&format!(
                            "{} · {}",
                            prefix.display(),
                            if ready {
                                "initialized"
                            } else {
                                "not initialized"
                            }
                        ));
                        status_row.set_subtitle("Inspected only · No program executed");
                        init_button.set_sensitive(!ready);
                        run_button.set_sensitive(ready);
                        dpi_button.set_sensitive(ready);
                        let dpi_description = engine::prefix_dpi(&selected.id)
                            .map(|dpi| format!("Current: {dpi} DPI"))
                            .unwrap_or_else(|| {
                                "Not explicitly set / prefix not initialized".to_string()
                            });
                        dpi_row.set_subtitle(&dpi_description);
                        *selection.borrow_mut() = Some(selected);
                    }
                    Err(error) => {
                        file_row.set_subtitle(&path.display().to_string());
                        kind_row.set_subtitle("Unsupported or damaged file");
                        prefix_row.set_subtitle("No prefix selected");
                        status_row.set_subtitle(&error);
                    }
                }
            });
        });
    }

    {
        let selection = selection.clone();
        let status_row = status_row.clone();
        let prefix_row = prefix_row.clone();
        let run_button = run_button.clone();
        let choose_button = choose_button.clone();
        let init_button_ref = init_button.clone();
        let dpi_row = dpi_row.clone();
        let dpi_button = dpi_button.clone();
        init_button.connect_clicked(move |_| {
            let Some(selected) = selection.borrow().clone() else {
                return;
            };
            init_button_ref.set_sensitive(false);
            choose_button.set_sensitive(false);
            run_button.set_sensitive(false);
            dpi_button.set_sensitive(false);
            status_row
                .set_subtitle("Initializing Wine prefix · Checking registry and Windows drive…");

            let (sender, receiver) = mpsc::channel();
            let id = selected.id.clone();
            std::thread::spawn(move || {
                let _ = sender.send(engine::initialize_prefix(&id));
            });
            let status_row = status_row.clone();
            let prefix_row = prefix_row.clone();
            let run_button = run_button.clone();
            let choose_button = choose_button.clone();
            let init_button = init_button_ref.clone();
            let dpi_row = dpi_row.clone();
            let dpi_button = dpi_button.clone();
            let selected_id = selected.id.clone();
            gtk::glib::timeout_add_local(Duration::from_millis(200), move || {
                match receiver.try_recv() {
                    Ok(result) => {
                        choose_button.set_sensitive(true);
                        match result {
                            Ok(message) => {
                                status_row.set_subtitle(&message);
                                prefix_row.set_subtitle("Dedicated Wine prefix initialized");
                                run_button.set_sensitive(true);
                                dpi_button.set_sensitive(true);
                                let dpi_description = engine::prefix_dpi(&selected_id)
                                    .map(|dpi| format!("Current: {dpi} DPI"))
                                    .unwrap_or_else(|| "DPI not explicitly detected".into());
                                dpi_row.set_subtitle(&dpi_description);
                            }
                            Err(error) => {
                                status_row.set_subtitle(&error);
                                init_button.set_sensitive(true);
                            }
                        }
                        gtk::glib::ControlFlow::Break
                    }
                    Err(TryRecvError::Empty) => gtk::glib::ControlFlow::Continue,
                    Err(TryRecvError::Disconnected) => {
                        status_row.set_subtitle("Prefix worker exited unexpectedly");
                        choose_button.set_sensitive(true);
                        init_button.set_sensitive(true);
                        gtk::glib::ControlFlow::Break
                    }
                }
            });
        });
    }

    {
        let selection = selection.clone();
        let status_row = status_row.clone();
        run_button.connect_clicked(move |_| {
            let Some(selected) = selection.borrow().clone() else {
                return;
            };
            match engine::launch(&selected.id, &selected.path) {
                Ok(info) => status_row.set_subtitle(&format!(
                    "Wine process started · PID {} · Log: {}",
                    info.pid,
                    info.log.display()
                )),
                Err(error) => status_row.set_subtitle(&error),
            }
        });
    }
    {
        let selection = selection.clone();
        let dpi_row = dpi_row.clone();
        let status_row = status_row.clone();
        let choose_button = choose_button.clone();
        let apply_button = dpi_button.clone();
        dpi_button.connect_clicked(move |_| {
            let Some(selected) = selection.borrow().clone() else {
                return;
            };
            let index = dpi_choices.selected() as usize;
            let Some(dpi) = engine::DPI_OPTIONS.get(index).copied() else {
                status_row.set_subtitle("Unsupported DPI selection");
                return;
            };
            apply_button.set_sensitive(false);
            choose_button.set_sensitive(false);
            status_row.set_subtitle(&format!("Configuring {dpi} DPI for selected prefix…"));
            let (sender, receiver) = mpsc::channel();
            let id = selected.id.clone();
            std::thread::spawn(move || {
                let _ = sender.send(engine::apply_dpi(&id, dpi));
            });
            let status_row = status_row.clone();
            let dpi_row = dpi_row.clone();
            let apply_button = apply_button.clone();
            let choose_button = choose_button.clone();
            let id = selected.id.clone();
            gtk::glib::timeout_add_local(Duration::from_millis(200), move || {
                match receiver.try_recv() {
                    Ok(result) => {
                        choose_button.set_sensitive(true);
                        apply_button.set_sensitive(true);
                        match result {
                            Ok(message) => {
                                status_row.set_subtitle(&message);
                                let value = engine::prefix_dpi(&id)
                                    .map(|v| format!("Current: {v} DPI (restart Windows apps)"))
                                    .unwrap_or_else(|| {
                                        "Requested, re-open prefix if status is stale".into()
                                    });
                                dpi_row.set_subtitle(&value);
                            }
                            Err(error) => status_row.set_subtitle(&error),
                        }
                        gtk::glib::ControlFlow::Break
                    }
                    Err(TryRecvError::Empty) => gtk::glib::ControlFlow::Continue,
                    Err(TryRecvError::Disconnected) => {
                        choose_button.set_sensitive(true);
                        apply_button.set_sensitive(true);
                        status_row.set_subtitle("DPI worker exited unexpectedly");
                        gtk::glib::ControlFlow::Break
                    }
                }
            });
        });
    }
    window.present();
}

fn usage() {
    println!(
        "zEXEz 0.1.0 · Sprint 02 Hotfix 02\n\
         Usage:\n\
           zexez                         Open native GUI\n\
           zexez diagnose                Read-only Wine diagnostics\n\
           zexez inspect FILE            Inspect EXE / MSI without executing\n\
           zexez prefix path ID          Print dedicated prefix path\n\
           zexez prefix create ID        Explicitly initialize isolated prefix (120 DPI for new)\n\
           zexez prefix verify ID        Read-only prefix integrity check\n\
           zexez prefix repair ID        Explicitly repair incomplete prefix\n\
           zexez dpi show ID             Read stored DPI (no Wine execution)\n\
           zexez dpi set ID VALUE        Set DPI 96/120/144/168/192 for prefix\n\
           zexez run ID FILE             Launch EXE / MSI using initialized prefix\n\
           zexez --help                  Print help\n\n\
         IDs: lowercase a-z, 0-9 and hyphens (max 48 chars).\n\
         No shell, sudo, or ~/.wine modification. Wine itself may show component prompts."
    );
}

fn run_cli(args: &[OsString]) -> Result<(), String> {
    let word = |idx: usize| -> Result<&str, String> {
        args.get(idx)
            .and_then(|arg| arg.to_str())
            .ok_or_else(|| "Missing or non-UTF-8 command/ID".to_string())
    };
    let path = |idx: usize| -> Result<PathBuf, String> {
        args.get(idx)
            .map(|arg| PathBuf::from(arg.as_os_str()))
            .ok_or_else(|| "Missing file path".to_string())
    };
    match (args.len(), word(0)?) {
        (1, "--help" | "-h" | "help") => usage(),
        (1, "diagnose") => {
            let r = runtime::RuntimeReport::detect();
            println!("Wine: {}", r.wine_summary());
            println!("Server: {}", r.server_summary());
            println!("Packages: {}", r.arch_summary());
            println!("Winetricks: {}", r.winetricks_summary());
        }
        (2, "inspect") => {
            let file = path(1)?;
            let selected = engine::select_file(&file)?;
            println!("File: {}", selected.path.display());
            println!("Format: {}", selected.kind);
            println!("Suggested ID: {}", selected.id);
            println!("Prefix: {}", engine::prefix_path(&selected.id)?.display());
            println!("No program was executed.");
        }
        (3, "prefix") if word(1)? == "path" => {
            println!("{}", engine::prefix_path(word(2)?)?.display());
        }
        (3, "prefix") if word(1)? == "create" => {
            println!("{}", engine::initialize_prefix(word(2)?)?);
        }
        (3, "prefix") if word(1)? == "verify" => {
            println!("{}", engine::verify_prefix(word(2)?)?);
        }
        (3, "prefix") if word(1)? == "repair" => {
            println!("{}", engine::repair_prefix(word(2)?)?);
        }
        (3, "dpi") if word(1)? == "show" => {
            println!("{}", engine::dpi_summary(word(2)?)?);
        }
        (4, "dpi") if word(1)? == "set" => {
            let dpi = word(3)?
                .parse::<u32>()
                .map_err(|_| "DPI must be a number".to_string())?;
            println!("{}", engine::apply_dpi(word(2)?, dpi)?);
        }
        (3, "run") => {
            let info = engine::launch(word(1)?, &path(2)?)?;
            println!("Wine process started · PID {}", info.pid);
            println!("Log: {}", info.log.display());
            println!("Starting a process does not guarantee software compatibility.");
        }
        _ => return Err("Unrecognized command or argument count. Run: zexez --help".into()),
    }
    Ok(())
}

fn main() {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if !args.is_empty() {
        if let Err(error) = run_cli(&args) {
            eprintln!("zEXEz: {error}");
            std::process::exit(1);
        }
        return;
    }
    let application = adw::Application::builder()
        .application_id("io.github.azislamayuda.zexez")
        .build();
    application.connect_activate(build_ui);
    application.run();
}
