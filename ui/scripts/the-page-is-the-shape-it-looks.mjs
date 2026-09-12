/*
 * Is the page the shape it looks like in the source?
 *
 * Written after "Take this off my device" ended up nested inside "Waiting to
 * be let in", because an edit removed a section's opening tag and put it back
 * somewhere else. Nothing caught it: every id was still present exactly once,
 * every button still had something listening, and the browser renders
 * scrambled markup perfectly happily — so the only symptom was a section that
 * appeared and disappeared with the wrong one.
 *
 * The checks we had all asked about *names*. This asks about *nesting*, which
 * is the thing an edit that moves blocks about can break.
 *
 * No parser is needed for it: a tag stack over the source is enough to catch
 * an element closed in the wrong order or never closed at all, which is what
 * this class of mistake produces.
 *
 *     node scripts/the-page-is-the-shape-it-looks.mjs
 */
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ui = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const html = readFileSync(join(ui, "index.html"), "utf8");

/** Elements that never have a closing tag. */
const VOID = new Set([
  "area", "base", "br", "col", "embed", "hr", "img", "input",
  "link", "meta", "source", "track", "wbr",
]);

// Comments and the contents of <script>/<style> are not markup to be counted.
const source = html
  .replace(/<!--[\s\S]*?-->/g, "")
  .replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi, "")
  .replace(/<style\b[^>]*>[\s\S]*?<\/style>/gi, "");

const lineOf = (index) => source.slice(0, index).split("\n").length;

const stack = [];
const problems = [];

for (const m of source.matchAll(/<\/?([a-zA-Z][a-zA-Z0-9-]*)\b[^>]*?(\/?)>/g)) {
  const [tag, name, selfClosing] = m;
  const lower = name.toLowerCase();
  if (VOID.has(lower) || selfClosing === "/" || lower === "!doctype") continue;

  const id = tag.match(/\bid="([^"]+)"/)?.[1];

  if (tag.startsWith("</")) {
    const open = stack.pop();
    if (!open) {
      problems.push(`line ${lineOf(m.index)}: </${lower}> closes nothing`);
    } else if (open.name !== lower) {
      problems.push(
        `line ${lineOf(m.index)}: </${lower}> closes ` +
          `<${open.name}${open.id ? ` id="${open.id}"` : ""}> ` +
          `opened on line ${open.line} — one of them is in the wrong place`,
      );
      // Keep going rather than cascading: put it back and report once.
      stack.push(open);
    }
  } else {
    stack.push({ name: lower, id, line: lineOf(m.index) });
  }
}

for (const left of stack) {
  problems.push(
    `line ${left.line}: <${left.name}${left.id ? ` id="${left.id}"` : ""}> is never closed`,
  );
}

if (problems.length) {
  console.error("The page is not the shape it looks like:\n");
  for (const p of problems) console.error("  " + p);
  console.error(
    "\nA browser will render this anyway, which is why nothing else noticed.",
  );
  process.exit(1);
}

console.log("Every element opens and closes where it appears to.");
