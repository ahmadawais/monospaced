#!/usr/bin/env node

import {readFileSync, writeFileSync} from "node:fs";
import path from "node:path";
import {fileURLToPath} from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");
const packageJsonPath = path.join(rootDir, "package.json");
const cargoTomlPath = path.join(rootDir, "Cargo.toml");

const packageJson = JSON.parse(readFileSync(packageJsonPath, "utf8"));
const cargoToml = readFileSync(cargoTomlPath, "utf8");

const nextCargoToml = cargoToml.replace(
  /^version = ".*"$/m,
  `version = "${packageJson.version}"`,
);

if (nextCargoToml !== cargoToml) {
  writeFileSync(cargoTomlPath, nextCargoToml);
  console.log(`Synced Cargo.toml to version ${packageJson.version}`);
}
