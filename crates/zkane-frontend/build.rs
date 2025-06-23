//! Build script for ZKane frontend

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=index.html");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // Get the target directory
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir).ancestors().nth(3).unwrap();
    
    // Set up wasm-pack build for different profiles
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    
    let wasm_pack_args = match profile.as_str() {
        "release" => vec![
            "build",
            "--target", "web",
            "--out-dir", "pkg",
            "--release",
            "--no-typescript",
        ],
        _ => vec![
            "build", 
            "--target", "web",
            "--out-dir", "pkg",
            "--dev",
            "--no-typescript",
        ]
    };

    // Only run wasm-pack if we're building for wasm32
    if env::var("TARGET").unwrap_or_default().contains("wasm32") {
        println!("cargo:warning=Building WASM package with wasm-pack");
        
        let output = Command::new("wasm-pack")
            .args(&wasm_pack_args)
            .current_dir(env::var("CARGO_MANIFEST_DIR").unwrap())
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    println!("cargo:warning=wasm-pack failed: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                println!("cargo:warning=Failed to run wasm-pack: {}. Make sure wasm-pack is installed.", e);
            }
        }
    }

    // Copy static assets if they exist
    copy_static_assets();
}

fn copy_static_assets() {
    use std::fs;
    
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let static_dir = Path::new(&manifest_dir).join("static");
    
    if static_dir.exists() {
        println!("cargo:rerun-if-changed=static/");
        
        // In a real build system, we would copy static assets to the output directory
        // For now, just print a message
        println!("cargo:warning=Static assets directory found at {:?}", static_dir);
    }
}