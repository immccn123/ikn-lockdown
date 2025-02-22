extern crate winapi;
extern crate winres;

#[cfg(all(target_os = "windows"))]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_manifest(include_str!("./app.manifest"))
        .set_icon("./assets/lockdown.ico");
    if let Err(error) = res.compile() {
        eprint!("{error}");
        std::process::exit(1);
    }
}
