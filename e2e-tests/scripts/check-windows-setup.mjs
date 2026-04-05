#!/usr/bin/env node
/**
 * Windows-specific validation script for Tauri E2E testing setup
 * Checks for common Windows issues before running tests
 */

import { existsSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { homedir, platform } from 'node:os';
import { spawnSync } from 'node:child_process';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const workspaceRoot = resolve(__dirname, '..', '..');
const e2eDir = join(workspaceRoot, 'e2e-tests');

function parseDotenvFile() {
  const envPath = join(workspaceRoot, '.env');
  if (!existsSync(envPath)) {
    return null;
  }

  const content = readFileSync(envPath, 'utf-8');
  const map = new Map();

  for (const line of content.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    const idx = trimmed.indexOf('=');
    if (idx <= 0) continue;

    const key = trimmed.slice(0, idx).trim();
    const value = trimmed.slice(idx + 1).trim();
    map.set(key, value);
  }

  return { envPath, map };
}

function inspectEnvValue(raw) {
  if (!raw) {
    return { ok: false, reason: 'missing' };
  }

  const startsQuote = raw.startsWith('"') || raw.startsWith("'");
  const endsQuote = raw.endsWith('"') || raw.endsWith("'");
  const endsSemicolon = raw.endsWith(';');

  if (startsQuote && !endsQuote) {
    return { ok: false, reason: 'starts with quote but does not end with quote' };
  }

  if (endsSemicolon) {
    return { ok: false, reason: 'ends with semicolon (;) which breaks OAuth secret' };
  }

  return { ok: true };
}

function firstOutputLine(stdout) {
  return stdout
    ?.split(/\r?\n/)
    .map((line) => line.trim())
    .find(Boolean) ?? null;
}

function lookupBinary(binaryName) {
  const command = platform() === 'win32' ? 'where' : 'which';
  const result = spawnSync(command, [binaryName], {
    encoding: 'utf-8',
    shell: true,
  });

  if (result.status !== 0) {
    return null;
  }

  return firstOutputLine(result.stdout);
}

function resolveTauriDriverPath() {
  const binaryName = platform() === 'win32' ? 'tauri-driver.exe' : 'tauri-driver';
  const cargoBinPath = join(homedir(), '.cargo', 'bin', binaryName);
  if (existsSync(cargoBinPath)) {
    return cargoBinPath;
  }

  return lookupBinary('tauri-driver');
}

function resolveNativeDriverPath() {
  if (platform() !== 'win32') {
    return null;
  }

  const configuredPath = process.env.TAURI_NATIVE_DRIVER;
  if (configuredPath) {
    const resolvedPath = resolve(configuredPath);
    if (existsSync(resolvedPath)) {
      return resolvedPath;
    }
  }

  const pathBinary = lookupBinary('msedgedriver');
  if (pathBinary) {
    return pathBinary;
  }

  const repoLocalBinary = join(workspaceRoot, 'msedgedriver.exe');
  if (existsSync(repoLocalBinary)) {
    return repoLocalBinary;
  }

  return null;
}

function resolveVswherePath() {
  const pathBinary = lookupBinary('vswhere');
  if (pathBinary) {
    return pathBinary;
  }

  const standardPath = join(
    process.env['ProgramFiles(x86)'] || 'C:\\Program Files (x86)',
    'Microsoft Visual Studio',
    'Installer',
    'vswhere.exe',
  );

  return existsSync(standardPath) ? standardPath : null;
}

function checkWindowsPrerequisites() {
  const isWindows = platform() === 'win32';

  console.log('=== Windows E2E Testing Prerequisites Check ===\n');
  console.log(`Platform: ${platform()}\n`);

  const errors = [];
  const warnings = [];

  // Check 1: tauri-driver
  const tauriDriverPath = resolveTauriDriverPath();
  if (tauriDriverPath) {
    console.log('✅ tauri-driver found:', tauriDriverPath);
  } else {
    errors.push('❌ tauri-driver not found. Install with: cargo install tauri-driver --locked');
    console.log(errors[errors.length - 1]);
  }

  // Check 2: Microsoft Edge Driver
  if (isWindows) {
    const nativeDriverPath = resolveNativeDriverPath();
    if (nativeDriverPath) {
      console.log('✅ msedgedriver found:', nativeDriverPath);
    } else {
      errors.push(
        '❌ msedgedriver.exe not found. Install a matching Microsoft Edge Driver and add it to PATH, or set TAURI_NATIVE_DRIVER.'
      );
      console.log(errors[errors.length - 1]);
    }
  }

  // Check 3: Visual Studio Build Tools
  if (isWindows) {
    const vswherePath = resolveVswherePath();

    if (vswherePath) {
      console.log('✅ Visual Studio Installer/vswhere found:', vswherePath);

      const result = spawnSync(vswherePath, ['-latest', '-products', '*', '-requires', 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64', '-property', 'installationPath'], {
        encoding: 'utf-8',
        shell: false,
      });

      if (result.stdout && result.stdout.trim()) {
        const buildToolsPath = result.stdout.trim();
        console.log('✅ Visual Studio Build Tools detected:', buildToolsPath);

        const vcvarsallPath = join(buildToolsPath, 'VC', 'Auxiliary', 'Build', 'vcvarsall.bat');
        if (existsSync(vcvarsallPath)) {
          console.log('✅ vcvarsall.bat found:', vcvarsallPath);
        } else {
          warnings.push('⚠️  vcvarsall.bat not found at expected location');
          console.log(warnings[warnings.length - 1]);
        }
      } else {
        warnings.push('⚠️  Visual Studio Build Tools not detected. Install from: https://visualstudio.microsoft.com/visual-cpp-build-tools/');
        console.log(warnings[warnings.length - 1]);
      }
    } else {
      warnings.push('⚠️  vswhere.exe not found. Required for detecting Visual Studio Build Tools.');
      console.log(warnings[warnings.length - 1]);
    }
  }

  // Check 4: WebView2 Runtime
  if (isWindows) {
    const webview2RegPath = 'HKLM\\SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}';
    const result = spawnSync('reg', ['query', webview2RegPath, '/v', 'pv'], {
      encoding: 'utf-8',
      shell: true,
    });

    if (result.status === 0 && result.stdout) {
      console.log('✅ WebView2 Runtime detected');
    } else {
      warnings.push('⚠️  WebView2 Runtime not detected. Windows 10+ includes it by default.');
      console.log(warnings[warnings.length - 1]);
    }
  }

  // Check 5: Node.js and Bun
  const nodeResult = spawnSync('node', ['--version'], { encoding: 'utf-8', shell: true });
  if (nodeResult.status === 0) {
    console.log('✅ Node.js:', nodeResult.stdout.trim());
  } else {
    errors.push('❌ Node.js not found');
    console.log(errors[errors.length - 1]);
  }

  const bunResult = spawnSync('bun', ['--version'], { encoding: 'utf-8', shell: true });
  if (bunResult.status === 0) {
    console.log('✅ Bun:', bunResult.stdout.trim());
  } else {
    errors.push('❌ Bun not found');
    console.log(errors[errors.length - 1]);
  }

  // Check 6: Cargo/Rust
  const cargoResult = spawnSync('cargo', ['--version'], { encoding: 'utf-8', shell: true });
  if (cargoResult.status === 0) {
    console.log('✅ Cargo:', cargoResult.stdout.trim());
  } else {
    errors.push('❌ Cargo not found');
    console.log(errors[errors.length - 1]);
  }

  // Check 7: WDIO dependencies
  const wdioPackageJson = join(e2eDir, 'package.json');
  if (existsSync(wdioPackageJson)) {
    console.log('✅ WDIO package.json found');

    const nodeModulesPath = join(e2eDir, 'node_modules');
    if (existsSync(nodeModulesPath)) {
      console.log('✅ WDIO node_modules found');
    } else {
      errors.push('❌ WDIO node_modules not found. Run: bun install --cwd e2e-tests');
      console.log(errors[errors.length - 1]);
    }
  } else {
    errors.push('❌ WDIO package.json not found in e2e-tests/');
    console.log(errors[errors.length - 1]);
  }

  // Check 8: OAuth env formatting
  const dotenv = parseDotenvFile();
  if (!dotenv) {
    warnings.push('⚠️  .env not found at workspace root. OAuth login may fail without GOOGLE_CLIENT_ID/GOOGLE_CLIENT_SECRET.');
    console.log(warnings[warnings.length - 1]);
  } else {
    const idRaw = dotenv.map.get('GOOGLE_CLIENT_ID');
    const secretRaw = dotenv.map.get('GOOGLE_CLIENT_SECRET');

    const idCheck = inspectEnvValue(idRaw);
    const secretCheck = inspectEnvValue(secretRaw);

    if (!idCheck.ok) {
      errors.push(`❌ GOOGLE_CLIENT_ID format issue in ${dotenv.envPath}: ${idCheck.reason}`);
      console.log(errors[errors.length - 1]);
    }

    if (!secretCheck.ok) {
      errors.push(`❌ GOOGLE_CLIENT_SECRET format issue in ${dotenv.envPath}: ${secretCheck.reason}`);
      console.log(errors[errors.length - 1]);
    }

    if (idCheck.ok && secretCheck.ok) {
      console.log(`✅ OAuth env formatting looks valid: ${dotenv.envPath}`);
    }
  }

  console.log('\n=== Summary ===');
  if (errors.length === 0 && warnings.length === 0) {
    console.log('✅ All checks passed! Ready to run E2E tests.\n');
    console.log('Next step: bun run --cwd e2e-tests test:ci');
    return true;
  }

  if (warnings.length > 0) {
    console.log(`⚠️  Found ${warnings.length} warning(s):\n`);
    warnings.forEach((issue, i) => console.log(`${i + 1}. ${issue}`));
    console.log('');
  }

  if (errors.length > 0) {
    console.log(`❌ Found ${errors.length} blocking issue(s):\n`);
    errors.forEach((issue, i) => console.log(`${i + 1}. ${issue}`));
    console.log('\nPlease resolve blocking issues before running E2E tests.');
    return false;
  }

  console.log('✅ No blocking issues found. You can continue to run E2E tests.');
  console.log('Recommended next step: bun run --cwd e2e-tests test:ci');
  return true;
}

const success = checkWindowsPrerequisites();
process.exit(success ? 0 : 1);
