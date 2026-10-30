fn main() {
    // The exe's embedded icon comes from icons/icon.ico; rebuild when it is regenerated.
    println!("cargo:rerun-if-changed=icons");
    tauri_build::build()
}
