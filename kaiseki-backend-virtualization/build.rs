use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn sdk_path(target: &str) -> Result<String, std::io::Error> {
    use std::process::Command;
    let sdk = if vec!["x86_64-apple-darwin", "aarch64-apple-darwin"].contains(&target) {
        "macosx12.1"
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
        // time.h as has a variable called timezone that conflicts with some of the objective-c
        // calls from NSCalendar.h in the Foundation framework. This removes that one variable.
        .blocklist_item("timezone")
        .blocklist_type("id")
        .blocklist_type("NSImage_")
        .blocklist_type("NSScreen_")
        .opaque_type("FndrOpaqueInfo")
        .opaque_type("HFSCatalogFolder")
        .opaque_type("HFSPlusCatalogFolder")
        .opaque_type("HFSCatalogFile")
        .opaque_type("HFSPlusCatalogFile")
        .header_contents(
            "Virtualization.h",
            "#include<Virtualization/Virtualization.h>",
        );

    // Generate the bindings.
    let bindings = builder
        .generate()
        .expect("unable to generate bindings")
        .to_string();

    let bindings = bindings
        .replace("unsafe fn setNeedsDisplayInRect_(&self, invalidRect: NSRect)","unsafe fn setNeedsDisplayInRect_(&self, invalidRect_: NSRect)")
        .replace("msg_send!(*self, setNeedsDisplayInRect: invalidRect)","msg_send!(*self, setNeedsDisplayInRect: invalidRect_)")
        .replace("unsafe fn setShadow_(&self, shadow: NSShadow)", "unsafe fn setShadow_(&self, shadow_: NSShadow)")
        .replace("msg_send!(*self, setShadow: shadow)","msg_send!(*self, setShadow: shadow_)")
        .replace("unsafe fn blendedColorWithFraction_ofColor_(&self, fraction: CGFloat, color: NSColor) -> NSColor","unsafe fn blendedColorWithFraction_ofColor_(&self, fraction_: CGFloat, color: NSColor) -> NSColor")
        .replace("msg_send ! (* self , blendedColorWithFraction : fraction ofColor : color)","msg_send ! (* self , blendedColorWithFraction : fraction_ ofColor : color)")
        .replace("unsafe fn selectColumnIndexes_byExtendingSelection_(&self, indexes: NSIndexSet, extend: BOOL)","unsafe fn selectColumnIndexes_byExtendingSelection_(&self, indexes: NSIndexSet, extend_: BOOL)")
        .replace("msg_send ! (* self , selectColumnIndexes : indexes byExtendingSelection : extend)","msg_send ! (* self , selectColumnIndexes : indexes byExtendingSelection : extend_)")
        .replace("unsafe fn selectRowIndexes_byExtendingSelection_(&self, indexes: NSIndexSet, extend: BOOL)","unsafe fn selectRowIndexes_byExtendingSelection_(&self, indexes: NSIndexSet, extend_: BOOL)")
        .replace("msg_send ! (* self , selectRowIndexes : indexes byExtendingSelection : extend)","msg_send ! (* self , selectRowIndexes : indexes byExtendingSelection : extend_)")
        .replace("unsafe fn selectColumn_byExtendingSelection_(&self, column: NSInteger, extend: BOOL)","unsafe fn selectColumn_byExtendingSelection_(&self, column: NSInteger, extend_: BOOL)")
        .replace("msg_send ! (* self , selectColumn : column byExtendingSelection : extend)","msg_send ! (* self , selectColumn : column byExtendingSelection : extend_)")
        .replace("unsafe fn selectRow_byExtendingSelection_(&self, row: NSInteger, extend: BOOL)","unsafe fn selectRow_byExtendingSelection_(&self, row: NSInteger, extend_: BOOL)")
        .replace("msg_send ! (* self , selectRow : row byExtendingSelection : extend)","msg_send ! (* self , selectRow : row byExtendingSelection : extend_)");

    // Get the cargo out directory.
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("env variable OUT_DIR not found"));
    let mut file = fs::File::create(out_dir.join("virtualization_bindings.rs"))
        .expect("could not open bindings file");

    // Write them to the crate root.
    file.write_all(bindings.as_bytes())
        .expect("could not write to bindings file");
    file.flush()
        .expect("could not flush contents to bindings file");
}

fn main() {
    let target = std::env::var("TARGET").unwrap();
    if !target.contains("apple-darwin") {
        panic!(
            "kaiseki-backend-virtualization requires the *-apple-darwin target (target is {})",
            target
        );
    }

    let directory = sdk_path(&target).ok();
    build(directory.as_ref().map(String::as_ref), &target);
}
