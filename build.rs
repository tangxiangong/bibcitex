#[cfg(windows)]
fn main() {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("icons/icon.ico");
    if let Err(e) = res.compile() {
        eprintln!("Failed to compile Windows resources: {}", e);
    }
}

#[cfg(not(windows))]
fn main() {}
