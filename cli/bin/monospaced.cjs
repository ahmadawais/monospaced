#!/usr/bin/env node

const {existsSync} = require("node:fs");
const path = require("node:path");
const {spawnSync} = require("node:child_process");

const TARGET_CANDIDATES = {
  darwin: {
    x64: ["x86_64-apple-darwin"],
    arm64: ["aarch64-apple-darwin"],
  },
  linux: {
    x64: ["x86_64-unknown-linux-musl", "x86_64-unknown-linux-gnu"],
    arm64: ["aarch64-unknown-linux-musl", "aarch64-unknown-linux-gnu"],
  },
  win32: {
    x64: ["x86_64-pc-windows-msvc"],
    arm64: ["aarch64-pc-windows-msvc", "x86_64-pc-windows-msvc"],
  },
};

const platformTargets = TARGET_CANDIDATES[process.platform];
const targetCandidates = platformTargets?.[process.arch];

if (!targetCandidates) {
  console.error(
    `Unsupported platform: ${process.platform} ${process.arch}. ` +
      "Supported targets are macOS (x64/arm64), Linux (x64/arm64), and Windows (x64).",
  );
  process.exit(1);
}

const binaryName = process.platform === "win32" ? "monospaced.exe" : "monospaced";
const binaryPath =
  targetCandidates
    .map((target) => path.join(__dirname, "native", target, binaryName))
    .find((candidate) => existsSync(candidate)) ??
  path.join(__dirname, "native", targetCandidates[0], binaryName);

if (!existsSync(binaryPath)) {
  console.error(
    `Missing native binary for ${process.platform} ${process.arch}.\n` +
      "Run `pnpm build` to stage the cross-platform release binaries before invoking the npm launcher.",
  );
  process.exit(1);
}

const result = spawnSync(binaryPath, process.argv.slice(2), {
  env: process.env,
  stdio: "inherit",
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);
