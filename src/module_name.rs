pub enum ModuleName {
    Client
}

impl ModuleName {
    pub fn file_name(&self) -> &'static str {
        match self {
            #[cfg(target_os="windows")]
            ModuleName::Client => "client.dll",
            #[cfg(target_os="linux")]
            ModuleName::Client => "client.so"
        }
    }
}