use std::{env, path};

use gtk::{gdk, gio, glib};
use gtk4 as gtk;

use crate::constants;

pub fn load_css() {
    let mut config_path = glib::user_config_dir();
    config_path.push(constants::JOTTO_LIB_CONFIG_DIR);
    config_path.push(constants::APP_CONFIG_DIR);
    config_path.push(constants::STYLE_FILE);

    let default_display = &gdk::Display::default().expect("Could not connect to a display");

    if config_path.exists() {
        let provider = gtk::CssProvider::new();
        let gio_file = gio::File::for_path(config_path);
        provider.load_from_file(&gio_file);
        gtk::style_context_add_provider_for_display(
            default_display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let provider = gtk::CssProvider::new();
    provider.load_from_data(constants::DEFAULT_STYLES);
    gtk::style_context_add_provider_for_display(
        default_display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

pub fn get_home_dir() -> Option<path::PathBuf> {
    #[cfg(not(unix))]
    {
        panic!("Unsupported os. I hope you are not using windows.");
    }

    env::var_os("HOME").map(path::PathBuf::from)
}
