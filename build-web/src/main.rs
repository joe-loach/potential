use pico_args::Arguments;
use std::env;
use std::path::Path;
use std::process::Command;

const HELP: &str = "\
cargo run-wasm

USAGE:
  cargo run-wasm [OPTIONS] NAME

OPTIONS:
  --release                    Build in release mode, with optimizations
  --example                    Build and run the example NAME instead of a package NAME
  --features <FEATURES>...     Comma separated list of features to activate

NAME:
  Name of the package (crate) within the workspace to run.
";

struct Args {
    release: bool,
    example: bool,
    name: String,
    features: Option<String>,
}

impl Args {
    pub fn from_env() -> Result<Self, String> {
        let mut args = Arguments::from_env();
        let release = args.contains("--release");
        let example = args.contains("--example");

        let features: Option<String> = args.opt_value_from_str("--features").unwrap();

        let mut unused_args: Vec<String> = args
            .finish()
            .into_iter()
            .map(|x| x.into_string().unwrap())
            .collect();

        for unused_arg in &unused_args {
            if unused_arg.starts_with('-') {
                return Err(format!("Unknown option {}", unused_arg));
            }
        }

        match unused_args.len() {
            0 => Err("Expected NAME arg, but there was no NAME arg".to_string()),
            1 => Ok(Args {
                release,
                example,
                name: unused_args.remove(0),
                features,
            }),
            len => Err(format!(
                "Expected exactly one free arg, but there was {} free args: {:?}",
                len, unused_args
            )),
        }
    }
}

pub fn build_wasm() {
    let args = match Args::from_env() {
        Ok(args) => args,
        Err(err) => {
            println!("{}\n\n{}", err, HELP);
            return;
        }
    };
    let profile = if args.release { "release" } else { "debug" };

    // build wasm example via cargo
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let project_root = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf();
    let mut cargo_args = vec![
        "build",
        "--target",
        "wasm32-unknown-unknown",
        // It is common to setup a faster linker such as mold or lld to run for just your native target.
        // It cant be set for wasm as wasm doesnt support building with these linkers.
        // This results in a separate rustflags value for native and wasm builds.
        // Currently rust triggers a full rebuild every time the rustflags value changes.
        //
        // Therefore we have this hack where we use a different target dir for wasm builds to avoid constantly triggering full rebuilds.
        // When this issue is resolved we might be able to remove this hack: https://github.com/rust-lang/cargo/issues/8716
        "--target-dir",
        "target/wasm",
    ];
    if args.example {
        cargo_args.extend(["--example", &args.name]);
    } else {
        cargo_args.extend(["--package", &args.name]);
    }
    if let Some(features) = &args.features {
        cargo_args.extend(["--features", features]);
    }
    if args.release {
        cargo_args.push("--release");
    }
    let status = Command::new(&cargo)
        .current_dir(&project_root)
        .args(&cargo_args)
        .status()
        .unwrap();
    if !status.success() {
        // We can return without printing anything because cargo will have already displayed an appropriate error.
        return;
    }

    // run wasm-bindgen on wasm file output by cargo, write to the destination folder
    let target_profile = Path::new("target/wasm/wasm32-unknown-unknown").join(profile);
    let wasm_source = if args.example {
        target_profile.join("examples")
    } else {
        target_profile
    }
    .join(format!("{}.wasm", &args.name));

    let out_dest = project_root.join("docs");
    std::fs::create_dir_all(&out_dest).unwrap();
    let mut bindgen = wasm_bindgen_cli_support::Bindgen::new();
    bindgen
        .web(true)
        .unwrap()
        .omit_default_module_path(false)
        .input_path(&wasm_source)
        .generate(&out_dest)
        .unwrap();

    // process template index.html and write to the destination folder
    let index_template = include_str!("index.template.html");
    let index_processed = index_template.replace("{{name}}", &args.name);
    std::fs::write(out_dest.join("index.html"), index_processed).unwrap();
}

fn main() {
    build_wasm();
}
