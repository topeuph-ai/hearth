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

/*
 * An invitation is one opaque line of text.
 *
 * It used to be raw JSON, which failed twice: a Uint8Array becomes
 * {"0":196,"1":93,...} when stringified, so the signature inside arrived as an
 * object of numbered keys and could never be used. And a multi-line JSON blob
 * is easy to half-copy out of a text message.
 *
 * One line. Nothing inside it a person can damage by accident.
 */
const bytesToBase64 = (bytes) =>
  btoa(String.fromCharCode(...new Uint8Array(bytes)));

const base64ToBytes = (text) =>
  Uint8Array.from(atob(text), (c) => c.charCodeAt(0));


/*
 * Two long strings pass between people and they look identical to a human: an
 * identifier (uhCAk...) and an invitation (base64 of a small bundle). Pasting
 * one where the other belongs is the obvious mistake, and the obvious mistake
 * deserves a sentence that explains the flow rather than "could not be read".
 */
const looksLikeAnIdentifier = (text) => /^uhCAk[A-Za-z0-9_-]{40,}$/.test(text.trim());

function looksLikeAnInvitation(text) {
  try {
    return typeof JSON.parse(atob(text.trim()))?.founder === "string";
  } catch {
    return false;
  }
}

function invitationToToken(bundle) {
  return btoa(
    JSON.stringify({
      ...bundle,
      invitation: { signature: bytesToBase64(bundle.invitation.signature) },
    }),
  );
}

function tokenToInvitation(token) {
  const parsed = JSON.parse(atob(token.trim()));
  return {
    ...parsed,
    invitation: { signature: base64ToBytes(parsed.invitation.signature) },
  };
}


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

/**
 * Hold a button while its work is happening, and say what is happening on it.
 *
 * Making a circle builds a whole new encrypted space, which takes a moment.
 * Without this the button looks broken: nothing changes, so the obvious thing
 * to do is press it again — and every press made another circle. Three circles
 * called "Mam", none of them removable. The press was not the mistake; a
 * button that stays silent while it works is.
 */
async function whileWorking(button, working, task) {
  const wasLabel = button.textContent;
  button.disabled = true;
  button.textContent = working;
  announce(working);
  try {
    return await task();
  } finally {
    button.disabled = false;
    button.textContent = wasLabel;
  }
}

/** The screen to return to when something goes wrong. Never "problem". */
let lastGoodScreen = "choose";

function show(...ids) {
  if (!ids.includes("problem") && !ids.includes("starting")) {
    lastGoodScreen = ids[0];
  }
  for (const id of [
    "starting",
    "choose",
    "create",
    "join",
    "circles",
    "circle",
    "problem",
  ]) {
    $(id).hidden = !ids.includes(id);
  }
}

