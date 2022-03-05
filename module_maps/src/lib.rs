mod native;
pub mod error;

pub struct ModuleMapping {
    pub base: *mut u8,
    // pub byte_length: usize,
    pub file_name: String,

    #[cfg(feature = "expose_native_module_types")]
    pub native_module: native::NativeModuleMapping
}

fn iterate_mappings() -> error::Result<impl Iterator<Item=ModuleMapping>> {
    native::iterate_mappings()
}

pub fn find_module<P>(predicate: P) -> error::Result<Option<ModuleMapping>>
where P: Fn(&ModuleMapping)->bool {
    let module = iterate_mappings()?
        .find(predicate);
    Ok(module)
}
