use std::fs;

use anyhow::{Result, bail};
use xshell::{Shell, cmd};

use crate::cli::{GenerateServiceArgs, K6BaselineArgs, RequireToolArgs};
use crate::support;
use crate::support::{
    collect_files_with_extension, has_tool, normalize_slashes, read, require_tool, run_capture,
    run_inherit, workspace_root, write,
};

pub(crate) fn generate_service(args: GenerateServiceArgs) -> Result<()> {
    let working_directory = workspace_root()?;
    let content = format!(
        "[Unit]\nDescription=Axum API Service\nAfter=network.target\n\n[Service]\nType=simple\nUser={}\nGroup={}\nWorkingDirectory={}\nEnvironmentFile={}\nExecStart={}\nRestart=on-failure\nRestartSec=5\nNoNewPrivileges=true\nProtectSystem=strict\nReadWritePaths=/var/lib/axum-api\n\n[Install]\nWantedBy=multi-user.target\n",
        args.user,
        args.group,
        working_directory.display(),
        args.env_file.display(),
        args.bin_path.display()
    );
    write(working_directory.join("axum-api.service"), &content)?;
    println!("Generated axum-api.service");
    Ok(())
}
pub(crate) fn setup_nats() -> Result<()> {
    let root = workspace_root()?;
    let nats_dir = root.join("tools/nats");
    let nats_bin = if cfg!(windows) {
        nats_dir.join("nats-server.exe")
    } else {
        nats_dir.join("nats-server")
    };
    if nats_bin.exists() {
        println!("✓ nats-server already present: {}", nats_bin.display());
        return Ok(());
    }

    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(windows) {
        "windows"
    } else {
        bail!("unsupported os")
    };
    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "amd64"
    };
    let version = "2.10.22";
    println!("Platform: {os}/{arch}");
    fs::create_dir_all(&nats_dir)?;

    if os == "windows" {
        let file_name = format!("nats-server-v{version}-{os}-{arch}.zip");
        let url = format!(
            "https://github.com/nats-io/nats-server/releases/download/v{version}/{file_name}"
        );
        let archive = nats_dir.join(&file_name);
        support::download_to(&url, &archive)?;
        let extracted = support::extract_zip_file(&archive, &nats_dir, "nats-server.exe")?;
        fs::rename(extracted, &nats_bin).ok();
        fs::remove_file(&archive).ok();
    } else {
        let file_name = format!("nats-server-v{version}-{os}-{arch}.tar.gz");
        let url = format!(
            "https://github.com/nats-io/nats-server/releases/download/v{version}/{file_name}"
        );
        let archive = nats_dir.join(&file_name);
        support::download_to(&url, &archive)?;
        support::extract_tar_gz(&archive, &nats_dir, Some("nats-server"))?;
        fs::remove_file(&archive).ok();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&nats_bin)?.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&nats_bin, permissions)?;
        }
    }

    println!("✓ nats-server installed: {}", nats_bin.display());
    Ok(())
}

pub(crate) fn cleanup_test_artifacts() -> Result<()> {
    let root = workspace_root()?;
    let tmp_dir = root.join(".tmp");
    if !tmp_dir.exists() {
        return Ok(());
    }
    const MAX_SIZE: u64 = 10 * 1024 * 1024;
    const KEEP_LINES: usize = 50_000;
    for log_file in collect_files_with_extension(&tmp_dir, "log") {
        let metadata = fs::metadata(&log_file)?;
        if metadata.len() <= MAX_SIZE {
            continue;
        }
        println!(
            "  截断 {} (当前 {} bytes)",
            normalize_slashes(log_file.strip_prefix(&root)?),
            metadata.len()
        );
        let content = read(&log_file)?;
        let kept = content
            .lines()
            .rev()
            .take(KEEP_LINES)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        write(&log_file, &kept)?;
    }
    Ok(())
}

pub(crate) fn clean_sweep() -> Result<()> {
    require_tool("cargo-sweep", "cargo install cargo-sweep")?;
    let root = workspace_root()?;
    if run_inherit("cargo", &["sweep", "--time", "7"], Some(&root))? != 0 {
        bail!("cargo sweep --time 7 failed");
    }
    Ok(())
}

