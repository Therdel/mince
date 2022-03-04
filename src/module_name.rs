pub enum ModuleName {
    Client
}

impl ModuleName {
    pub fn file_name(&self) -> &'static str {
        match self {
            ModuleName::Client => "client.dll"
        }
    }
}