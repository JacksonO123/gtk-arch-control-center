use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4::{gio, glib};

const ACTIVE_CLASS: &'static str = "active";

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("jackson.control_center")
        .build();

    app.connect_startup(|_| load_css());
    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Control Center")
            .default_width(400)
            .default_height(200)
            .build();

        let active = std::rc::Rc::new(std::cell::Cell::new(false));

        let button = gtk::Button::builder()
            .label("i am a button")
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();
        button.set_size_request(50, 25);
        button.connect_clicked(glib::clone!(
            #[strong]
            button,
            move |_| {
                active.set(!active.get());
                if active.get() {
                    button.add_css_class(ACTIVE_CLASS);
                } else {
                    button.remove_css_class(ACTIVE_CLASS);
                }
                toggle_hyprsunset();
                start_timeout_check(button.clone(), active.clone());
            }
        ));

        window.set_child(Some(&button));

        window.present();
    });

    app.run()
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    let gio_file = gio::File::for_path("assets/style.css");
    provider.load_from_file(&gio_file);

    gtk::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn toggle_hyprsunset() {
    _ = std::process::Command::new("sh")
        .arg("-c")
        .arg("if pgrep -x hyprsunset >/dev/null; then pkill -INT hyprsunset; else hyprsunset --temperature 4000 & fi")
        .stdout(std::process::Stdio::null())
        .spawn()
        .expect("Failed to toggle hyprsunset");
}

fn start_timeout_check(button: gtk::Button, active: std::rc::Rc<std::cell::Cell<bool>>) {
    glib::MainContext::default().spawn_local(async move {
        glib::timeout_future_seconds(1).await;
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg("pgrep -x hyprsunset >/dev/null && echo true || echo false")
            .output()
            .expect("Failed to check on hyprsunset");

        if !output.status.success() {
            return;
        }

        let output = String::from_utf8(output.stdout).unwrap();
        let output = output.trim();

        if output == "true" {
            button.add_css_class(ACTIVE_CLASS);
            active.set(true);
        } else {
            button.remove_css_class(ACTIVE_CLASS);
            active.set(false);
        }
    });
}
