use gtk4::prelude::*;
use gtk4::{self as gtk, gdk};
use gtk4::{gio, glib};
use gtk4_layer_shell::LayerShell;

const ACTIVE_CLASS: &str = "active";

trait ToggleUtil {
    const ENABLED_OUTPUT_SUCCESS_CASE: &'static str;

    fn is_enabled_cmd() -> std::process::Output;
    fn toggle();

    fn is_enabled() -> bool {
        let output = Self::is_enabled_cmd();

        if !output.status.success() {
            return false;
        }

        let output = String::from_utf8(output.stdout).unwrap();
        output.trim() == Self::ENABLED_OUTPUT_SUCCESS_CASE
    }

    fn begin_timeout_check(button: gtk::Button, active: std::rc::Rc<std::cell::Cell<bool>>) {
        glib::MainContext::default().spawn_local(async move {
            glib::timeout_future_seconds(1).await;
            if Self::is_enabled() {
                button.add_css_class(ACTIVE_CLASS);
                active.set(true);
            } else {
                button.remove_css_class(ACTIVE_CLASS);
                active.set(false);
            }
        });
    }
}

struct HyprsunsetUtils;
impl ToggleUtil for HyprsunsetUtils {
    const ENABLED_OUTPUT_SUCCESS_CASE: &'static str = "1";

    fn is_enabled_cmd() -> std::process::Output {
        std::process::Command::new("sh")
            .arg("-c")
            .arg("pgrep -x hyprsunset >/dev/null && echo 1 || echo 0")
            .output()
            .expect("Failed to check on hyprsunset")
    }

    fn toggle() {
        _ = std::process::Command::new("sh")
        .arg("-c")
        .arg("if pgrep -x hyprsunset >/dev/null; then pkill -INT hyprsunset; else setsid -f hyprsunset --temperature 4000 >/dev/null 2>&1; fi")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to toggle hyprsunset")
    }
}

struct WifiUtils;
impl ToggleUtil for WifiUtils {
    const ENABLED_OUTPUT_SUCCESS_CASE: &'static str = "enabled";

    fn is_enabled_cmd() -> std::process::Output {
        std::process::Command::new("nmcli")
            .arg("radio")
            .arg("wifi")
            .output()
            .expect("Failed to check on wifi")
    }

    fn toggle() {
        _ = std::process::Command::new("sh")
            .arg("-c")
            .arg("[[ $(nmcli radio wifi) == \"enabled\" ]] && nmcli radio wifi off || nmcli radio wifi on")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("Failed to toggle wifi")
    }
}

struct BluetoothUtils;
impl ToggleUtil for BluetoothUtils {
    const ENABLED_OUTPUT_SUCCESS_CASE: &'static str = "1";

    fn is_enabled_cmd() -> std::process::Output {
        unreachable!()
    }

    fn is_enabled() -> bool {
        let cmd1 = std::process::Command::new("bluetoothctl")
            .arg("show")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to check bluetooth");

        let cmd1_stdout = cmd1.stdout.expect("Failed to open bluetooth stdout");

        let cmd2 = std::process::Command::new("grep")
            .arg("-q")
            .arg("Powered: yes")
            .stdin(std::process::Stdio::from(cmd1_stdout))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .expect("Failed to search bluetooth");

        cmd2.status.success()
    }

    fn toggle() {
        _ = std::process::Command::new("sh")
            .arg("-c")
            .arg("if bluetoothctl show | grep -q \"Powered: yes\"; then bluetoothctl power off; else bluetoothctl power on; fi")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("Failed to toggle bluetooth");
    }
}

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("com.jackson.control_center")
        .build();

    app.connect_startup(|_| load_css());
    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Control Center")
            .default_width(400)
            .default_height(200)
            .build();

        window.init_layer_shell();
        window.set_layer(gtk4_layer_shell::Layer::Overlay);

        let controller = gtk::EventControllerKey::new();
        controller.connect_key_pressed(|_, key, _, _| {
            match key {
                gdk::Key::Escape => {
                    println!("Escape pressed! Exiting...");
                    std::process::exit(0);
                }
                _ => {}
            }

            glib::Propagation::Proceed
        });
        window.add_controller(controller);

        let hyprsunset_button = init_toggle_button::<HyprsunsetUtils>("hyprsunset");
        let wifi_button = init_toggle_button::<WifiUtils>("wifi");
        let bluetooth_button = init_toggle_button::<BluetoothUtils>("bluetooth");

        let layout = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        layout.append(&hyprsunset_button);
        layout.append(&wifi_button);
        layout.append(&bluetooth_button);

        window.set_child(Some(&layout));

        window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
        window.set_anchor(gtk4_layer_shell::Edge::Left, true);
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);

        window.present();
    });

    app.run()
}

fn init_toggle_button<T: ToggleUtil>(label: &str) -> gtk::Button {
    let button = gtk::Button::builder()
        .label(label)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    let active = T::is_enabled();
    if active {
        button.add_css_class(ACTIVE_CLASS);
    }
    let active = std::rc::Rc::new(std::cell::Cell::new(active));

    button.set_size_request(50, 25);
    button.connect_clicked(glib::clone!(
        #[strong]
        button,
        #[strong]
        active,
        move |_| {
            active.set(!active.get());
            if active.get() {
                button.add_css_class(ACTIVE_CLASS);
            } else {
                button.remove_css_class(ACTIVE_CLASS);
            }
            T::toggle();
            T::begin_timeout_check(button.clone(), active.clone());
        }
    ));

    button
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
