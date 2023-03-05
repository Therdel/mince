#[derive(Clone, Copy)]
pub enum ModuleName {
    Client,
    Engine,
}

impl ModuleName {
    pub fn file_name(&self) -> &'static str {
        match self {
            #[cfg(target_os="windows")]
            ModuleName::Client => "client.dll",
            #[cfg(target_os="linux")]
            ModuleName::Client => "client.so",
            #[cfg(target_os="windows")]
            ModuleName::Engine => "engine.dll",
            #[cfg(target_os="linux")]
            ModuleName::Engine => "engine.so",
        }
    }
}