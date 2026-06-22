use gtk4::prelude::*;
use gtk4::{self as gtk, gdk};
use gtk4::{gio, glib};
use gtk4_layer_shell::LayerShell;

const ACTIVE_CLASS: &'static str = "active";
const CONTENT_WIDTH: i32 = 350;
const BTN_GAP: i32 = 12;

trait ToggleUtil {
    fn toggle();
    fn is_enabled() -> bool;

    fn matches_stdout(stdout: std::vec::Vec<u8>, success_str: &'static str) -> bool {
        let output = String::from_utf8(stdout).unwrap();
        output.trim() == success_str
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
    fn is_enabled() -> bool {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg("pgrep -x hyprsunset >/dev/null")
            .output()
            .expect("Failed to check on hyprsunset");
        output.status.success()
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
    fn is_enabled() -> bool {
        const SUCCESS_CASE: &'static str = "enabled";
        let output = std::process::Command::new("nmcli")
            .arg("radio")
            .arg("wifi")
            .output()
            .expect("Failed to check on wifi");
        Self::matches_stdout(output.stdout, SUCCESS_CASE)
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

trait CmdUtil {
    fn run_cmd();
}

struct LockUtil;
impl CmdUtil for LockUtil {
    fn run_cmd() {
        println!("running lock");
    }
}

struct SleepUtil;
impl CmdUtil for SleepUtil {
    fn run_cmd() {
        println!("running sleep");
    }
}

struct LogOutUtil;
impl CmdUtil for LogOutUtil {
    fn run_cmd() {
        println!("running log out");
    }
}

struct RebootUtil;
impl CmdUtil for RebootUtil {
    fn run_cmd() {
        println!("running reboot");
    }
}

struct PowerOffUtil;
impl CmdUtil for PowerOffUtil {
    fn run_cmd() {
        println!("running power off");
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
            .build();

        window.add_css_class("overlay-root");

        window.init_layer_shell();
        window.set_layer(gtk4_layer_shell::Layer::Overlay);

        let wifi_button = init_toggle_button::<WifiUtils>("󰤥");
        wifi_button.add_css_class("wifi-btn");
        let bluetooth_button = init_toggle_button::<BluetoothUtils>("󰂯");
        let hyprsunset_button = init_toggle_button::<HyprsunsetUtils>("");
        hyprsunset_button.add_css_class("hyprsunset-btn");

        let fill = gtk::Box::new(gtk::Orientation::Vertical, 0);
        fill.set_halign(gtk::Align::Fill);
        fill.set_valign(gtk::Align::Fill);
        fill.set_hexpand(true);
        fill.set_vexpand(true);

        let toggle_buttons = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(BTN_GAP)
            .hexpand(true)
            .build();
        append_expanded_btns_to_box(
            &toggle_buttons,
            vec![&wifi_button, &bluetooth_button, &hyprsunset_button],
        );
        toggle_buttons.add_css_class("toggle-buttons");

        let lock_button = init_cmd_button::<LockUtil>("󰍁");
        let sleep_button = init_cmd_button::<SleepUtil>("󰍁");
        let log_out_button = init_cmd_button::<LogOutUtil>("󰍁");
        let reboot_button = init_cmd_button::<RebootUtil>("󰍁");
        let poweroff_button = init_cmd_button::<PowerOffUtil>("󰍁");

        let cmd_buttons = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(BTN_GAP)
            .hexpand(true)
            .build();
        cmd_buttons.add_css_class("cmd-buttons");
        append_expanded_btns_to_box(
            &cmd_buttons,
            vec![
                &lock_button,
                &sleep_button,
                &log_out_button,
                &reboot_button,
                &poweroff_button,
            ],
        );

        let content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(BTN_GAP)
            .halign(gtk::Align::Start)
            .valign(gtk::Align::End)
            .vexpand(true)
            .build();
        content.append(&toggle_buttons);
        content.append(&cmd_buttons);
        content.add_css_class("content");
        fill.append(&content);

        window.set_child(Some(&fill));

        window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
        window.set_anchor(gtk4_layer_shell::Edge::Left, true);
        window.set_anchor(gtk4_layer_shell::Edge::Right, true);
        window.set_anchor(gtk4_layer_shell::Edge::Top, true);

        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);

        let click = gtk::GestureClick::new();
        click.set_propagation_phase(gtk::PropagationPhase::Capture);
        click.connect_pressed(glib::clone!(
            #[weak]
            window,
            #[weak]
            content,
            move |_, _, x, y| {
                if let Some(bounds) = content.compute_bounds(&window) {
                    if !bounds.contains_point(&gtk::graphene::Point::new(x as f32, y as f32)) {
                        window.close();
                    }
                }
            }
        ));
        window.add_controller(click);

        let key = gtk::EventControllerKey::new();
        key.connect_key_pressed(glib::clone!(
            #[weak]
            window,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, key, _, _| {
                if key == gdk::Key::Escape {
                    window.close();
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        ));
        window.add_controller(key);

        window.present();
    });

    app.run()
}

fn append_expanded_btns_to_box(box_layout: &gtk::Box, btns: std::vec::Vec<&gtk::Button>) {
    if btns.len() == 0 {
        return;
    }

    let btn_count: i32 = btns.len() as i32;
    let btn_width = (CONTENT_WIDTH / btn_count) - BTN_GAP;
    for btn in btns {
        btn.set_width_request(btn_width);
        box_layout.append(btn);
    }
}

fn init_toggle_button<T: ToggleUtil>(label: &'static str) -> gtk::Button {
    let button = gtk::Button::builder().label(label).build();

    let active = T::is_enabled();
    if active {
        button.add_css_class(ACTIVE_CLASS);
    }
    let active = std::rc::Rc::new(std::cell::Cell::new(active));

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

fn init_cmd_button<T: CmdUtil>(label: &'static str) -> gtk::Button {
    let button = gtk::Button::builder().label(label).build();

    button.connect_clicked(|_| {
        T::run_cmd();
    });

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
