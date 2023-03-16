// build.rs

fn main() {
    println!("cargo:rustc-link-lib=dylib=curl");
    println!("cargo:rustc-link-lib=dylib=ssl");
    println!("cargo:rustc-link-search=native=/Software/Krecik/lib");
}
