fn main() {
    println!("cargo:rerun-if-changed=index.html");
    println!("cargo:rerun-if-changed=assets");

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed=sidecar.rc");
        embed_resource::compile("sidecar.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("LiteTUI sidecar Windows icon resource");
    }
}
