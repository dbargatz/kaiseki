use std::path::Path;

fn sdk_path(target: &str) -> Result<String, std::io::Error> {
    use std::process::Command;
    let sdk = if vec!["x86_64-apple-darwin", "aarch64-apple-darwin"].contains(&target) {
        "macosx12.3"
    } else {
        unreachable!();
    };

    let output = Command::new("xcrun")
        .args(&["--sdk", sdk, "--show-sdk-path"])
        .output()?
        .stdout;
    let prefix_str = std::str::from_utf8(&output).expect("invalid output from `xcrun`");
    Ok(prefix_str.trim_end().to_string())
}

fn build(sdk_path: Option<&str>, target: &str) {
    // Generate one large set of bindings for all frameworks.
    //
    // We do this rather than generating a module per framework as some frameworks depend on other
    // frameworks and in turn share types. To ensure all types are compatible across each
    // framework, we feed all headers to bindgen at once.
    //
    // Only link to each framework and include their headers if their features are enabled and they
    // are available on the target os.
    println!("cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=Virtualization");

    // See https://github.com/rust-lang/rust-bindgen/issues/1211
    // Technically according to the llvm mailing list, the argument to clang here should be
    // -arch arm64 but it looks cleaner to just change the target.
    let target = if target == "aarch64-apple-darwin" {
        "arm64-apple-darwin"
    } else {
        target
    };
    // Begin building the bindgen params.
    let mut builder = bindgen::Builder::default();

    let target_arg = format!("--target={}", target);
    let mut clang_args = vec!["-x", "objective-c", "-fblocks", &target_arg];
    if let Some(sdk_path) = sdk_path {
        clang_args.extend(&["-isysroot", sdk_path]);
    }

    builder = builder
        .clang_args(&clang_args)
        .layout_tests(true)
        .rustfmt_bindings(true)
        .allowlist_type("[I|P|]VZ.*")
        .allowlist_type("[I|P|]NSAccessibility")
        .allowlist_type("[I|P|]NSActionCell")
        .allowlist_type("[I|P|]NSCell")
        .allowlist_type("[I|P|]NSControl")
        .allowlist_type("[I|P|]NSError")
        .allowlist_type("[I|P|]NSMutableAttributedString")
        .allowlist_type("[I|P|]NSObject")
        .allowlist_type("[I|P|]NSPanel")
        .allowlist_type("[I|P|]NSResponder")
        .allowlist_type("[I|P|]NSString")
        .allowlist_type("[I|P|]NSURL")
        .allowlist_type("[I|P|]NSValue")
        .allowlist_type("[I|P|]NSView")
        .allowlist_type("NSString_NSStringExtensionMethods")
        .allowlist_type("VZVirtualMachineConfiguration_VZVirtualMachineConfigurationValidation")
        .allowlist_var("NSUTF8StringEncoding")
        .header_contents(
            "Virtualization.h",
            "#include<Virtualization/Virtualization.h>",
        );

    // Generate the bindings and write them to a file in the source hierarchy for easy debugging.
    // The generated bindings are in the .gitignore and are NOT checked in.
    let out_file = Path::new("src/bindings.rs");
    builder
        .generate()
        .expect("unable to generate bindings")
        .write_to_file(out_file)
        .expect("unable to write bindings to file");
}

fn main() {
    let target = std::env::var("TARGET").unwrap();
    if !target.contains("apple-darwin") {
        panic!(
            "virtualization-sys requires the *-apple-darwin target (target is {})",
            target
        );
    }

    let directory = sdk_path(&target).ok();
    build(directory.as_ref().map(String::as_ref), &target);
}
