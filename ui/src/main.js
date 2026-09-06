/*
 * Hearth — the smallest interface that makes the argument.
 *
 * Two rules this file is built around:
 *
 * 1. Offline is not a failure state. There are no sync indicators, no
 *    "reconnecting" banners and no staleness warnings anywhere in here. If a
 *    device is closed, nothing is wrong: that person is living their life. An
 *    exhausted carer does not need software implying she has fallen behind.
 *
 * 2. An acknowledgement proves that a key asserted it had read a version. It
 *    does NOT prove anybody's profession. The word "claimed" earns its place
 *    every time it appears below.
 */

import { AppWebsocket, encodeHashToBase64 } from "@holochain/client";

/*
 * Identifiers are bytes inside Holochain and text everywhere a person can see
 * them. A Uint8Array stringifies to "132,32,36,..." and does not survive JSON,
 * so anything shown, copied, pasted or sent must go through here first.
 */
const asText = (hash) => (hash ? encodeHashToBase64(hash) : "");

const ROLE = "aboutme";
const ZOME = "aboutme";

let client;
let circle = null; // the cloned cell we are showing
let circles = []; // every circle this person is in
let me = null; // our AgentPubKey
let holder = null; // whose circle this is
let record = null; // the current About Me record

const $ = (id) => document.getElementById(id);

/** Whether this is your circle. Compares text, never byte arrays. */
const isHolder = () => Boolean(holder) && holder === asText(me);

/** Tell screen reader users what just happened, without stealing focus. */
function announce(message) {
  $("announcer").textContent = message;
}

function show(...ids) {
  for (const id of ["starting", "no-circle", "circles", "circle", "problem"]) {
    $(id).hidden = !ids.includes(id);
  }
}

function problem(error) {
  console.error(error);
  $("problem-detail").textContent = String(error?.message ?? error);
  show("problem");
}

const FIELDS = [
  ["what_matters_to_me", "What matters to me"],
  ["how_to_communicate_with_me", "How to talk with me"],
  ["how_to_support_me", "How to help me feel at ease"],
  ["people_who_matter", "People who matter to me"],
];

