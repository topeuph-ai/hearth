/*
 * Launch the demo: as many agents as you ask for, one machine.
 *
 *     npm run demo        two people
 *     npm run demo -- 3   three
 *     npm run demo -- 6   six
 *
 * One command. It starts the interface server, waits for it, packs a fresh
 * hApp and opens the windows — because a demo command that needs a second
 * terminal is not a demo command.
 *
 * On the number: the conductor is not what limits this. Measured on this
 * machine, each extra agent costs about 52MB of holochain and 6MB of
 * lair-keystore. The window costs several times that, because it is Chrome.
 * So the ceiling is the browser, not Holochain — which is worth knowing,
 * because it is the opposite of what people expect of a peer-to-peer app.
 *
 * One machine, though. Every agent here shares one bootstrap and one relay
 * server running on 127.0.0.1, so two laptops each running this would never
 * find each other. Crossing machines is the desktop build's job — see the
 * README.
 *
 * Two things this exists to prevent, both of which cost real time:
 *
 * 1. hc-spin shells out to `kitsune2-bootstrap-srv`, `holochain`,
 *    `lair-keystore` and `hc` by bare name and does not bundle them. When they
 *    are missing the only symptom is an empty error, because the spawn failed
 *    rather than the process. So ../bin goes on PATH.
 *
 * 2. The .happ is a snapshot of the wasm, and rebuilding the wasm does not
 *    update it. A stale bundle fails by telling you a function you can read in
 *    the source does not exist. So it is always re-packed.
 */

import { spawn, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import net from "node:net";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "..", "..");
const ui = resolve(here, "..");
const bin = join(root, "bin");
const happ = join(root, "workdir", "aboutme.happ");
const UI_PORT = 5273;
const isWindows = process.platform === "win32";
const exe = (name) => join(bin, isWindows ? `${name}.exe` : name);

const die = (message) => {
  console.error(`\n${message}\n`);
  process.exit(1);
};

// Checked before anything else happens. A typo here should cost a message,
// not a packed hApp and a port check.
const asked = process.argv[2] ?? "2";
const agents = Number(asked);

if (!Number.isInteger(agents) || agents < 1) {
  die(
    `"${asked}" is not a number of people.\n\n` +
      `  npm run demo        two people\n` +
      `  npm run demo -- 3   three`,
  );
}

// Not a limit, a warning. Nothing here stops a larger number; it is just that
// each window is a Chrome, and a machine has only so much of that in it.
if (agents > 6) {
  console.log(
    `\n${agents} windows is a lot of Chrome for one machine. Carrying on.\n`,
  );
}


// ---------------------------------------------------------------------------
// Everything present?
// ---------------------------------------------------------------------------

const missing = ["hc", "holochain", "lair-keystore", "kitsune2-bootstrap-srv"]
  .filter((name) => !existsSync(exe(name)));

if (missing.length) {
  die(
    `Missing from ${bin}: ${missing.join(", ")}\n\n` +
      `Fetch them from the Holochain 0.7.0 release — see the README.`,
  );
}

const wasm = join(root, "target", "wasm32-unknown-unknown", "release");
for (const name of ["aboutme.wasm", "aboutme_integrity.wasm"]) {
  if (!existsSync(join(wasm, name))) {
    die(
      `Missing ${name}. Build the zomes first:\n` +
        `  cargo build --target wasm32-unknown-unknown --release`,
    );
  }
}

// ---------------------------------------------------------------------------
// Always pack fresh
// ---------------------------------------------------------------------------

console.log("Packing the hApp...");
for (const [what, where] of [
  ["dna", join(root, "dnas", "aboutme", "workdir")],
  ["app", join(root, "workdir")],
]) {
  const packed = spawnSync(exe("hc"), [what, "pack", where], { stdio: "inherit" });
  if (packed.status !== 0) die(`Could not pack the ${what}.`);
}
if (!existsSync(happ)) die(`No hApp at ${happ}`);

// ---------------------------------------------------------------------------
// The interface server
// ---------------------------------------------------------------------------

const portIsOpen = () =>
  new Promise((resolve) => {
    const socket = net.connect(UI_PORT, "127.0.0.1");
    socket.once("connect", () => (socket.destroy(), resolve(true)));
    socket.once("error", () => (socket.destroy(), resolve(false)));
  });

/*
 * Closing the windows does not always take the conductors with them, and a
 * stray conductor then blocks the next run. Clearing up on the way in is more
 * reliable than remembering to on the way out.
 *
 * **Only processes started from this project's bin/.** Killing every
 * holochain.exe would take down conductors belonging to entirely different
 * projects on the same machine, which is a rude thing for a demo script to do.
 */
function clearLeftovers() {
  if (!isWindows) return;

  const ours = bin.replace(/\\/g, "\\\\");
  const script =
    `Get-CimInstance Win32_Process | ` +
    `Where-Object { $_.ExecutablePath -like '${ours}*' } | ` +
    `ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }`;

  spawnSync("powershell", ["-NoProfile", "-Command", script], {
    stdio: "ignore",
  });
}

if (await portIsOpen()) {
  console.log("Clearing a previous run...");
  clearLeftovers();
  await new Promise((r) => setTimeout(r, 2000));

  if (await portIsOpen()) {
    die(
      `Port ${UI_PORT} is still in use by something that is not a previous\n` +
        `demo. Close it and try again.`,
    );
  }
} else {
  clearLeftovers();
}

console.log("Starting the interface...");
// Bind IPv4 explicitly. Left to itself vite listens on [::1] only, the probe
// below connects to 127.0.0.1, and the two never meet - which looks exactly
// like the server failing to start.
const vite = spawn("npx", ["vite", "--host", "127.0.0.1", "--port", String(UI_PORT), "--strictPort"], {
  cwd: ui,
  stdio: "inherit",
  shell: true,
});

const stopVite = () => {
  if (!vite.killed) vite.kill();
};
process.on("exit", stopVite);
process.on("SIGINT", () => (stopVite(), process.exit(0)));

// Wait for it rather than assuming. Launching is not evidence of running.
const deadline = Date.now() + 60_000;
while (!(await portIsOpen())) {
  if (Date.now() > deadline) {
    stopVite();
    die(`The interface server did not come up on port ${UI_PORT}.`);
  }
  await new Promise((r) => setTimeout(r, 300));
}

// ---------------------------------------------------------------------------
// However many people you asked for
// ---------------------------------------------------------------------------

console.log(
  `\nOpening ${agents} window${agents === 1 ? "" : "s"}. Close them to stop.\n`,
);

const spin = spawn(
  "hc-spin",
  ["-n", String(agents), "--ui-port", String(UI_PORT), happ],
  {
    cwd: ui,
    stdio: "inherit",
    shell: true,
    env: {
      ...process.env,
      PATH: `${bin}${isWindows ? ";" : ":"}${process.env.PATH}`,
    },
  },
);

spin.on("exit", (code) => {
  stopVite();
  process.exit(code ?? 0);
});
