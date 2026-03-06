#!/usr/bin/env node

import {chmodSync, copyFileSync, mkdirSync, rmSync} from "node:fs";
import path from "node:path";
import {fileURLToPath} from "node:url";
import {spawnSync} from "node:child_process";

const DEFAULT_TARGETS = [
  "aarch64-apple-darwin",
  "x86_64-apple-darwin",
  "x86_64-unknown-linux-musl",
  "aarch64-unknown-linux-musl",
  "x86_64-pc-windows-msvc",
];

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");
const hostTarget = detectHostTarget();
const targets = parseTargets(process.env.TARGETS) ?? DEFAULT_TARGETS;
const nativeDir = path.join(rootDir, "bin", "native");
const hasCross = commandExists("cross");
const ensuredRustTargets = new Set();

rmSync(nativeDir, {recursive: true, force: true});

for (const target of targets) {
  const builder = selectBuilder(target, hostTarget);
  const binaryName = target.includes("windows") ? "monospaced.exe" : "monospaced";

  if (builder === "cargo") {
    ensureRustTarget(target);
  } else if (!hasCross) {
    fail(
      "Missing `cross`.\n" +
        "Install it once with `cargo install cross --git https://github.com/cross-rs/cross` " +
        "and make sure Docker or Podman is available.",
    );
  }

  console.log(`Building ${target} with ${builder}...`);
  run(builder, ["build", "--release", "--target", target], {cwd: rootDir, stdio: "inherit"});

  const sourcePath = path.join(rootDir, "target", target, "release", binaryName);
  const destinationDir = path.join(nativeDir, target);
  const destinationPath = path.join(destinationDir, binaryName);

  mkdirSync(destinationDir, {recursive: true});
  copyFileSync(sourcePath, destinationPath);

  if (!binaryName.endsWith(".exe")) {
    chmodSync(destinationPath, 0o755);
  }
}

console.log(`Staged ${targets.length} target(s) in ${path.relative(rootDir, nativeDir)}`);

function parseTargets(value) {
  if (!value) {
    return null;
  }

  const targets = value
    .split(",")
    .map((entry) => entry.trim())
    .filter(Boolean);

  return targets.length > 0 ? targets : null;
}

function selectBuilder(target, host) {
  if (target.endsWith("apple-darwin")) {
    if (!host.endsWith("apple-darwin")) {
      fail(`Apple target ${target} must be built from a macOS host.`);
    }

    return "cargo";
  }

  return target === host ? "cargo" : "cross";
}

function ensureRustTarget(target) {
  if (ensuredRustTargets.has(target)) {
    return;
  }

  run("rustup", ["target", "add", target], {cwd: rootDir, stdio: "inherit"});
  ensuredRustTargets.add(target);
}

function detectHostTarget() {
  const result = spawnSync("rustc", ["-vV"], {
    cwd: rootDir,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  });

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }

  const hostLine = result.stdout
    .split("\n")
    .find((line) => line.startsWith("host: "));

  if (!hostLine) {
    fail("Unable to detect the Rust host target.");
  }

  return hostLine.replace("host: ", "").trim();
}

function commandExists(command) {
  const result = spawnSync(command, ["--version"], {
    cwd: rootDir,
    stdio: "ignore",
  });

  return result.status === 0;
}

function run(command, args, options) {
  const result = spawnSync(command, args, options);
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
