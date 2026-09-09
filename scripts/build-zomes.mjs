/*
 * Build the zomes so that every machine of the same platform gets the same
 * wasm — and therefore the same DNA hash, and therefore the same circle.
 *
 * Why this script exists at all
 * ----------------------------
 * A circle IS the hash of the compiled integrity wasm. So anything that
 * changes the bytes changes which circle you are in, and two people running
 * "the same" app can end up unable to find each other with nothing to see.
 *
 * Rust embeds the source paths of every dependency into the binary, for panic
 * messages. Those paths contain the building machine's home directory, so
 * plain `cargo build` produces a different DNA hash on every computer, and the
 * released installer carried the string `C:\Users\user\...` inside it.
 *
 * `--remap-path-prefix` fixes that, but it cannot be put in `.cargo/config.toml`
 * because the prefix is different on every machine and cargo does not expand
 * variables there. Hence a script: it works the prefixes out, then builds.
 *
 * What this does NOT fix, and cannot
 * ----------------------------------
 * Windows writes the remapped paths with backslashes and Linux with forward
 * slashes — `/cargo\registry\...` against `/cargo/registry/...`. The separator
 * is the operating system's, not the prefix's, so no flag available on stable
 * Rust makes the two platforms produce identical bytes.
 *
 * **So Windows and Linux builds are, and will remain, different networks.**
 * The answer to that is not to build twice: it is to build once and publish
 * the result. See the README — a desktop app for any platform is assembled
 * from the released .webhapp, not from a local compile.
 */

import { spawnSync } from "node:child_process";
import { homedir } from "node:os";
import { join, resolve } from "node:path";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));

// Where cargo keeps the dependency sources whose paths get baked in.
const cargoHome = process.env.CARGO_HOME || join(homedir(), ".cargo");

/*
 * These two must match `.cargo/config.toml` exactly.
 *
 * Setting RUSTFLAGS in the environment REPLACES the config's rustflags rather
 * than adding to them, so they have to be repeated here. Losing
 * getrandom_backend produces an error about wasm32 support that looks entirely
 * unrelated to anything, and costs an afternoon.
 */
const required = [
  "-C",
  "link-arg=--import-undefined",
  "--cfg",
  'getrandom_backend="custom"',
];

const flags = [
  ...required,
  `--remap-path-prefix=${cargoHome}=/cargo`,
  `--remap-path-prefix=${root}=/hearth`,
];

if (!existsSync(join(root, ".cargo", "config.toml"))) {
  console.error("No .cargo/config.toml — the required flags may have moved.");
  process.exit(1);
}

const result = spawnSync(
  "cargo",
  ["build", "--target", "wasm32-unknown-unknown", "--release"],
  {
    cwd: root,
    stdio: "inherit",
    shell: process.platform === "win32",
    env: { ...process.env, RUSTFLAGS: flags.join(" ") },
  },
);

if (result.status !== 0) process.exit(result.status ?? 1);