function problem(error) {
  console.error(error);
  $("problem-detail").textContent = String(error?.message ?? error);
  show("problem");
  // Focus the way out, so a keyboard user is not hunting for it.
  $("go-back").focus();
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

/**
 * Put her name everywhere the interface asks somebody a question about her.
 *
 * The read view keeps "What matters to me", because that is her record in her
 * own voice. The forms ask whoever is typing, and they are usually not her.
 */
function nameHer(name) {
  const who = name?.trim() || "them";
  knownName = name?.trim() ?? knownName;
  $("relationship-whom").textContent = who;
  $("suggest-whose").textContent = who;
  for (const span of document.querySelectorAll(".about-whom")) {
    span.textContent = who;
  }
}

/*
 * Her name, held for the length of the visit.
 *
 * The write form does not ask for it: it was given when the circle was made,
 * and every label on that form is already using it. So saving has to get it
 * from somewhere, and "somewhere" cannot be the record alone — right after a
 * circle is made the record has not come back from the network yet, which is
 * the whole reason the old name box appeared empty.
 */
let knownName = "";

/**
 * The name to write into the record, in the order the app is sure of it.
 *
 * The record wins when it is here, because that is what everybody else reads.
 * What was typed when the circle was made is the fallback, for the minute
 * before the record has come back.
 */
function personName() {
  const fromRecord =
    record?.current?.record?.entry?.Present?.entry?.display_name?.trim();
  return fromRecord || knownName.trim();
}

/*
 * Whether a circle is somebody's own record is a fact about how they use this
 * device, not about the circle — a daughter and her mother could both be in it
 * with different answers. So it lives here, per device, keyed by the cell.
 */
const ownRecordKey = (cellId) => `hearth:own:${asText(cellId?.[0])}`;

function markAsOwnRecord(cellId, isOwn) {
  try {
    localStorage.setItem(ownRecordKey(cellId), isOwn ? "yes" : "no");
  } catch {
    // Private windows and locked-down browsers refuse this. It is a
    // convenience, not a rule, so carry on without it.
  }
}

function isOwnRecord(cellId) {
  try {
    return localStorage.getItem(ownRecordKey(cellId)) === "yes";
  } catch {
    return false;
  }
}

/**
 * Is there anything here a person would call written?
 *
 * Making a circle seeds a record with her name in it and four empty fields,
 * so a record exists from the first moment — which is not the same as
 * something having been written. Reading these apart matters: the screen used
 * to say "Read this over" above a page with nothing on it but a name.
 */
const hasBeenWritten = (entry) =>
  Boolean(entry) && FIELDS.some(([key]) => entry[key]?.trim());

function renderRecord(current) {
  const entry = current?.record?.entry?.Present?.entry;

  // Her name is hers whether or not anything has been written yet, and every
  // question on this screen is phrased around it.
  if (entry?.display_name) nameHer(entry.display_name);

  if (!hasBeenWritten(entry)) {
    $("no-record").hidden = false;
    $("record").hidden = true;
    return;
  }

  $("no-record").hidden = true;
  $("record").hidden = false;
  $("record-name").textContent = entry.display_name;

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
  //
  // Only for a real fork. Editing your own record twice is not two people
  // disagreeing, and saying it was is worse than saying nothing: it invites
  // somebody to go looking for a conflict that never happened.
  const note = $("version-note");
  if (current.divergent_versions > 1) {
    note.hidden = false;
    note.textContent =
      `This was written in ${current.divergent_versions} places while devices ` +
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
  /*
   * There are three states here, not two, and writing it as two branches
   * produced a screen with no message on it at all.
   *
   *   1. The holder's circle, genuinely empty until they write.
   *   2. Somebody else's circle, nothing arrived yet.
   *   3. Somebody else's circle where the *reference* has arrived but the
   *      content has not — gossip carries links and entries separately, so
   *      this is a real and ordinary state, not a glitch.
   *
   * State 3 fell between the two branches: a record existed, so the empty
   * branch never ran, but there was nothing to render, so the reader saw an
   * empty box and a "Check again" button that never turned itself off.
   */
  const amHolder = isHolder();
  $("check-it-over").hidden = true;

  const originals = await call("get_circle_about_me", null, circle.cellId);
  const original = originals[0] ?? null;

  const current = original
    ? await call("get_current_about_me", original, circle.cellId)
    : null;

  const entry = current?.record?.entry?.Present?.entry;
  const haveIt = Boolean(entry);

  /*
   * "A record exists" and "somebody has written something" are different
   * facts, and treating them as one produced a screen that said "Read this
   * over" above a page containing nothing but a name. Making the circle seeds
   * the record so it is never nameless, which means a record exists from the
   * first second — before anybody has typed a word into it.
   *
   * So: `haveIt` decides whether there is an entry to update, which is a
   * question about the chain. `written` decides what the screen says, which
   * is a question about the person.
   */
  const written = hasBeenWritten(entry);

  record = haveIt ? { original, current } : null;
  renderRecord(haveIt ? current : null);

  $("no-record-empty").hidden = written || !amHolder;
  $("no-record-waiting").hidden = written || amHolder;

  $("record-actions").hidden = false;
  $("edit-record").hidden = !amHolder;
  $("edit-record").textContent = written ? "Change this" : "Write it";
  $("acknowledge").hidden = amHolder || !written;
  // Only offer this while there is actually something to wait for.
  $("check-again").hidden = amHolder || written;
  // Nothing to invite anybody to until something has been written. A name and
  // four empty headings is a confusing thing to be invited into.
  $("invite-section").hidden = !amHolder || !written;

  renderReaders(
    written
      ? await call(
          "get_acknowledgements",
          current.record.signed_action.hashed.hash,
          circle.cellId,
        )
      : [],
  );

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
    // Two names on this page, and they are two different people: the one the
    // circle is about, and the one filling it in. Conflating them is a real
    // mistake, so they are asked separately and kept separately.
    const fullName = $("person-name").value.trim();
    const myOwn = $("about-me").checked;
    // What this device lists her under. Hers alone: the clone's name is set by
    // each member for themselves, so it never travels.
    const label = (myOwn ? "" : $("circle-name").value.trim()) || fullName;
    // Nobody introduces themselves to their own record.
    const carerName = myOwn ? "" : $("carer-name").value.trim();
    const carerRelationship = myOwn ? "" : $("carer-relationship").value.trim();

    const cell = await whileWorking(
      $("create-circle-submit"),
      "Making the circle…",
      () =>
        call("create_circle", {
          founder: asText(me),
          name: label,
          network_seed: crypto.randomUUID(),
        }),
    );
    circle = { cellId: cell.cell_id };
    holder = asText(me);
    markAsOwnRecord(circle.cellId, myOwn);
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

    // She has just told us who she is, so do not ask again inside the circle.
    // Nothing here is checked; it is how she describes herself, and how she
    // relates to the person is a separate question asked later.
    if (carerName) {
      await call(
        "introduce_myself",
        { name: carerName, relationship: carerRelationship },
        circle.cellId,
      );
    }

    circles.push({ cellId: circle.cellId, name: label });
    $("back-to-circles").hidden = circles.length < 2;
    show("circle");
    announce(`Circle made for ${fullName}. Now write what people should know.`);
    await loadCircle();

    // Straight into writing it. An empty circle is not much use to anybody,
    // and inviting people to nothing is worse.
    nameHer(fullName);
    fillForm();
    $("record-form").hidden = false;
    $("record-actions").hidden = true;
    $("what-matters").focus();
  } catch (error) {
    problem(error);
  }
});

