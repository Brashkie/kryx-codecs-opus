//! build.rs — Smart orchestration for libopus + Rust FFI (M2)
//!
//! Responsibilities:
//!   1. Check that Zig is installed (fail with clear message if not).
//!   2. Detect if libopus.a is already built (skip zig build if so).
//!   3. Invoke `zig build` from `../../zig/` directory.
//!   4. Tell cargo to link libopus statically + the platform C runtime.
//!   5. Rebuild triggers: .zig, .c, and .h changes.
//!
//! This runs before compiling opus-core. The user only needs to run:
//!   npm run build:native

use std::env;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

fn main() {
    let workspace_root = workspace_root();
    let zig_dir = workspace_root.join("zig");
    let vendor_dir = workspace_root.join("vendor").join("libopus");

    // ─── 1. Sanity check: vendor/libopus/ must exist ─────────────────────
    if !vendor_dir.join("include").join("opus.h").exists() {
        fail(&format!(
            "libopus sources not found at:\n    {}\n\n\
             Expected file: vendor/libopus/include/opus.h\n\
             This should have been vendored as part of M1.",
            vendor_dir.display()
        ));
    }

    // ─── 2. Check Zig is installed ────────────────────────────────────────
    if !zig_available() {
        fail(
            "Zig 0.14.x or newer is required to build @kryxjs/codecs-opus \
             but was not found in PATH.\n\n\
             Please install Zig: https://ziglang.org/download/\n\n\
             On Windows:\n\
             \x20 1. Download zig-windows-x86_64-0.14.1.zip\n\
             \x20 2. Extract to C:\\Tools\\zig\n\
             \x20 3. Add C:\\Tools\\zig to your PATH\n\
             \x20 4. Restart your terminal\n\
             \x20 5. Verify with: zig version\n\n\
             On macOS:  brew install zig\n\
             On Linux:  see https://ziglang.org/download/",
        );
    }

    // ─── 3. Determine expected artifact path ──────────────────────────────
    let zig_out = zig_dir.join("zig-out").join("lib");
    let (libopus_static, link_name) = expected_artifact(&zig_out);

    // ─── 4. Invoke `zig build` if libopus.a is stale or missing ───────────
    let needs_rebuild = should_rebuild(&libopus_static, &vendor_dir);
    if needs_rebuild {
        // Print to stderr as an informational line (not a cargo:warning,
        // which napi surfaces noisily). Only shown on the first build.
        eprintln!("[opus-core] Building libopus with Zig (first build, ~1 min)...");
        run_zig_build(&zig_dir);
    }
    // When reusing the cache we stay silent to keep build output clean.

    // ─── 5. Verify the artifact was produced ──────────────────────────────
    if !libopus_static.exists() {
        fail(&format!(
            "Expected libopus artifact was not produced:\n    {}\n\n\
             This usually means `zig build` failed. Check the output above.\n\
             You can also run manually:  cd zig && zig build",
            libopus_static.display()
        ));
    }

    // ─── 6. Tell cargo how to link ────────────────────────────────────────
    println!("cargo:rustc-link-search=native={}", zig_out.display());
    println!("cargo:rustc-link-lib=static={}", link_name);

    // Platform C runtime linking so libopus's fprintf/stderr (used by
    // celt_fatal) resolves at the final link of the .node addon.
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    if target_os == "windows" && target_env == "msvc" {
        // MSVC C runtime: legacy_stdio_definitions.lib provides fprintf and
        // friends for objects compiled against the modern UCRT. It ships with
        // the MSVC toolchain and is found on the default lib search path, so
        // we reference it by its bare name (Rust appends .lib automatically).
        // This resolves the `fprintf` symbol pulled in by celt_fatal.
        println!("cargo:rustc-link-lib=legacy_stdio_definitions");
    } else if target_os == "linux" {
        println!("cargo:rustc-link-lib=dylib=m");
        println!("cargo:rustc-link-lib=dylib=c");
    } else if target_os == "macos" {
        println!("cargo:rustc-link-lib=dylib=m");
    }

    // ─── 7. Rebuild triggers ──────────────────────────────────────────────
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../zig/build.zig");
    println!("cargo:rerun-if-changed=../../zig/src");
    println!("cargo:rerun-if-changed=../../vendor/libopus/include");
}

// ═══════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════

fn fail(msg: &str) -> ! {
    eprintln!();
    eprintln!("═══════════════════════════════════════════════════════════════");
    for line in msg.lines() {
        eprintln!("  {line}");
    }
    eprintln!("═══════════════════════════════════════════════════════════════");
    eprintln!();
    exit(1);
}

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .expect("Could not determine workspace root")
        .to_path_buf()
}

fn zig_available() -> bool {
    Command::new("zig")
        .arg("version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

fn expected_artifact(zig_out_lib: &Path) -> (PathBuf, &'static str) {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    let filename = if target_os == "windows" && target_env == "msvc" {
        "opus.lib"
    } else {
        "libopus.a"
    };
    (zig_out_lib.join(filename), "opus")
}

fn should_rebuild(artifact: &Path, vendor: &Path) -> bool {
    if !artifact.exists() {
        return true;
    }
    let artifact_mtime = match std::fs::metadata(artifact).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return true,
    };
    let representative = [
        vendor.join("src").join("opus_encoder.c"),
        vendor.join("include").join("opus.h"),
    ];
    for path in representative {
        if let Ok(meta) = std::fs::metadata(&path) {
            if let Ok(mtime) = meta.modified() {
                if mtime > artifact_mtime {
                    return true;
                }
            }
        }
    }
    false
}

fn run_zig_build(zig_dir: &Path) {
    let optimize = if env::var("PROFILE").as_deref() == Ok("release") {
        "ReleaseFast"
    } else {
        "Debug"
    };

    let status = Command::new("zig")
        .arg("build")
        .arg(format!("-Doptimize={}", optimize))
        .current_dir(zig_dir)
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => fail(&format!(
            "`zig build` exited with status: {s}\n\
             See the output above for compilation errors."
        )),
        Err(e) => fail(&format!(
            "Could not execute `zig build`: {e}\n\
             Even though `zig version` succeeded, running `zig build` failed."
        )),
    }
}