async function call(fnName, payload, cellId) {
  return client.callZome({
    ...(cellId ? { cell_id: cellId } : { role_name: ROLE }),
    zome_name: ZOME,
    fn_name: fnName,
    payload,
  });
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

function renderRecord(current) {
  const entry = current?.record?.entry?.Present?.entry;
  if (!entry) {
    $("no-record").hidden = false;
    $("record").hidden = true;
    return;
  }

  $("no-record").hidden = true;
  $("record").hidden = false;
  $("record-name").textContent = entry.display_name;
  $("relationship-whom").textContent = entry.display_name;
  $("suggest-whose").textContent = entry.display_name;

  const list = $("record-fields");
  list.replaceChildren();
  for (const [key, label] of FIELDS) {
    if (!entry[key]?.trim()) continue;
    const dt = document.createElement("dt");
    dt.textContent = label;
    const dd = document.createElement("dd");
    dd.textContent = entry[key];
    list.append(dt, dd);
  }

  // Two people editing while apart both produce valid versions. Say so rather
  // than quietly picking a winner and pretending there was never a question.
  const note = $("version-note");
  if (current.version_count > 1) {
    note.hidden = false;
    note.textContent =
      `This was changed in ${current.version_count} places while devices ` +
      `were apart. You are seeing the most recent.`;
  } else {
    note.hidden = true;
  }
}

function renderReaders(records) {
  const section = $("readers");
  const list = $("readers-list");
  list.replaceChildren();

  if (!records.length) {
    section.hidden = true;
    return;
  }
  section.hidden = false;

  for (const r of records) {
    const entry = r?.entry?.Present?.entry;
    if (!entry) continue;
    const li = document.createElement("li");
    // Never "Read by District Nurse" — that implies a credential nobody
    // checked. The claim and the claimant are shown as separate facts.
    const who = describe(r.signed_action.hashed.content.author);
    li.textContent = `${who} read this. Role claimed: ${entry.role}`;
    list.append(li);
  }
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

async function loadCircle() {
  const originals = await call("get_circle_about_me", null, circle.cellId);

  if (!originals.length) {
    record = null;
    renderRecord(null);
    $("record-actions").hidden = false;
    $("acknowledge").hidden = true;
    $("edit-record").textContent = "Write it";
    renderReaders([]);
    return;
  }

  const original = originals[0];
  const current = await call("get_current_about_me", original, circle.cellId);
  record = { original, current };
  renderRecord(current);

  const amHolder = isHolder();
  $("record-actions").hidden = false;
  $("edit-record").hidden = !amHolder;
  $("edit-record").textContent = "Change this";
  $("acknowledge").hidden = amHolder;
  $("invite-section").hidden = !amHolder;

  const readers = await call(
    "get_acknowledgements",
    record.current.record.signed_action.hashed.hash,
    circle.cellId,
  );
  renderReaders(readers);

  await loadMembers();
  await loadSuggestions();
}

async function loadSuggestions() {
  const amHolder = isHolder();

  // The person whose circle it is edits the record directly; everyone else
  // offers. Both see the list, so a carer can tell that what she noticed was
  // used.
  $("suggest-section").hidden = amHolder;

  suggestions = await call("get_suggestions", null, circle.cellId);
  renderSuggestions();
}

function fillForm() {
  const entry = record?.current?.record?.entry?.Present?.entry;
  $("display-name").value = entry?.display_name ?? "";
  $("what-matters").value = entry?.what_matters_to_me ?? "";
  $("how-to-communicate").value = entry?.how_to_communicate_with_me ?? "";
  $("how-to-support").value = entry?.how_to_support_me ?? "";
  $("people-who-matter").value = entry?.people_who_matter ?? "";
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

$("create-circle-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    // Two different names, and conflating them is a real mistake. The full
    // name is what a nurse or social worker reads and goes in the record.
    // "Mum" is a private label on this device — the clone's name is set by
    // each member for themselves, so it never travels.
    const fullName = $("person-name").value.trim();
    const label = $("circle-name").value.trim() || fullName;

    const cell = await call("create_circle", {
      founder: asText(me),
      name: label,
      network_seed: crypto.randomUUID(),
    });
    circle = { cellId: cell.cell_id };
    holder = asText(me);
    $("circle-heading").textContent = label;

    // Start the record with their name in it, so it is never nameless.
    await call(
      "create_about_me",
      {
        display_name: fullName,
        what_matters_to_me: "",
        how_to_communicate_with_me: "",
        how_to_support_me: "",
        people_who_matter: "",
      },
      circle.cellId,
    );

    circles.push({ cellId: circle.cellId, name: label });
    $("back-to-circles").hidden = circles.length < 2;
    show("circle");
    announce(`Circle made for ${fullName}.`);
    await loadCircle();
  } catch (error) {
    problem(error);
  }
});

$("edit-record").addEventListener("click", () => {
  fillForm();
  $("record-form").hidden = false;
  $("record-actions").hidden = true;
  $("cancel-edit").hidden = !record;
  $("display-name").focus();
});

$("cancel-edit").addEventListener("click", () => {
  $("record-form").hidden = true;
  $("record-actions").hidden = false;
  $("edit-record").focus();
});

$("record-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    const aboutMe = {
      display_name: $("display-name").value.trim(),
      what_matters_to_me: $("what-matters").value,
      how_to_communicate_with_me: $("how-to-communicate").value,
      how_to_support_me: $("how-to-support").value,
      people_who_matter: $("people-who-matter").value,
    };

    if (record) {
      const head = record.current.record.signed_action.hashed.hash;
      await call(
        "update_about_me",
        {
          original_action_hash: record.original,
          previous_action_hash: head,
          about_me: aboutMe,
        },
        circle.cellId,
      );
    } else {
      await call("create_about_me", aboutMe, circle.cellId);
    }

    $("record-form").hidden = true;
    announce("Saved.");
    await loadCircle();
  } catch (error) {
    problem(error);
  }
});

$("acknowledge").addEventListener("click", async () => {
  try {
    const role = window.prompt("What should they know you are?", "") ?? "";
    await call(
      "acknowledge",
      {
        about_me: record.current.record.signed_action.hashed.hash,
        role: role.trim(),
      },
      circle.cellId,
    );
    announce("Marked as read.");
    await loadCircle();
  } catch (error) {
    problem(error);
  }
});

$("invite-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    const invitee = $("invitee").value.trim();
    const invitation = await call("invite", invitee, circle.cellId);
    const output = $("invitation-output");
    output.hidden = false;
    output.textContent = JSON.stringify(invitation);
    announce("Invitation ready.");
  } catch (error) {
    problem(error);
  }
});

// ---------------------------------------------------------------------------
// Start
// ---------------------------------------------------------------------------