$("edit-record").addEventListener("click", () => {
  fillForm();
  $("record-form").hidden = false;
  $("record-actions").hidden = true;
  $("what-matters").focus();
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
      display_name: personName(),
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
    await loadCircle();

    // The holder has just written it, so show it back to them to check before
    // anybody else is asked to rely on it — but only if there is something to
    // read. Saving a form with every box empty is a real thing to do, usually
    // by accident, and "Read this over" pointing at a bare name is the screen
    // telling somebody to check work that does not exist.
    const somethingToRead = hasBeenWritten(aboutMe);

    if (isHolder() && somethingToRead) {
      $("check-it-over").hidden = false;
      announce("Saved. Read it over, then invite people.");
    } else if (isHolder()) {
      announce(`Saved. Nothing has been written about ${aboutMe.display_name.trim() || "them"} yet.`);
    } else {
      announce("Saved.");
    }
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

    if (looksLikeAnInvitation(invitee)) {
      throw new Error(
        "That is an invitation, not an identifier. This box wants the long " +
          "line beginning uhCAk that they copied from their own Hearth.",
      );
    }
    if (!looksLikeAnIdentifier(invitee)) {
      throw new Error(
        "That does not look like an identifier. It is one long line beginning " +
          "uhCAk, which they copy from “Your identifier” at the top of their " +
          "own Hearth. It is not their name.",
      );
    }

    const invitation = await call("invite", invitee, circle.cellId);
    const output = $("invitation-output");
    output.hidden = false;
    output.textContent = invitationToToken(invitation);
    $("copy-invitation").hidden = false;
    $("invitee").value = "";
    announce("Invitation ready. Send it to them however you like.");
  } catch (error) {
    problem(error);
  }
});

