#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.set("ProductName", "NotchIsland");
    res.set("FileDescription", "NotchIsland - Dynamic Island for Windows");
    res.set("FileVersion", "0.1.0.0");
    res.set("ProductVersion", "0.1.0.0");
    res.set("LegalCopyright", "Copyright (c) 2026 Antigravity");
    let _ = res.compile();
}

#[cfg(not(windows))]
fn main() {}