async function start() {
  client = await AppWebsocket.connect();

  const info = await client.appInfo();
  me = info.agent_pub_key;
  $("my-identifier").textContent = asText(me);

  await loadCircles();

  // Someone read the record. Told to us by their device, not by a server.
  client.on("signal", async (signal) => {
    const payload = signal?.payload ?? signal;
    if (payload?.kind === "Acknowledged") {
      announce(`Someone read this. They said they are: ${payload.role}`);
      if (circle) await loadCircle();
    }
    if (payload?.kind === "Suggested") {
      announce("Someone has suggested something for the record.");
      if (circle) await loadCircle();
    }
  });
}

start().catch(problem);

// ---------------------------------------------------------------------------
// Suggestions
// ---------------------------------------------------------------------------

const FIELD_LABELS = {
  WhatMattersToMe: ["what_matters_to_me", "What matters to me"],
  HowToCommunicateWithMe: ["how_to_communicate_with_me", "How to talk with me"],
  HowToSupportMe: ["how_to_support_me", "How to help me feel at ease"],
  PeopleWhoMatter: ["people_who_matter", "People who matter to me"],
};

let suggestions = [];

function renderSuggestions() {
  const list = $("suggestions-list");
  list.replaceChildren();
  $("suggestions-section").hidden = suggestions.length === 0;

  const amHolder = isHolder();

  for (const item of suggestions) {
    const entry = item.suggestion?.entry?.Present?.entry;
    if (!entry) continue;

    const author = item.suggestion.signed_action.hashed.content.author;
    const mine = asText(author) === asText(me);
    const [, label] = FIELD_LABELS[entry.field] ?? [null, entry.field];

    const li = document.createElement("li");
    li.className = "suggestion";

    const who = document.createElement("p");
    who.className = "who";
    // No names exist in this build. Saying "someone" is honest; inventing a
    // name would not be.
    who.textContent = `${describe(author)} suggested this for “${label}”`;
    li.append(who);

    const text = document.createElement("p");
    text.textContent = entry.text;
    li.append(text);

    if (entry.because?.trim()) {
      const because = document.createElement("p");
      because.className = "because";
      because.textContent = entry.because;
      li.append(because);
    }

    const outcome = item.outcome?.entry?.Present?.entry;
    if (outcome) {
      const decided = document.createElement("p");
      decided.className = "outcome";
      // Set aside is shown, never hidden. Somebody took the trouble to notice
      // something; letting it vanish silently is how people stop noticing.
      decided.textContent = outcome.accepted
        ? "Added to the record."
        : "Set aside for now. Thank you for offering it.";
      li.append(decided);
    } else if (amHolder) {
      const actions = document.createElement("div");
      actions.className = "actions";

      const accept = document.createElement("button");
      accept.type = "button";
      accept.textContent = "Add this";
      accept.addEventListener("click", () =>
        decide(item, entry, true).catch(problem),
      );

      const setAside = document.createElement("button");
      setAside.type = "button";
      setAside.className = "secondary";
      setAside.textContent = "Not this one";
      setAside.addEventListener("click", () =>
        decide(item, entry, false).catch(problem),
      );

      actions.append(accept, setAside);
      li.append(actions);
    } else {
      const waiting = document.createElement("p");
      waiting.className = "who";
      waiting.textContent = "Not looked at yet.";
      li.append(waiting);
    }

    list.append(li);
  }
}

/** Accept or set aside. Accepting also puts the words into the record. */
async function decide(item, entry, accepted) {
  const hash = item.suggestion.signed_action.hashed.hash;

  await call("decide_on_suggestion", { suggestion: hash, accepted }, circle.cellId);

  if (accepted && record) {
    const [key] = FIELD_LABELS[entry.field] ?? [];
    const current = record.current.record.entry.Present.entry;
    const existing = current[key]?.trim();

    await call(
      "update_about_me",
      {
        original_action_hash: record.original,
        previous_action_hash: record.current.record.signed_action.hashed.hash,
        about_me: {
          ...current,
          [key]: existing ? `${existing}\n${entry.text}` : entry.text,
        },
      },
      circle.cellId,
    );
  }

  announce(accepted ? "Added to the record." : "Set aside.");
  await loadCircle();
}

$("suggest-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    await call(
      "suggest",
      {
        field: $("suggest-field").value,
        text: $("suggest-text").value.trim(),
        because: $("suggest-because").value.trim(),
      },
      circle.cellId,
    );
    $("suggest-form").reset();
    announce("Offered. The person who holds this circle will see it.");
    await loadCircle();
  } catch (error) {
    problem(error);
  }
});

// ---------------------------------------------------------------------------
// Who is who
// ---------------------------------------------------------------------------

