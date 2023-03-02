# mince
Cross-platform rust gamehacking experiment

## Prerequisites
**Keystone**: Assembly engine
- builds the keystone library from source - needs installed CMake and C/C++ compiler (e.g. Visual Studio)
- tries to create a symlink from inside keystone/bindings/rust/keystone-sys to keystone root
    - under windows this is a [priviledged action](https://doc.rust-lang.org/std/os/windows/fs/fn.symlink_dir.html#limitations) and is part of the build process
    - it has do be done once, and be done with elevated rights. Recommendation: Execute ```cargo check``` once from an elevated terminal
