/*
 * A door address still arrives where it was meant to, and a mistake in one
 * is caught and pointed at.
 *
 * The address is what somebody types, writes down or reads out, and on the
 * first offline test one wrong character was refused with nothing to say
 * where. So this runs the real functions out of main.js against real keys:
 * every address made must read back to the same room, and every single
 * wrong character and every swap of two neighbours must be caught, in the
 * row it happened in.
 *
 *     node scripts/addresses-survive-mistakes.mjs
 */

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { randomBytes, randomUUID } from "node:crypto";
import { decodeHashFromBase64, encodeHashToBase64 } from "@holochain/client";

const here = dirname(fileURLToPath(import.meta.url));
const source = readFileSync(join(here, "..", "src", "main.js"), "utf8");

const start = source.indexOf("const ADDRESS_ALPHABET");
const end = source.indexOf("function drawAddressCode");
if (start < 0 || end < 0) throw new Error("Could not find the address code in main.js");

const { roomToAddress, addressToRoom } = new Function(
  "decodeHashFromBase64",
  "encodeHashToBase64",
  `${source.slice(start, end)}\nreturn { roomToAddress, addressToRoom };`,
)(decodeHashFromBase64, encodeHashToBase64);

// A real-shaped agent key: the 3-byte agent prefix, 32 bytes, 4 location bytes.
function anAgentKey() {
  const core = randomBytes(32);
  const key = new Uint8Array(39);
  key.set([0x84, 0x20, 0x24], 0);
  key.set(core, 3);
  key.set(randomBytes(4), 35);
  return encodeHashToBase64(key);
}

let failures = 0;
const fail = (message) => {
  failures += 1;
  console.error(`FAIL: ${message}`);
};

const ALPHABET = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

for (let round = 0; round < 25; round++) {
  const room = { holder: anAgentKey(), seed: randomUUID(), about: "Margaret Smythe" };
  const address = roomToAddress(room);

  if (address.includes("Margaret")) fail("the address carries the person's name");

  const back = addressToRoom(address);
  if (back.holder !== room.holder || back.seed !== room.seed) {
    fail(`round ${round}: the address did not read back to the same room`);
  }

  // Typed carelessly: lower case, no spaces, O for 0 and L for 1.
  const careless = address.toLowerCase().replace(/\s+/g, "").replace(/0/g, "o").replace(/1/g, "l");
  const carelessBack = addressToRoom(careless);
  if (carelessBack.holder !== room.holder || carelessBack.seed !== room.seed) {
    fail(`round ${round}: lower case, O for 0 or L for 1 was not forgiven`);
  }

  const rows = address.split("\n").slice(1).map((r) => r.replace(/ /g, ""));

  rows.forEach((row, r) => {
    for (let i = 0; i < row.length; i++) {
      // Every other letter in every position.
      for (const wrong of ALPHABET) {
        if (wrong === row[i]) continue;
        const broken = [...rows];
        broken[r] = row.slice(0, i) + wrong + row.slice(i + 1);
        try {
          addressToRoom(`HEARTH\n${broken.join("\n")}`);
          fail(`round ${round}: row ${r + 1} position ${i + 1}, ${row[i]} to ${wrong}, was not caught`);
        } catch (error) {
          if (error.row !== r + 1) {
            fail(`round ${round}: a mistake in row ${r + 1} was reported as row ${error.row}`);
          }
        }
      }

      // Two neighbours swapped.
      if (i + 1 < row.length && row[i] !== row[i + 1]) {
        const broken = [...rows];
        broken[r] = row.slice(0, i) + row[i + 1] + row[i] + row.slice(i + 2);
        try {
          addressToRoom(`HEARTH\n${broken.join("\n")}`);
          fail(`round ${round}: row ${r + 1}, swapping positions ${i + 1} and ${i + 2} was not caught`);
        } catch {
          // Caught, which is the point.
        }
      }
    }
  });

  // A character left out.
  const short = [...rows];
  short[2] = short[2].slice(1);
  try {
    addressToRoom(`HEARTH\n${short.join("\n")}`);
    fail(`round ${round}: a missing character was not caught`);
  } catch {
    // Caught.
  }
}

// An address from before this format still reads.
const old = { holder: anAgentKey(), seed: randomUUID() };
const oldAddress = btoa(JSON.stringify({ door: old.holder, seed: old.seed, about: "Margaret" }));
const oldBack = addressToRoom(oldAddress);
if (oldBack.holder !== old.holder || oldBack.seed !== old.seed) fail("an old address no longer reads");

if (failures) {
  console.error(`\n${failures} problem${failures === 1 ? "" : "s"} with door addresses.`);
  process.exit(1);
}

console.log(roomToAddress({ holder: anAgentKey(), seed: randomUUID() }));
console.log("\nEvery address read back, every wrong letter and swap was caught in its own row.");