pub(crate) fn clean_sweep_deps() -> Result<()> {
    require_tool("cargo-sweep", "cargo install cargo-sweep")?;
    let root = workspace_root()?;
    if run_inherit("cargo", &["sweep", "--installed"], Some(&root))? != 0 {
        bail!("cargo sweep --installed failed");
    }
    Ok(())
}

pub(crate) fn require_tool_cmd(args: RequireToolArgs) -> Result<()> {
    require_tool(&args.tool, &args.install_hint)
}

pub(crate) fn setup_sccache() -> Result<()> {
    if has_tool("sccache") {
        println!("sccache already installed");
    } else {
        println!("Installing sccache...");
        if run_inherit("cargo", &["install", "sccache", "--locked"], None)? != 0 {
            bail!("cargo install sccache --locked failed");
        }
    }
    Ok(())
}

pub(crate) fn setup_sccache_verify() -> Result<()> {
    println!("--- sccache 状态 ---");
    if has_tool("sccache") {
        let _ = run_inherit("sccache", &["--show-stats"], None)?;
    } else {
        println!("✗ sccache not in PATH");
    }

    println!("\n--- 环境变量检查 ---");
    match std::env::var("RUSTC_WRAPPER") {
        Ok(wrapper) if wrapper == "sccache" => println!("✓ RUSTC_WRAPPER=sccache"),
        Ok(wrapper) => println!("⚠ RUSTC_WRAPPER={wrapper} (应该是 sccache)"),
        Err(_) => println!("✗ RUSTC_WRAPPER 未设置"),
    }

    println!("\n--- .cargo/config.toml 检查 ---");
    let root = workspace_root()?;
    let config = root.join(".cargo/config.toml");
    if config.exists() && read(&config)?.contains("rustc-wrapper") {
        println!("✓ rustc-wrapper 已配置");
    } else {
        println!("✗ rustc-wrapper 未设置");
    }
    Ok(())
}

pub(crate) fn setup_hakari() -> Result<()> {
    if has_tool("cargo-hakari") {
        let version = run_capture("cargo", &["hakari", "--version"], None)
            .map(|outcome| outcome.output)
            .unwrap_or_else(|_| "unknown version".to_string());
        println!("cargo-hakari already installed: {version}");
    } else {
        println!("Installing cargo-hakari...");
        if run_inherit("cargo", &["install", "cargo-hakari"], None)? != 0 {
            bail!("cargo install cargo-hakari failed");
        }
    }
    println!("Generating unified dependency resolution...");
    if run_inherit("cargo", &["hakari", "generate"], None)? != 0 {
        bail!("cargo hakari generate failed");
    }
    if run_inherit("cargo", &["hakari", "manage-deps", "--yes"], None)? != 0 {
        bail!("cargo hakari manage-deps --yes failed");
    }
    println!("hakari configured — run cargo build to see speedup");
    Ok(())
}

pub(crate) fn setup_hakari_verify() -> Result<()> {
    let root = workspace_root()?;
    if root.join(".config/hakari.toml").exists() {
        println!("✓ .config/hakari.toml exists");
    } else {
        println!("✗ .config/hakari.toml missing");
    }
    if read(root.join("Cargo.toml"))?.contains("workspace-hack") {
        println!("✓ workspace-hack in members");
    } else {
        println!("✗ workspace-hack NOT in members");
    }
    Ok(())
}

pub(crate) fn setup_coverage() -> Result<()> {
    for (tool, install_name) in [
        ("cargo-llvm-cov", "cargo-llvm-cov"),
        ("cargo-sweep", "cargo-sweep"),
    ] {
        if !has_tool(tool) && run_inherit("cargo", &["install", install_name], None)? != 0 {
            bail!("cargo install {install_name} failed");
        }
    }
    if run_inherit("rustup", &["component", "add", "llvm-tools"], None)? != 0 {
        bail!("rustup component add llvm-tools failed");
    }
    println!("Coverage tools ready");
    Ok(())
}

pub(crate) fn audit_rust() -> Result<()> {
    if run_inherit("cargo", &["audit"], None)? != 0 {
        println!("Install cargo-audit: cargo install cargo-audit");
        bail!("cargo audit failed");
    }
    Ok(())
}

