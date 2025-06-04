#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("icon.ico")
        .set("ProductName", "Password Manager")
        .set("FileDescription", "Secure Password Manager")
        .set("LegalCopyright", "Copyright (c) 2025")
        .set("ProductVersion", "1.0.0")
        .set("FileVersion", "1.0.0");
    
    if let Err(e) = res.compile() {
        eprintln!("Warning: Could not compile Windows resources: {}", e);
    }
}

#[cfg(not(windows))]
fn main() {
}