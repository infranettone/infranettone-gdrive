// Builds the gdrive CLI and drops it where Tauri expects the sidecar declared
// as `binaries/gdrive` in tauri.conf.json: `binaries/gdrive-<target-triple>`.
//
// `tauri build` and `tauri dev` both refuse to start when the sidecar for the
// target triple is missing, so this runs before either of them.
//
// With no argument it builds for the host *without* `--target`, into the same
// target/release/ that `tauri build` uses, so the dependencies compiled here
// are reused by the app build instead of being compiled a second time. Pass a
// triple only for a cross build.
import { execFileSync } from "node:child_process";
import { mkdirSync, copyFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

const requested = process.argv[2];

const triple =
  requested ??
  execFileSync("rustc", ["-vV"], { encoding: "utf8" })
    .split("\n")
    .find((line) => line.startsWith("host:"))
    .slice("host:".length)
    .trim();

const ext = triple.includes("windows") ? ".exe" : "";

const args = ["build", "--release", "-p", "gdrive"];
if (requested) args.push("--target", requested);

execFileSync("cargo", args, { cwd: root, stdio: "inherit" });

const built = requested
  ? join(root, "target", requested, "release", `gdrive${ext}`)
  : join(root, "target", "release", `gdrive${ext}`);

const binaries = join(root, "gdrive-ui", "src-tauri", "binaries");
mkdirSync(binaries, { recursive: true });
copyFileSync(built, join(binaries, `gdrive-${triple}${ext}`));

console.log(`Sidecar ready: binaries/gdrive-${triple}${ext}`);
