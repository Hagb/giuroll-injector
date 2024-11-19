# Giuroll DLL Injector

A simple injector for [giuroll](https://github.com/Giufinn/giuroll) and [giuroll-hagb](https://github.com/hagb/giuroll-hagb), modified from [dll injector by @kimjongbing](https://github.com/kimjongbing/dll_injector), not detected as virus by Windows Defender.
It applies Giuroll to Hisoutensoku without requiring `SWRSToys`, and works akin to the legacy `SokurollLoader.exe` used in older copies of the game.

## Build Commands
```
rustup install nightly
rustup +nightly component add rust-src
cargo +nightly build --target i686-win7-windows-msvc -Z build-std
```

## Usage
1. Ensure that `injector.exe`, `giuroll.dll` and `giuroll.ini` are all in the same directory,
2. Start `th123.exe`,
3. Run `injector.exe`

If successful, the console window will display the words `Injection successful` for a few seconds before closing, and the title of the game window will include `Giuroll <version_number>` at the end.

If the injector closes abruptly, contact me about it.