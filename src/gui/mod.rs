pub mod message_box {
    pub fn info(message: impl AsRef<str>) {
        message_box(Type::Info, message.as_ref());
    }

    pub fn warn(message: impl AsRef<str>) {
        message_box(Type::Warn, message.as_ref());
    }

    pub fn yes_no(message: impl AsRef<str>) -> bool {
        message_box(Type::YesNo, message.as_ref())
    }

    enum Type {
        Info,
        Warn,
        YesNo,
    }

    #[cfg(target_os = "linux")]
    fn message_box(message_type: Type, message: &str) -> bool {
        let title = format!("--title=Mince");
        let type_string = match message_type {
            Type::Info => "--info",
            Type::Warn => "--warning",
            Type::YesNo => "--question",
        };

        // name labels "Yes" and "No"
        let (ok_label, cancel_label) = match message_type {
            Type::YesNo => ("--ok-label=Yes", "--cancel-label=No"),
            _ => ("", ""),
        };

        // source: https://unix.stackexchange.com/a/144925
        let output = std::process::Command::new("zenity")
            .args([
                title.as_str(),
                type_string,
                format!("--text={message}").as_str(),
                ok_label,
                cancel_label,
            ])
            .output()
            .expect("failed to execute zenity");

        match output.status.code().unwrap() {
            0 => true,
            _ => false,
        }
    }

    #[cfg(target_os = "windows")]
    fn message_box(message_type: Type, message: &str) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::{
            MessageBoxA, IDOK, IDYES, MB_ICONINFORMATION, MB_ICONQUESTION, MB_ICONWARNING, MB_OK,
            MB_YESNO,
        };

        let message_type = match message_type {
            Type::Info => MB_OK | MB_ICONINFORMATION,
            Type::Warn => MB_OK | MB_ICONWARNING,
            Type::YesNo => MB_YESNO | MB_ICONQUESTION,
        };

        // let params = match message_type
        let answer = unsafe { MessageBoxA(None, message, "Mince", message_type) };

        matches!(answer, IDYES | IDOK)
    }
}