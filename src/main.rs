use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4::{gio, glib};

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
                    button.add_css_class("active");
                } else {
                    button.remove_css_class("active");
                }
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
