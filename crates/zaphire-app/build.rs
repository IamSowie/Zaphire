// Embed the icon and version info into zaphire.exe on Windows.
fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/zaphire.ico");
        res.set("ProductName", "Zaphire");
        res.set("FileDescription", "Zaphire");
        if let Err(e) = res.compile() {
            println!("cargo::warning=could not embed the Windows icon: {e}");
        }
    }
    println!("cargo::rerun-if-changed=../../assets/zaphire.ico");
}
