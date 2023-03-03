mod native;
use anyhow::Result;
use region::Region;

pub struct ModuleMap {
    native_module_map: native::ModuleMap,
}

impl ModuleMap {
    fn snapshot() -> Result<ModuleMap> {
        Ok(Self {
            native_module_map: native::ModuleMap::snapshot()?,
        })
    }
}

pub struct Modules {
    native_modules: native::Modules,
}

impl Iterator for Modules {
    type Item = Module;
    fn next(&mut self) -> Option<Self::Item> {
        self.native_modules.next()
            .map(|native_module| Module {native_module})
    }
}

impl IntoIterator for ModuleMap {
    type Item = Module;

    type IntoIter = Modules;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            native_modules: self.native_module_map.into_iter()
        }
    }
}

pub struct Module {
    native_module: native::Module,
}

impl Module {
    pub fn base(&self) -> *const u8 {
        self.native_module.base()
    }
    pub fn file_name(&self) -> &str {
        self.native_module.file_name()
    }
    pub fn regions_snapshot(&self) -> Regions {
        Regions { native_regions: self.native_module.regions_snapshot() }
    }
}

pub struct Regions {
    native_regions: native::Regions,
}

impl Iterator for Regions {
    type Item = Region;

    fn next(&mut self) -> Option<Self::Item> {
        self.native_regions.next()
    }
}

pub fn find_module<P>(predicate: P) -> Result<Option<Module>>
where P: Fn(&Module)->bool {
    let module = ModuleMap::snapshot()?.into_iter()
        .find(predicate);
    Ok(module)
}
