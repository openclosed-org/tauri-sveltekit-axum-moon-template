use anyhow::Result;

use crate::support::{has_tool, run_capture, workspace_root};

pub(crate) fn doctor() -> Result<()> {
    println!("=== Toolchain Check ===\n");
    for (name, tool) in [
        ("cargo", "cargo"),
        ("rustc", "rustc"),
        ("moon", "moon"),
        ("just", "just"),
    ] {
        if has_tool(tool) {
            let result = run_capture(tool, &["--version"], None)?;
            println!("✓ {name}: {}", result.output);
        } else {
            println!("✗ MISSING: {name}");
        }
    }

    println!("\n=== Config Files Check ===\n");
    let root = workspace_root()?;
    for config in [".env", ".env.example", ".mise.toml", "rust-toolchain.toml"] {
        let path = root.join(config);
        if path.exists() {
            println!("✓ {config}: exists");
        } else {
            println!("✗ MISSING: {config}");
        }
    }

    println!("\n=== Done ===");
    Ok(())
}
