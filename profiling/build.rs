use std::collections::HashSet;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let php_config_includes_output = Command::new("php-config")
        .arg("--includes")
        .output()
        .expect("Unable to run `php-config`. Is it in your PATH?");

    if !php_config_includes_output.status.success() {
        match String::from_utf8(php_config_includes_output.stderr) {
            Ok(stderr) => panic!("`php-config failed: {}", stderr),
            Err(err) => panic!(
                "`php-config` failed, not utf8: {}",
                String::from_utf8_lossy(err.as_bytes())
            ),
        }
    }

    let php_config_includes = std::str::from_utf8(php_config_includes_output.stdout.as_slice())
        .expect("`php-config`'s stdout to be valid utf8");

    generate_bindings(php_config_includes);
    build_zend_php_ffis(php_config_includes);

    cfg_php_major_version();
    cfg_zts();
}

fn build_zend_php_ffis(php_config_includes: &str) {
    println!("cargo:rerun-if-changed=src/php_ffi.h");
    println!("cargo:rerun-if-changed=src/php_ffi.c");

    let output = Command::new("php-config")
        .arg("--prefix")
        .output()
        .expect("Unable to run `php-config`. Is it in your PATH?");

    let prefix = String::from_utf8(output.stdout).expect("only utf8 chars work");
    println!("cargo:rustc-link-search=native={}/lib", prefix.trim());

    cc::Build::new()
        .files(["src/php_ffi.c"])
        .includes(
            str::replace(php_config_includes, "-I", "")
                .split(' ')
                .map(Path::new),
        )
        .flag_if_supported("-fuse-ld=lld")
        .flag_if_supported("-std=c11")
        .flag_if_supported("-std=c17")
        .compile("php_ffi");
}
#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}

fn generate_bindings(php_config_includes: &str) {
    println!("cargo:rerun-if-changed=src/php_ffi.h");
    let ignored_macros = IgnoreMacros(
        [
            "FP_INFINITE".into(),
            "FP_NAN".into(),
            "FP_NORMAL".into(),
            "FP_SUBNORMAL".into(),
            "FP_ZERO".into(),
            "IPPORT_RESERVED".into(),
        ]
        .into_iter()
        .collect(),
    );

    let bindings = bindgen::Builder::default()
        .ctypes_prefix("libc")
        .raw_line("extern crate libc;")
        .header("src/php_ffi.h")
        // Block some zend items that we'll provide manual definitions for
        .blocklist_item("sapi_getenv")
        .blocklist_item("zend_bool")
        .blocklist_item("_zend_extension")
        .blocklist_item("zend_extension")
        .blocklist_item("_zend_module_entry")
        .blocklist_item("zend_module_entry")
        .blocklist_item("zend_result")
        .blocklist_item("zend_register_extension")
        // Block our own globals
        .blocklist_item("zend_datadog_php_profiling_globals")
        .blocklist_item("datadog_php_profiling_globals_get")
        // I had to block these for some reason *shrug*
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_INT_DOWNWARD")
        .blocklist_item("FP_INT_TONEAREST")
        .blocklist_item("FP_INT_TONEARESTFROMZERO")
        .blocklist_item("FP_INT_TOWARDZERO")
        .blocklist_item("FP_INT_UPWARD")
        .blocklist_item("FP_NAN")
        .blocklist_item("FP_NORMAL")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_ZERO")
        .blocklist_item("IPPORT_RESERVED")
        .rustified_enum("datadog_php_profiling_log_level")
        .parse_callbacks(Box::new(ignored_macros))
        .clang_args(php_config_includes.split(' '))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustfmt_bindings(true)
        .layout_tests(true)
        .generate()
        .expect("bindgen to succeed");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("bindings to be written successfully");
}

fn cfg_php_major_version() {
    let output = Command::new("php")
        .arg("-r")
        .arg("echo PHP_MAJOR_VERSION, PHP_EOL;")
        .output()
        .expect("Unable to run `php`. Is it in your PATH?");

    if !output.status.success() {
        match String::from_utf8(output.stderr) {
            Ok(stderr) => panic!("`php failed: {}", stderr),
            Err(err) => panic!(
                "`php` failed, not utf8: {}",
                String::from_utf8_lossy(err.as_bytes())
            ),
        }
    }

    let version =
        std::str::from_utf8(output.stdout.as_slice()).expect("`php`'s stdout to be valid utf8");

    // Clean up surrounding whitespace for both printing and parsing.
    let version = version.trim();

    let parsed_version: u8 = version.parse().expect("version string to fit in u8");

    if parsed_version == 7 || parsed_version == 8 {
        println!("cargo:rustc-cfg=php{}", parsed_version);
    } else {
        panic!("Unidentified major PHP version: {}", version);
    }
}

fn cfg_zts() {
    let output = Command::new("php")
        .arg("-r")
        .arg("echo PHP_ZTS, PHP_EOL;")
        .output()
        .expect("Unable to run `php`. Is it in your PATH?");

    if !output.status.success() {
        match String::from_utf8(output.stderr) {
            Ok(stderr) => panic!("`php failed: {}", stderr),
            Err(err) => panic!("`php` failed, not utf8: {}", err),
        }
    }

    let zts =
        std::str::from_utf8(output.stdout.as_slice()).expect("`php`'s stdout to be valid utf8");

    if zts.contains('1') {
        println!("cargo:rustc-cfg=php_zts");
    }
}
