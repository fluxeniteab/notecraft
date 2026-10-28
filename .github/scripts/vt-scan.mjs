#!/usr/bin/env node
/**
 * VirusTotal scan for release installers.
 *
 * Usage:
 *   VT_API_KEY=... node .github/scripts/vt-scan.mjs <file> [file...]
 *
 * Exits non-zero when VirusTotal reports at least one malicious engine for any
 * artifact, so a release cannot ship a flagged binary by accident.
 * Files above the 32 MiB direct-upload limit are sent through the large file
 * upload URL that the API hands out.
 */
import fs from "node:fs";
import path from "node:path";

const API = "https://www.virustotal.com/api/v3";
const DIRECT_LIMIT = 32 * 1024 * 1024;
const POLL_TIMEOUT_MS = 15 * 60 * 1000;
const POLL_INTERVAL_MS = 15000;

const apiKey = process.env.VT_API_KEY || process.env.VT_SCAN;
if (!apiKey) {
  console.error("VT_API_KEY is not set — skipping VirusTotal scan.");
  process.exit(1);
}

const files = process.argv
  .slice(2)
  .map((value) => path.resolve(value))
  .filter((value) => fs.existsSync(value) && fs.statSync(value).isFile());

if (files.length === 0) {
  console.error("No artifacts matched — nothing to scan. Failing so a release never ships unscanned.");
  process.exit(1);
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const headers = { "x-apikey": apiKey };

async function upload(file) {
  const size = fs.statSync(file).size;
  const buffer = fs.readFileSync(file);
  let endpoint = `${API}/files`;
  if (size > DIRECT_LIMIT) {
    const response = await fetch(`${API}/files/upload_url`, { headers });
    if (!response.ok) {
      throw new Error(`upload_url failed: ${response.status} ${await response.text()}`);
    }
    const body = await response.json();
    endpoint = body.data;
  }
  const form = new FormData();
  form.append("file", new Blob([buffer]), path.basename(file));
  const response = await fetch(endpoint, { method: "POST", headers, body: form });
  if (!response.ok) {
    throw new Error(`upload failed: ${response.status} ${await response.text()}`);
  }
  const body = await response.json();
  return body.data?.id;
}

async function poll(analysisId) {
  const deadline = Date.now() + POLL_TIMEOUT_MS;
  while (Date.now() < deadline) {
    const response = await fetch(`${API}/analyses/${analysisId}`, { headers });
    if (!response.ok) {
      throw new Error(`analysis failed: ${response.status} ${await response.text()}`);
    }
    const body = await response.json();
    const attributes = body.data?.attributes || {};
    if (attributes.status === "completed") {
      const stats = attributes.stats || {};
      return {
        analysisId,
        stats,
        verdict: attributes.verdict,
        link: `https://www.virustotal.com/gui/file/${attributes.sha256 || ""}`,
      };
    }
    await sleep(POLL_INTERVAL_MS);
  }
  throw new Error("timed out waiting for VirusTotal analysis");
}

const rows = [];
let failed = false;

for (const file of files) {
  const name = path.basename(file);
  try {
    const analysisId = await upload(file);
    const result = await poll(analysisId);
    const malicious = result.stats.malicious || 0;
    const suspicious = result.stats.suspicious || 0;
    const harmless = result.stats.harmless || 0;
    if (malicious > 0) failed = true;
    rows.push({ name, malicious, suspicious, harmless, link: result.link });
    console.log(`${name}: ${malicious} malicious, ${suspicious} suspicious of ${harmless} clean`);
  } catch (error) {
    // A scan that could not run must not be reported as clean.
    failed = true;
    rows.push({ name, error: String(error?.message || error) });
    console.error(`${name}: scan error — ${error?.message || error}`);
  }
}

const summary = [
  "### VirusTotal scan",
  "",
  "| Artifact | Malicious | Suspicious | Clean | Report |",
  "| --- | --- | --- | --- | --- |",
  ...rows.map((row) =>
    row.error
      ? `| ${row.name} | scan failed | — | — | ${row.error} |`
      : `| ${row.name} | ${row.malicious} | ${row.suspicious} | ${row.harmless} | ${row.link} |`
  ),
  "",
].join("\n");

if (process.env.GITHUB_STEP_SUMMARY) {
  fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, `${summary}\n`);
}
console.log(summary);
process.exit(failed ? 1 : 0);
