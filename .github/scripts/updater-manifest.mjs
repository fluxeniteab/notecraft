#!/usr/bin/env node
/**
 * Merge one platform's updater artifact into `latest.json`.
 *
 * Each release workflow builds on its own runner, so this script reads the
 * manifest already attached to the release, adds or replaces the entry for the
 * current platform, and uploads it again. The Tauri updater reads the result.
 *
 * Required env: RELEASE_TAG, PLATFORM, ARTIFACT (and SIGNATURE or a .sig file).
 */
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

const tag = process.env.RELEASE_TAG;
const platform = process.env.PLATFORM;
if (!tag || !platform) {
  console.error("RELEASE_TAG and PLATFORM are required");
  process.exit(1);
}

const config = JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8"));
const version = config.version;

const artifact = process.env.ARTIFACT;
if (!artifact || !fs.existsSync(artifact)) {
  console.error(`Updater artifact not found: ${artifact}`);
  process.exit(1);
}

let signature = process.env.SIGNATURE || "";
if (!signature) {
  const sigPath = `${artifact}.sig`;
  if (!fs.existsSync(sigPath)) {
    console.error(`Signature missing for ${artifact} — did the build run without TAURI_SIGNING_PRIVATE_KEY?`);
    process.exit(1);
  }
  signature = fs.readFileSync(sigPath, "utf8").trim();
}
if (!signature) {
  console.error("Empty updater signature");
  process.exit(1);
}

const repo = process.env.GITHUB_REPOSITORY || "fluxeniteab/notecraft";
const workdir = fs.mkdtempSync(path.join(os.tmpdir(), "notecraft-updater-"));
const manifestPath = path.join(workdir, "latest.json");

spawnSync("gh", ["release", "download", tag, "-p", "latest.json", "-D", workdir, "--clobber"], {
  stdio: "ignore",
});

let manifest = { version, notes: "", pub_date: new Date().toISOString(), platforms: {} };
if (fs.existsSync(manifestPath)) {
  try {
    const existing = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
    manifest = { ...manifest, ...existing, platforms: { ...(existing.platforms || {}) } };
  } catch {
    console.warn("Existing latest.json could not be parsed; writing a fresh manifest");
  }
}

manifest.version = version;
manifest.pub_date = new Date().toISOString();
manifest.platforms[platform] = {
  signature,
  url: `https://github.com/${repo}/releases/download/${tag}/${path.basename(artifact)}`,
};

fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
const upload = spawnSync("gh", ["release", "upload", tag, manifestPath, "--clobber"], {
  stdio: "inherit",
});
if (upload.status !== 0) {
  console.error("Could not upload latest.json");
  process.exit(1);
}
console.log(`latest.json now advertises ${version} for ${platform}`);