// ---------------------------------------------------------------------------
// Start
// ---------------------------------------------------------------------------

/*
 * A connection that hangs is worse than one that fails.
 *
 * Waiting forever leaves "Starting up." on screen with no error and no way to
 * tell whether anything is wrong. Give it a deadline, and say so when it
 * passes.
 */
function withTimeout(promise, seconds, what) {
  return Promise.race([
    promise,
    new Promise((_, reject) =>
      setTimeout(
        () =>
          reject(
            new Error(
              `${what} took longer than ${seconds} seconds. Hearth could not ` +
                `reach its own conductor. Closing and reopening usually helps.`,
            ),
          ),
        seconds * 1000,
      ),
    ),
  ]);
}

async function start() {
  client = await withTimeout(AppWebsocket.connect(), 20, "Connecting");

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

  /*
   * Only for somebody who has not said yet.
   *
   * Whoever made the circle answered both of these on the way in — her name,
   * and how she is connected to the person — so putting them up again inside
   * the circle asks a person to introduce herself to a record she has just
   * written. Somebody who joined by invitation has not been asked, and this
   * is where they are.
   *
   * Asked of my own chain, not of the list above. That list comes from the
   * network, and in the seconds after introducing myself my own entry is not
   * in it yet — so this section reappeared, empty, on the screen I had just
   * filled in. What I have said about myself is never a question for the
   * network.
   */
  const mine = await call("my_introduction", null, circle.cellId);

  // And put myself in the list for the same reason: until my introduction
  // comes back over the network I am in my own circle as a stranger.
  if (mine) members.set(asText(me), mine);

  $("introduce-section").hidden = Boolean(mine);
  $("relationship-field").hidden = isOwnRecord(circle.cellId);
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
    const pasted = $("invitation-in").value.trim();

    // Their own identifier is the likeliest paste of all: it is at the top of
    // this very screen, a few centimetres above the box. Say so exactly rather
    // than hedging with "it may even be your own".
    if (pasted === asText(me)) {
      throw new Error(
        "That is your own identifier. It does not go here — it goes to the " +
          "other person. Send it to them, they paste it into their Hearth, " +
          "and what comes back is what belongs in this box.",
      );
    }

    if (looksLikeAnIdentifier(pasted)) {
      throw new Error(
        "That is somebody's identifier, not an invitation. An identifier only " +
          "says who a person is. Send yours to whoever holds the circle, and " +
          "they will send back an invitation.",
      );
    }

    const bundle = tokenToInvitation(pasted);
    const label = $("join-label").value.trim() || bundle.about || "Circle";

    const cell = await whileWorking(
      $("join-circle-submit"),
      "Joining…",
      () =>
        call("join_circle", {
          founder: bundle.founder,
          name: label,
          network_seed: bundle.network_seed,
          invitation: bundle.invitation,
        }),
    );

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
    const damaged =
      error instanceof SyntaxError ||
      /JSON|atob|InvalidCharacter/i.test(String(error?.message ?? ""));
    problem(
      damaged
        ? new Error(
            "That invitation could not be read. Copy the whole of it — it is " +
              "one long line, with nothing before or after.",
          )
        : error,
    );
  }
});

/**
 * Wire up a copy button so that pressing it visibly does something.
 *
 * The first version only called announce(), which writes to a screen-reader
 * live region. A sighted person pressed it, saw nothing at all, and reasonably
 * concluded it was broken. Feedback that exists only for assistive technology
 * is not feedback.
 */
function wireCopyButton(buttonId, getText, doneLabel = "Copied") {
  const button = $(buttonId);
  if (!button) return;
  const original = button.textContent;
  let resetAfter;

  button.addEventListener("click", async () => {
    const text = getText();
    let ok = true;
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      ok = false;
    }

    // Say it on the button, and say it to a screen reader. Both, always.
    button.textContent = ok ? doneLabel : "Press Ctrl+C instead";
    announce(
      ok
        ? `${doneLabel}. Send it to them however you like.`
        : "Could not copy. Select the text and copy it yourself.",
    );

    clearTimeout(resetAfter);
    resetAfter = setTimeout(() => {
      button.textContent = original;
    }, 2500);
  });
}

