/*
 * Launch the demo: two agents, two windows, one machine.
 *
 * hc-spin shells out to `kitsune2-bootstrap-srv`, `holochain`, `lair-keystore`
 * and `hc` by bare name, so they have to be on PATH. It does not bundle them.
 * When they are missing the only symptom is:
 *
 *     [hc-spin] | [hc run-local-services] ERROR:
 *
 * — an empty message, because the spawn failed rather than the process. That
 * cost an hour once. This script puts ../bin on PATH so it cannot happen again.
 */

import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const bin = resolve(here, "..", "..", "bin");
const happ = resolve(here, "..", "..", "workdir", "aboutme.happ");

const needed = ["hc", "holochain", "lair-keystore", "kitsune2-bootstrap-srv"];
const missing = needed.filter(
  (name) => !existsSync(join(bin, `${name}.exe`)) && !existsSync(join(bin, name)),
);

if (missing.length) {
  console.error(
    `Missing from ${bin}: ${missing.join(", ")}\n\n` +
      `Fetch them from the Holochain 0.7.0 release — see the README.`,
  );
  process.exit(1);
}

if (!existsSync(happ)) {
  console.error(
    `No hApp at ${happ}\n\n` +
      `Build and pack it first:\n` +
      `  cargo build --target wasm32-unknown-unknown --release\n` +
      `  ./bin/hc dna pack dnas/aboutme/workdir\n` +
      `  ./bin/hc app pack workdir`,
  );
  process.exit(1);
}

const agents = process.argv[2] ?? "2";

const child = spawn(
  "hc-spin",
  ["-n", agents, "--ui-port", "5273", happ],
  {
    stdio: "inherit",
    shell: true,
    env: { ...process.env, PATH: `${bin}${process.platform === "win32" ? ";" : ":"}${process.env.PATH}` },
  },
);

child.on("exit", (code) => process.exit(code ?? 0));
