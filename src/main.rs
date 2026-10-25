//! zEXEz Sprint 01: native GTK4/Libadwaita application shell.
//! This version only inspects the Wine runtime; it never executes an installer.

mod runtime;

use adw::prelude::*;

fn make_row(title: &str, subtitle: &str) -> adw::ActionRow {
    adw::ActionRow::builder()
        .title(title)
        .subtitle(subtitle)
        .build()
}

fn build_ui(app: &adw::Application) {
    let initial = runtime::RuntimeReport::detect();

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("zEXEz")
        .default_width(980)
        .default_height(700)
        .build();

    let title = adw::WindowTitle::new("zEXEz", "Windows Application Manager for Linux");
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&title));

    let refresh_button = gtk::Button::from_icon_name("view-refresh-symbolic");
    refresh_button.set_tooltip_text(Some("Refresh runtime diagnostics"));
    header.pack_end(&refresh_button);

    let wine_row = make_row("Wine", &initial.wine_summary());
    let server_row = make_row("Wine server", &initial.server_summary());
    let arch_row = make_row("Architecture packages", &initial.arch_summary());
    let tricks_row = make_row("Winetricks", &initial.winetricks_summary());

    let runtime_group = adw::PreferencesGroup::builder()
        .title("Compatibility runtime")
        .description("Read-only diagnostics for the Wine installation on this computer")
        .build();
    runtime_group.add(&wine_row);
    runtime_group.add(&server_row);
    runtime_group.add(&arch_row);
    runtime_group.add(&tricks_row);

    let empty_state = adw::StatusPage::builder()
        .icon_name("application-x-executable-symbolic")
        .title("Welcome to zEXEz")
        .description(
            "A native Linux home for your selected Windows applications. \
             Installing EXE/MSI files and managing isolated prefixes \
             are planned for future sprints.",
        )
        .build();
    empty_state.set_vexpand(true);

    let note = gtk::Label::new(Some(
        "Sprint 01 · Diagnostic mode only · No application prefixes are created",
    ));
    note.add_css_class("dim-label");
    note.set_wrap(true);
    note.set_halign(gtk::Align::Center);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 20);
    content.set_margin_top(16);
    content.set_margin_bottom(28);
    content.set_margin_start(28);
    content.set_margin_end(28);
    content.append(&empty_state);
    content.append(&runtime_group);
    content.append(&note);

    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_hscrollbar_policy(gtk::PolicyType::Never);
    scrolled.set_child(Some(&content));

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&scrolled));
    window.set_content(Some(&toolbar));

    refresh_button.connect_clicked(move |_| {
        let report = runtime::RuntimeReport::detect();
        wine_row.set_subtitle(&report.wine_summary());
        server_row.set_subtitle(&report.server_summary());
        arch_row.set_subtitle(&report.arch_summary());
        tricks_row.set_subtitle(&report.winetricks_summary());
    });

    window.present();
}

fn main() {
    let application = adw::Application::builder()
        .application_id("io.github.azislamayuda.zexez")
        .build();
    application.connect_activate(build_ui);
    application.run();
}
