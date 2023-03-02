mod native;
pub mod error;
use error::Result;

pub struct ModuleMapping {
    pub memory: *const [u8],
    pub file_name: String,

    #[cfg(feature = "expose_native_module_types")]
    pub native_module: native::NativeModuleMapping
}

fn iterate_mappings() -> Result<impl Iterator<Item=ModuleMapping>> {
    native::iterate_mappings()
}

pub fn find_module<P>(predicate: P) -> Result<Option<ModuleMapping>>
where P: Fn(&ModuleMapping)->bool {
    let module = iterate_mappings()?
        .find(predicate);
    Ok(module)
}
