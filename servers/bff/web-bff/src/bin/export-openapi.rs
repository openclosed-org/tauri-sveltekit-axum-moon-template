use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = PathBuf::from("packages/contracts/generated/openapi/web-bff.openapi.yaml");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => {
                let Some(value) = args.next() else {
                    return Err("--output requires a path".into());
                };
                output = PathBuf::from(value);
            }
            "--format" => {
                let Some(value) = args.next() else {
                    return Err("--format requires a value".into());
                };
                if value != "yaml" {
                    return Err(format!("unsupported format '{value}', expected yaml").into());
                }
            }
            other => return Err(format!("unexpected argument '{other}'").into()),
        }
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output, web_bff::openapi().to_yaml()?)?;
    Ok(())
}
