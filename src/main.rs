use gtk::prelude::*;
use gtk::{gdk, gio, glib};
use gtk4 as gtk;
use gtk4_layer_shell::LayerShell;
use std::{cell::Cell, rc::Rc};

use crate::constants::css_classes;

mod constants;
mod ui;
mod utils;

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("com.jackson.control_center")
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    app.connect_startup(|_| utils::load_css());

    app.connect_command_line(|app, cmd_line| {
        let args: Vec<_> = cmd_line.arguments();

        let wants_to_close = args.iter().any(|arg| arg == "--close");

        let window = match app.windows().first() {
            Some(win) => win.clone().downcast::<gtk::ApplicationWindow>().unwrap(),
            None => init_window(app),
        };

        window.present();

        if wants_to_close {
            window.set_visible(false);
        }

        glib::ExitCode::SUCCESS
    });

    app.connect_activate(|app| {
        init_window(app);
    });

    app.run()
}

fn init_window(app: &gtk::Application) -> gtk::ApplicationWindow {
    _ = app.hold();

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Control Center")
        .css_classes([css_classes::OVERLAY_ROOT])
        .build();

    window.init_layer_shell();
    window.set_layer(gtk4_layer_shell::Layer::Overlay);

    let wifi_button = init_toggle_button::<ui::WifiUtils>("󰤥", "wifi-btn");
    let bluetooth_button = init_toggle_button::<ui::BluetoothUtils>("󰂯", "bluetooth-btn");
    let hyprsunset_button = init_toggle_button::<ui::HyprsunsetUtils>("", "hyprsunset-btn");

    let fill = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Fill)
        .valign(gtk::Align::Fill)
        .hexpand(true)
        .vexpand(true)
        .css_classes([css_classes::OVERLAY_FILL])
        .build();

    let toggle_buttons = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(constants::BTN_GAP)
        .hexpand(true)
        .css_classes([css_classes::TOGGLE_BUTTONS])
        .build();
    append_expanded_btns_to_box(
        &toggle_buttons,
        vec![&wifi_button, &bluetooth_button, &hyprsunset_button],
    );

    let lock_button = init_cmd_button::<ui::LockUtil>("󰍁", "lock-btn");
    let sleep_button = init_cmd_button::<ui::SleepUtil>("󰤄", "sleep-btn");
    let log_out_button = init_cmd_button::<ui::LogOutUtil>("󰗼", "logout-btn");
    let reboot_button = init_cmd_button::<ui::RebootUtil>("󰜉", "reboot-btn");
    let power_off_button = init_cmd_button::<ui::PowerOffUtil>("󰤆", "power-off-btn");

    let cmd_buttons = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(constants::BTN_GAP)
        .hexpand(true)
        .css_classes([css_classes::CMD_BUTTONS])
        .build();
    append_expanded_btns_to_box(
        &cmd_buttons,
        vec![
            &lock_button,
            &sleep_button,
            &log_out_button,
            &reboot_button,
            &power_off_button,
        ],
    );

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(constants::BTN_GAP)
        .halign(gtk::Align::Start)
        .valign(gtk::Align::End)
        .vexpand(true)
        .width_request(constants::CONTENT_WIDTH)
        .css_classes([css_classes::CONTENT])
        .build();
    content.append(&toggle_buttons);
    content.append(&cmd_buttons);
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
            if let Some(bounds) = content.compute_bounds(&window)
                && !bounds.contains_point(&gtk::graphene::Point::new(x as f32, y as f32))
            {
                handle_close_window(&window);
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
                handle_close_window(&window);
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        }
    ));
    window.add_controller(key);

    window
}

fn append_expanded_btns_to_box(box_layout: &gtk::Box, btns: Vec<&gtk::Button>) {
    if btns.is_empty() {
        return;
    }

    box_layout.set_homogeneous(true);

    for btn in btns {
        btn.set_hexpand(true);
        box_layout.append(btn);
    }
}

fn init_toggle_button<T: ui::ToggleUtil>(
    label: &'static str,
    class_name: &'static str,
) -> gtk::Button {
    let button = gtk::Button::builder()
        .label(label)
        .valign(gtk::Align::Center)
        .build();

    button.add_css_class(class_name);

    let active = T::is_enabled();
    if active {
        button.add_css_class(constants::ACTIVE_CLASS);
    }
    let active = Rc::new(Cell::new(active));

    button.connect_clicked(glib::clone!(
        #[strong]
        active,
        move |button| {
            active.set(!active.get());
            if active.get() {
                button.add_css_class(constants::ACTIVE_CLASS);
            } else {
                button.remove_css_class(constants::ACTIVE_CLASS);
            }
            T::toggle();
            T::begin_timeout_check(button.clone(), active.clone());
        }
    ));

    button
}

fn init_cmd_button<T: ui::CmdUtil>(label: &'static str, class_name: &'static str) -> gtk::Button {
    let button = gtk::Button::builder()
        .label(label)
        .valign(gtk::Align::Center)
        .css_classes([class_name])
        .build();

    button.connect_clicked(|_| {
        T::run_cmd();
    });

    button
}

fn handle_close_window(window: &gtk::ApplicationWindow) {
    window.set_visible(false);
}
