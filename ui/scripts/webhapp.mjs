/*
 * Build the .webhapp — the hApp and the interface in one file.
 *
 * This is what kangaroo-electron takes to produce an installable desktop app,
 * which is the difference between "two windows on my laptop" and "two laptops
 * with the router unplugged". Only the second is the argument.
 */

import { spawnSync } from "node:child_process";
import { existsSync, rmSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "..", "..");
const ui = resolve(here, "..");
const workdir = join(root, "workdir");
const bin = join(root, "bin");
const hc = join(bin, process.platform === "win32" ? "hc.exe" : "hc");

const run = (cmd, args, opts = {}) => {
  const r = spawnSync(cmd, args, { stdio: "inherit", shell: true, ...opts });
  if (r.status !== 0) {
    console.error(`\nFailed: ${cmd} ${args.join(" ")}`);
    process.exit(r.status ?? 1);
  }
};

// 1. The zomes, then the bundles. Never pack a stale wasm — the .happ is a
//    snapshot, and rebuilding the wasm does not update it.
run("cargo", ["build", "--target", "wasm32-unknown-unknown", "--release"], {
  cwd: root,
});
run(hc, ["dna", "pack", join(root, "dnas", "aboutme", "workdir")]);
run(hc, ["app", "pack", workdir]);

// 2. The interface.
run("npm", ["run", "build"], { cwd: ui });

const icon = join(ui, "dist", "icon.png");
if (!existsSync(icon)) {
  console.error(
    `No icon.png in the built UI. Kangaroo needs one of at least 256x256 at\n` +
      `the root of the UI assets. Put it in ui/public/ so Vite copies it.`,
  );
  process.exit(1);
}

// 3. Zip the built interface. No zip binary is assumed: PowerShell has one on
//    Windows, and every Unix worth the name has `zip`.
const zip = join(workdir, "ui.zip");
rmSync(zip, { force: true });

if (process.platform === "win32") {
  run("powershell", [
    "-NoProfile",
    "-Command",
    `Compress-Archive -Path '${join(ui, "dist")}\\*' -DestinationPath '${zip}' -Force`,
  ]);
} else {
  run("zip", ["-r", "-q", zip, "."], { cwd: join(ui, "dist") });
}

// 4. And the whole thing.
run(hc, ["web-app", "pack", workdir]);

console.log(`\nBuilt ${join(workdir, "hearth.webhapp")}`);
console.log(
  `\nDrop it into kangaroo-electron's pouch/ folder to build the desktop app.`,
);
