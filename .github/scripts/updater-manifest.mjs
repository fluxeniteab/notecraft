#!/usr/bin/env node
/**
 * Merge one platform's updater artifact into `latest.json`.
 *
 * Each release workflow builds on its own runner, so this script reads the
 * manifest already attached to the release, adds or replaces the entry for the
 * current platform, and uploads it again. The Tauri updater reads the result.
 *
 * Because the three platform workflows can publish at the same time, every
 * attempt re-reads the manifest and verifies afterwards that this platform's
 * entry survived; a lost update is retried instead of silently dropping a
 * platform from the update feed.
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
    console.error(
      `Signature missing for ${artifact} — did the build run without TAURI_SIGNING_PRIVATE_KEY?`
    );
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
const entry = {
  signature,
  url: `https://github.com/${repo}/releases/download/${tag}/${path.basename(artifact)}`,
};

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

function download() {
  fs.rmSync(manifestPath, { force: true });
  spawnSync("gh", ["release", "download", tag, "-p", "latest.json", "-D", workdir, "--clobber"], {
    stdio: "ignore",
  });
  if (!fs.existsSync(manifestPath)) return null;
  try {
    return JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  } catch {
    console.warn("Existing latest.json could not be parsed; writing a fresh manifest");
    return null;
  }
}

const attempts = 6;
for (let attempt = 1; attempt <= attempts; attempt += 1) {
  const existing = download();
  const manifest = {
    version,
    notes: existing?.notes ?? "",
    pub_date: new Date().toISOString(),
    platforms: { ...(existing?.platforms ?? {}) },
  };
  manifest.version = version;
  manifest.platforms[platform] = entry;

  fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
  const upload = spawnSync("gh", ["release", "upload", tag, manifestPath, "--clobber"], {
    stdio: "inherit",
  });
  if (upload.status !== 0) {
    console.error("Could not upload latest.json");
    process.exit(1);
  }

  // Give a concurrent publisher a moment to land, then confirm our entry is
  // still part of the manifest. A lost update is retried with a fresh read.
  await sleep(1500 + Math.round(Math.random() * 2500));
  const verified = download();
  if (verified?.platforms?.[platform]?.signature === signature) {
    const platforms = Object.keys(verified.platforms).sort().join(", ");
    console.log(`latest.json now advertises ${version} for ${platform} (platforms: ${platforms})`);
    process.exit(0);
  }
  console.warn(`Attempt ${attempt}: another platform overwrote latest.json, retrying the merge`);
  await sleep(2000 + Math.round(Math.random() * 3000));
}

console.error("latest.json could not be merged: this platform kept getting overwritten");
process.exit(1);
