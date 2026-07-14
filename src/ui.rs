use gtk::{gio, glib};
use gtk4::{self as gtk, prelude::WidgetExt};
use std::{cell::Cell, rc::Rc};

use crate::{constants, utils};

pub trait ToggleUtil {
    fn toggle();
    fn is_enabled() -> bool;

    fn matches_stdout(stdout: Vec<u8>, success_str: &'static str) -> bool {
        let output = String::from_utf8(stdout).unwrap();
        output.trim() == success_str
    }

    fn begin_timeout_check(button: gtk::Button, active: Rc<Cell<bool>>) {
        glib::MainContext::default().spawn_local(async move {
            glib::timeout_future_seconds(1).await;
            if Self::is_enabled() {
                button.add_css_class(constants::ACTIVE_CLASS);
                active.set(true);
            } else {
                button.remove_css_class(constants::ACTIVE_CLASS);
                active.set(false);
            }
        });
    }
}

pub struct HyprsunsetUtils;
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
        _ = gio::Subprocess::newv(
            &["sh".as_ref(), "-c".as_ref(), "if pgrep -x hyprsunset >/dev/null; then pkill -INT hyprsunset; else setsid -f hyprsunset --temperature 4000 >/dev/null 2>&1; fi".as_ref()],
            gio::SubprocessFlags::STDOUT_SILENCE | gio::SubprocessFlags::STDERR_SILENCE ,
        );
    }
}

pub struct WifiUtils;
impl ToggleUtil for WifiUtils {
    fn is_enabled() -> bool {
        const SUCCESS_CASE: &str = "enabled";
        let output = std::process::Command::new("nmcli")
            .arg("radio")
            .arg("wifi")
            .output()
            .expect("Failed to check on wifi");
        Self::matches_stdout(output.stdout, SUCCESS_CASE)
    }

    fn toggle() {
        _ = gio::Subprocess::newv(
            &["sh".as_ref(), "-c".as_ref(), "[[ $(nmcli radio wifi) == \"enabled\" ]] && nmcli radio wifi off || nmcli radio wifi on".as_ref()],
            gio::SubprocessFlags::STDOUT_SILENCE | gio::SubprocessFlags::STDERR_SILENCE,
        );
    }
}

pub struct BluetoothUtils;
impl ToggleUtil for BluetoothUtils {
    fn is_enabled() -> bool {
        let mut cmd1 = std::process::Command::new("bluetoothctl")
            .arg("show")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to check bluetooth");

        let cmd1_stdout = cmd1.stdout.take().expect("Failed to open bluetooth stdout");

        let cmd2 = std::process::Command::new("grep")
            .arg("-q")
            .arg("Powered: yes")
            .stdin(std::process::Stdio::from(cmd1_stdout))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .expect("Failed to search bluetooth");

        _ = cmd1.wait();
        cmd2.status.success()
    }

    fn toggle() {
        _ = gio::Subprocess::newv(
            &["sh".as_ref(), "-c".as_ref(), "if bluetoothctl show | grep -q \"Powered: yes\"; then bluetoothctl power off; else bluetoothctl power on; fi".as_ref()],
            gio::SubprocessFlags::STDOUT_SILENCE | gio::SubprocessFlags::STDERR_SILENCE,
        );
    }
}

pub trait CmdUtil {
    fn run_cmd();
}

pub struct LockUtil;
impl CmdUtil for LockUtil {
    fn run_cmd() {
        _ = std::process::Command::new("loginctl")
            .arg("lock-session")
            .spawn()
            .expect("Failed to lock session")
            .wait();
    }
}

pub struct SleepUtil;
impl CmdUtil for SleepUtil {
    fn run_cmd() {
        _ = std::process::Command::new("systemctl")
            .arg("suspend")
            .spawn()
            .expect("Failed to sleep")
            .wait();
    }
}

pub struct LogOutUtil;
impl CmdUtil for LogOutUtil {
    fn run_cmd() {
        _ = std::process::Command::new("hyprshutdown")
            .spawn()
            .expect("Failed to log out")
            .wait();
    }
}

pub struct RebootUtil;
impl CmdUtil for RebootUtil {
    fn run_cmd() {
        _ = std::process::Command::new("systemctl")
            .arg("reboot")
            .spawn()
            .expect("Failed to reboot")
            .wait();
    }
}

pub struct PowerOffUtil;
impl CmdUtil for PowerOffUtil {
    fn run_cmd() {
        let status = std::process::Command::new("hyprshutdown")
            .status()
            .expect("Hyprshutdown failed");

        if status.success() {
            _ = std::process::Command::new("systemctl")
                .arg("poweroff")
                .status()
                .expect("Failed to power off");
        }
    }
}

pub struct Wallpaper;
impl CmdUtil for Wallpaper {
    fn run_cmd() {
        let Some(mut home_dir) = utils::get_home_dir() else {
            return;
        };
        home_dir.push("scripts/bin/wui");
        _ = std::process::Command::new("sh")
            .arg("-c")
            .arg(home_dir.to_str().unwrap())
            .spawn()
            .expect("Failed to start wui");
    }
}
