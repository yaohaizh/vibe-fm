fn main() {
    // Only run on Windows
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/vibe.ico");
        res.set("ProductName", "Vibe File Manager");
        res.set("FileDescription", "A modern dual-pane file manager");
        res.set("LegalCopyright", "Copyright 2024");

        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }
}
