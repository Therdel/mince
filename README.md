# mince
Rust gamehacking experiment for Counter-Strike: Source on Windows

### Features
1. Bunnyhop
2. Menu
3. Ejection - Clean teardown, restores original game code

### Techniques used
- Code injection
- Signature scanning
- Hooking

## Prerequisites
1. **Submodules**
    this repo contains submodules. Initialize them with
    ```console
    ~$ git submodule init
    ~$ git submodule update --recursive
    ```

2. **Keystone**: Assembly engine
    - the project depends on (and ships) the keystone assembler engine
    - this entails building the keystone library from source, which needs an installed CMake and C/C++ compiler (e.g. Visual Studio)
    - problem: the build tries to create a symlink from inside `lib/keystone/bindings/rust/keystone-sys` to the keystone root
    - under windows this is a [priviledged action](https://doc.rust-lang.org/std/os/windows/fs/fn.symlink_dir.html#limitations) and is part of the build process
    - it has do be done once, and be done with elevated rights
    - I recommend doing a cargo check from an elevated terminal _once_, which builds all dependencies without the project itself, creating said symlink correctly
        ```console
        (from an elevated terminal)
        ~$ cargo check
        ```

3. **Cross-compilation targets**
    Counter-Strike: Source is a 32 bit game, so cross compilation is needed
    ```console
    ~$ rustup target add i686-pc-windows-msvc
    ```
4. **Build library**
    ```console
    ~$ cargo build
    ```

## Getting Started
1. **do prerequisites**
2. **start the game**
    - start game, create a local server (unsecured), join a team
2. **inject library**
    - inject with an injector like [WINJECT 1.7](https://www.oldschoolhack.me/downloads/tools/3610-winject-17), the library is found at
    `target/i686-pc-windows-msvc/debug/mince.dll`