let members = new Map(); // agent key string -> { name, relationship }

/**
 * How to refer to somebody. Never invents an identity: an unintroduced member
 * is "someone in the circle", which is true, rather than a guess dressed up as
 * a fact.
 */
function describe(agentKey) {
  const key = asText(agentKey);
  if (key === asText(me)) return "You";
  const member = members.get(key);
  if (!member) return "Someone in the circle";
  return member.relationship?.trim()
    ? `${member.name} (${member.relationship})`
    : member.name;
}

async function loadMembers() {
  const records = await call("get_members", null, circle.cellId);
  members = new Map();
  for (const r of records) {
    const entry = r?.entry?.Present?.entry;
    if (!entry) continue;
    // Latest introduction wins; people correct how they describe themselves.
    members.set(asText(r.signed_action.hashed.content.author), entry);
  }

  const mine = members.get(asText(me));
  $("introduce-section").hidden = false;
  if (mine) {
    $("member-name").value = mine.name;
    $("member-relationship").value = mine.relationship ?? "";
  }
}

$("introduce-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    await call(
      "introduce_myself",
      {
        name: $("member-name").value.trim(),
        relationship: $("member-relationship").value.trim(),
      },
      circle.cellId,
    );
    announce("Saved.");
    await loadCircle();
  } catch (error) {
    problem(error);
  }
});

// ---------------------------------------------------------------------------
// Joining a circle you were invited to
// ---------------------------------------------------------------------------

$("join-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    const bundle = JSON.parse($("invitation-in").value.trim());
    const label = $("join-label").value.trim() || bundle.about || "Circle";

    const cell = await call("join_circle", {
      founder: bundle.founder,
      name: label,
      network_seed: bundle.network_seed,
      invitation: bundle.invitation,
    });

    circle = { cellId: cell.cell_id };
    holder = bundle.founder; // already text, out of the invitation
    $("circle-heading").textContent = label;
    circles.push({ cellId: circle.cellId, name: label });
    $("back-to-circles").hidden = circles.length < 2;
    show("circle");
    announce(`You have joined ${bundle.about || "the circle"}.`);
    await loadCircle();
  } catch (error) {
    // The commonest cause by far is a half-copied invitation.
    problem(
      error?.message?.includes("JSON")
        ? new Error("That invitation looks incomplete. Copy the whole of it.")
        : error,
    );
  }
});

$("copy-identifier").addEventListener("click", async () => {
  try {
    await navigator.clipboard.writeText(asText(me));
    announce("Copied. Send it to whoever is inviting you.");
  } catch {
    // Clipboard access can be refused. Selecting the text still works, so say
    // so rather than failing silently.
    announce("Could not copy. Select the text and copy it yourself.");
  }
});


// ---------------------------------------------------------------------------
// The list of people
// ---------------------------------------------------------------------------

/** Every circle this person belongs to. Circles are clones of the lobby. */
async function loadCircles() {
  const info = await client.appInfo();
  const cells = info.cell_info[ROLE] ?? [];

  circles = cells
    .map((c) => c?.value ?? c?.cloned ?? c)
    .filter((c) => c?.clone_id || c?.original_dna_hash)
    .map((c) => ({ cellId: c.cell_id, name: c.name || "Circle" }));

  if (circles.length === 0) {
    show("no-circle");
    $("person-name").focus();
    return;
  }

  renderCircles();
  show("circles");
}

function renderCircles() {
  const list = $("circles-list");
  list.replaceChildren();

  for (const item of circles) {
    const li = document.createElement("li");
    const button = document.createElement("button");
    button.type = "button";
    button.className = "circle-link";
    // Just their name. No counts, no badges, no "2 new". She is looking
    // somebody up, not clearing a queue.
    button.textContent = item.name;
    button.addEventListener("click", () => openCircle(item).catch(problem));
    li.append(button);
    list.append(li);
  }
}

async function openCircle(item) {
  circle = { cellId: item.cellId };

  // The holder is named in the cell's own properties, so a circle you joined
  // reads correctly rather than assuming you hold everything.
  try {
    holder = asText(await call("who_holds_this", null, circle.cellId));
  } catch {
    holder = null;
  }

  $("circle-heading").textContent = item.name;
  $("back-to-circles").hidden = circles.length < 2;
  show("circle");
  await loadCircle();
}

$("back-to-circles").addEventListener("click", () => {
  renderCircles();
  show("circles");
});

$("add-circle").addEventListener("click", () => {
  show("no-circle");
  $("person-name").focus();
});
