// build.rs

fn main() {
    println!("{}\n{}\n",
            "cargo:rustc-link-lib=dylib=curl",
            "cargo:rustc-link-search=native=/Software/Curl_lib/lib");
}