const K6_BASELINE_SCRIPT: &str = r#"import http from "k6/http";
import { check, sleep } from "k6";

export const options = {
  vus: __ENV.TEST_VUS ? parseInt(__ENV.TEST_VUS) : 5,
  duration: __ENV.TEST_DURATION || "30s",
  thresholds: {
    http_req_duration: ["p(95)<500"],
    http_req_failed: ["rate<0.65"],
    checks: ["rate>0.3"],
  },
};

const BASE_URL = __ENV.BASE_URL || "http://localhost:3010";

const B64_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
function base64url(str) {
  var bytes = [];
  for (var i = 0; i < str.length; i++) {
    var c = str.charCodeAt(i);
    if (c < 0x80) bytes.push(c);
    else if (c < 0x800) { bytes.push(0xC0 | (c >> 6), 0x80 | (c & 0x3F)); }
    else { bytes.push(0xE0 | (c >> 12), 0x80 | ((c >> 6) & 0x3F), 0x80 | (c & 0x3F)); }
  }
  var result = "";
  for (var j = 0; j < bytes.length; j += 3) {
    var a = bytes[j], b = j + 1 < bytes.length ? bytes[j + 1] : 0, c = j + 2 < bytes.length ? bytes[j + 2] : 0;
    result += B64_CHARS[a >> 2];
    result += B64_CHARS[((a & 3) << 4) | (b >> 4)];
    result += j + 1 < bytes.length ? B64_CHARS[((b & 15) << 2) | (c >> 6)] : "=";
    result += j + 2 < bytes.length ? B64_CHARS[c & 63] : "=";
  }
  return result.replace(/=+$/, "").replace(/\+/g, "-").replace(/\//g, "_");
}

function uniqueUserSub() {
  return "k6-user-" + __VU + "-" + Date.now();
}

function devJwt(sub) {
  var header = base64url('{"alg":"HS256","typ":"JWT"}');
  var payload = base64url('{"sub":"' + sub + '"}');
  return header + "." + payload + ".dev-signature";
}

export default function () {
  var userSub = uniqueUserSub();
  var jwt = devJwt(userSub);
  var authHeaders = {
    "Content-Type": "application/json",
    Authorization: "Bearer " + jwt,
  };

  var healthRes = http.get(BASE_URL + "/healthz");
  check(healthRes, {
    "healthz 200": (r) => r.status === 200,
  });

  var tenantRes = http.post(
    BASE_URL + "/api/tenant/init",
    JSON.stringify({ user_sub: userSub, user_name: "k6-" + __VU }),
    { headers: authHeaders }
  );
  check(tenantRes, {
    "tenant init 200": (r) => r.status === 200,
  });

  for (var i = 0; i < 3; i++) {
    var incRes = http.post(
      BASE_URL + "/api/counter/increment",
      "{}",
      { headers: authHeaders }
    );
    check(incRes, {
      "increment 200": (r) => r.status === 200,
    });
  }

  var valRes = http.get(BASE_URL + "/api/counter/value", { headers: authHeaders });
  check(valRes, {
    "get value 200": (r) => r.status === 200,
    "value == 3": (r) => {
      try {
        return JSON.parse(r.body).value === 3;
      } catch {
        return false;
      }
    },
  });

  sleep(1);
}
"#;

pub(crate) fn k6_baseline(args: K6BaselineArgs) -> Result<()> {
    require_tool("k6", "install k6 via mise or your package manager")?;
    let root = workspace_root()?;
    let shell = Shell::new()?;
    shell.change_dir(&root);
    let base_url = args.base_url;
    let duration = args.duration;
    let vus = args.vus;

    let temp_dir = tempfile::tempdir()?;
    let script_path = temp_dir.path().join("k6-baseline.js");
    fs::write(&script_path, K6_BASELINE_SCRIPT)?;

    cmd!(
        shell,
        "k6 run -e BASE_URL={base_url} -e TEST_DURATION={duration} -e TEST_VUS={vus} {script_path}"
    )
    .run()?;
    Ok(())
}
