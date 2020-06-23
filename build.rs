// build.rs

fn main() {
    println!(
        "cargo:rustc-link-lib=dylib=curl\ncargo:rustc-link-search=native=/Software/Curl_lib/lib\n"
    );
}