wireCopyButton("copy-identifier", () => asText(me), "Copied");


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
    // A circle taken off this device is disabled, not deleted, so the
    // conductor still lists it. It should not be on her screen.
    .filter((c) => c.enabled !== false)
    .map((c) => ({ cellId: c.cell_id, name: c.name || "Circle" }));

  if (circles.length === 0) {
    show("choose");
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

  // Forget the last person before showing this one. A name carried over from
  // the circle just closed could otherwise be written into this one, which is
  // the worst thing a record about a person could get wrong.
  knownName = "";
  record = null;

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

$("leave-circle").addEventListener("click", async () => {
  try {
    const leaving = $("circle-heading").textContent;

    await whileWorking($("leave-circle"), "Taking it off…", () =>
      // Sent to the lobby cell, not to the circle: a cell cannot be the one to
      // switch itself off.
      call("leave_circle", circle.cellId[0]),
    );

    circle = null;
    holder = null;
    $("leave-details").open = false;

    // Straight back to the list, which is where she was heading. If that was
    // the last one, loadCircles puts her at the first question again rather
    // than an empty page with nothing on it.
    await loadCircles();
    announce(`${leaving} is off this device.`);
  } catch (error) {
    problem(error);
  }
});

$("back-to-circles").addEventListener("click", () => {
  renderCircles();
  show("circles");
});

$("add-circle").addEventListener("click", () => {
  show("choose");
});

wireCopyButton(
  "copy-invitation",
  () => $("invitation-output").textContent,
  "Invitation copied",
);

/*
 * Two questions on the create form name her, and she is being typed in right
 * above them. "How are you connected to them?" and "What do you call them?"
 * are both ambiguous while two people are being named on one page, so neither
 * is allowed to say "them" once there is a name to use.
 */
$("person-name").addEventListener("input", () => {
  const who = $("person-name").value.trim() || "them";
  $("create-relationship-whom").textContent = who;
  $("call-them-whom").textContent = who;
});

$("go-back").addEventListener("click", () => {
  // Back to where they were, with whatever they typed still in the fields.
  // A mistyped character should cost a correction, not a restart.
  show(lastGoodScreen);
});

// ---------------------------------------------------------------------------
// Choosing what to do
// ---------------------------------------------------------------------------

$("choose-create").addEventListener("click", () => {
  show("create");
  $("person-name").focus();
});

$("choose-join").addEventListener("click", () => {
  show("join");
  $("invitation-in").focus();
});

// Every form has a way out. Getting somewhere by accident should cost one
// press to undo, not a restart.
for (const button of document.querySelectorAll(".back-to-choose")) {
  button.addEventListener("click", () => show("choose"));
}

$("check-again").addEventListener("click", async () => {
  const button = $("check-again");
  const original = button.textContent;
  button.textContent = "Looking...";
  try {
    await loadCircle();
    // If it still is not here, that is not a failure. It means the other
    // device has not been on since it was written.
    if (!record) announce("Still not here yet.");
  } catch (error) {
    problem(error);
  } finally {
    button.textContent = original;
  }
});

// ---------------------------------------------------------------------------
// Whose circle is this?
// ---------------------------------------------------------------------------

function updateWhoseCircle() {
  const mine = $("about-me").checked;
  $("person-name-label").textContent = mine
    ? "Your full name"
    : "Their full name";
  $("person-name-hint").textContent = mine
    ? "As a nurse or social worker would need to see it."
    : "As a nurse or social worker would need to see it. Everyone in the circle reads this one.";
  // Your own record has one person in it, and you have already named them.
  // There is nobody to be connected to, and nobody calls themselves a
  // nickname on their own device.
  for (const id of [
    "carer-name-field",
    "carer-relationship-field",
    "private-label-field",
  ]) {
    $(id).hidden = mine;
  }
}

for (const id of ["about-me", "about-someone-else"]) {
  $(id).addEventListener("change", updateWhoseCircle);
}
updateWhoseCircle();
