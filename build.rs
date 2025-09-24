fn main() {
    // CUDA FFI bindings
    #[cfg(feature = "nvenc")]
    println!("cargo:rustc-link-lib=dylib=cuda");
    #[cfg(feature = "nvenc")]
    println!("cargo:rustc-link-search=native=/usr/lib");
}
