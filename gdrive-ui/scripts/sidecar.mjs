// Builds the gdrive CLI and drops it where Tauri expects the sidecar declared
// as `binaries/gdrive` in tauri.conf.json: `binaries/gdrive-<target-triple>`.
//
// `tauri build` and `tauri dev` both refuse to start when the sidecar for the
// host triple is missing, so this runs before either of them.
import { execFileSync } from "node:child_process";
import { mkdirSync, copyFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

// The triple can be passed through for a cross build; otherwise use rustc's host.
const triple =
  process.argv[2] ??
  execFileSync("rustc", ["-vV"], { encoding: "utf8" })
    .split("\n")
    .find((line) => line.startsWith("host:"))
    .slice("host:".length)
    .trim();

const ext = triple.includes("windows") ? ".exe" : "";

execFileSync("cargo", ["build", "--release", "-p", "gdrive", "--target", triple], {
  cwd: root,
  stdio: "inherit",
});

const binaries = join(root, "gdrive-ui", "src-tauri", "binaries");
mkdirSync(binaries, { recursive: true });
copyFileSync(
  join(root, "target", triple, "release", `gdrive${ext}`),
  join(binaries, `gdrive-${triple}${ext}`),
);

console.log(`Sidecar ready: binaries/gdrive-${triple}${ext}`);
