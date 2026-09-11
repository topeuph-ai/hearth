/*
 * Does every button on the page do anything when pressed?
 *
 * Written after a "Back" button sat on two screens doing nothing. The line
 * that wired it had no id in it — it found the buttons by class — so it sat
 * between two lines being cut out together and went with them, and every
 * check we had looked straight past it. A name in the code with no element on
 * the page is caught; an element on the page with no code behind it was not.
 *
 * It is a static check on the two files, not a running browser, so it is
 * cheap enough for CI and for a shell prompt. It cannot know what a handler
 * does. It can know whether one exists, which is the whole of the fault it
 * was written for.
 *
 * A button counts as wired if its id is used in main.js, or if one of its
 * classes is used as a selector there, or if it is a submit button inside a
 * form the code listens to. Buttons made in JS are not in index.html at all
 * and are not its business.
 *
 *     node scripts/every-button-does-something.mjs
 */
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ui = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const html = readFileSync(join(ui, "index.html"), "utf8");
const js = readFileSync(join(ui, "src", "main.js"), "utf8");

/** Every class main.js hands to a query selector. */
const selectorClasses = new Set(
  [...js.matchAll(/querySelector(?:All)?\(\s*"\.([A-Za-z0-9_-]+)"/g)].map(
    (m) => m[1],
  ),
);

/**
 * Every id main.js mentions at all.
 *
 * Deliberately any string literal, not only `$("...")`. Some buttons are wired
 * by a helper that takes the id as an argument — wireCopyButton does — and a
 * check that flagged those would be a check people learn to ignore.
 */
const wantedIds = new Set(
  [...js.matchAll(/"([A-Za-z][A-Za-z0-9-]*)"/g)].map((m) => m[1]),
);

/** Forms whose submit main.js listens to. */
const listenedForms = new Set(
  [
    ...js.matchAll(
      /\$\("([^"]+)"\)\.addEventListener\(\s*"submit"/g,
    ),
  ].map((m) => m[1]),
);

// Which form each button sits in, by walking the markup in order. Crude and
// sufficient: forms in this file do not nest.
const problems = [];
let openForm = null;

const tokens = html.matchAll(
  /<form\b[^>]*>|<\/form>|<button\b[^>]*>/g,
);

for (const token of tokens) {
  const tag = token[0];

  if (tag.startsWith("</form")) {
    openForm = null;
    continue;
  }
  if (tag.startsWith("<form")) {
    openForm = tag.match(/\bid="([^"]+)"/)?.[1] ?? null;
    continue;
  }

  const id = tag.match(/\bid="([^"]+)"/)?.[1] ?? null;
  const classes = (tag.match(/\bclass="([^"]+)"/)?.[1] ?? "").split(/\s+/);
  const type = tag.match(/\btype="([^"]+)"/)?.[1] ?? "submit";
  const label = id ? `#${id}` : `button.${classes.join(".") || "(no class)"}`;

  if (id && wantedIds.has(id)) continue;
  if (classes.some((c) => selectorClasses.has(c))) continue;
  if (type === "submit" && openForm && listenedForms.has(openForm)) continue;

  problems.push(
    `${label}${openForm ? ` in #${openForm}` : ""} — nothing listens to it`,
  );
}

if (problems.length) {
  console.error("Buttons that do nothing when pressed:\n");
  for (const p of problems) console.error("  " + p);
  console.error(
    "\nEither wire it up or take it off the page. A button that does nothing",
  );
  console.error("is worse than no button: it is pressed, and then pressed again.");
  process.exit(1);
}

console.log("Every button on the page has something behind it.");
