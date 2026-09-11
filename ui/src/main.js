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
// Records arrive with their entries still packed. Holochain speaks msgpack on
// the wire and does not unpack app entries for you.
import { decode } from "@msgpack/msgpack";

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

/*
 * Three long lines of characters now pass between people, and to anybody
 * looking at them they are the same thing: an identifier, an invitation, and
 * the address of a waiting room. Each has a box of its own, on a screen of its
 * own, and putting one in the wrong place is the obvious mistake.
 *
 * So each box knows the shape of the other two and says which screen the thing
 * belongs on. Without this, an address pasted into the invitation box reached
 * the invitation parser and came out as "Cannot read properties of undefined
 * (reading 'signature')" — a sentence that tells somebody nothing except that
 * they have broken it, which they have not.
 */
function looksLikeARoomAddress(text) {
  try {
    const parsed = JSON.parse(atob(text.trim()));
    return typeof parsed?.door === "string" && typeof parsed?.seed === "string";
  } catch {
    return false;
  }
}

function invitationToToken(bundle) {
  const seconded = bundle.invitation.seconded;
  return btoa(
    JSON.stringify({
      ...bundle,
      invitation: {
        signature: bytesToBase64(bundle.invitation.signature),
        // Absent until the second person has agreed. An invitation without it,
        // to a circle that asks for one, is not a weak invitation — it is an
        // unfinished one.
        seconded: seconded ? bytesToBase64(seconded) : null,
      },
    }),
  );
}

function tokenToInvitation(token) {
  const parsed = JSON.parse(atob(token.trim()));
  return {
    ...parsed,
    invitation: {
      signature: base64ToBytes(parsed.invitation.signature),
      seconded: parsed.invitation.seconded
        ? base64ToBytes(parsed.invitation.seconded)
        : null,
    },
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
let announcementFades;

/**
 * Say what just happened, where somebody will see it.
 *
 * Goes on its own after a while, because it is a thing that happened, not a
 * state anybody has to clear. Nothing counts up, nothing waits to be
 * dismissed, and an empty bar is the ordinary condition of this app.
 */
function announce(message) {
  const bar = $("announcer");
  bar.textContent = message;
  bar.hidden = !message;

  clearTimeout(announcementFades);
  if (message) {
    announcementFades = setTimeout(() => {
      bar.textContent = "";
      bar.hidden = true;
    }, 8000);
  }
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
    // Asking to be let in is part of joining now, not a screen of its own:
    // one box takes the address, and what it produces is a place in the queue.
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

/*
 * The seven sections of About Me, in her own voice and in the standard's own
 * order. The form asks whoever is typing; this is the record speaking.
 */
const FIELDS = [
  ["what_matters_to_me", "What matters most to me"],
  ["people_who_matter", "People who matter to me"],
  ["how_to_communicate_with_me", "How to talk with me"],
  ["my_wellness", "My wellness"],
  ["please_do_and_please_do_not", "Please do, and please do not"],
  ["how_to_support_me", "How and when to support me"],
  ["also_worth_knowing", "Also worth knowing about me"],
];

/**
 * The app entry inside a record.
 *
 * A record arrives with its entry still packed: `entry.Present.entry` is a
 * run of bytes, not an object. Reading a field straight off it gives
 * undefined for every field, silently — which is why a record with four
 * paragraphs in it came back as "Nothing has been written yet", and why the
 * form reopened empty over answers that were safely written down.
 */
function entryOf(record) {
  const packed = record?.entry?.Present?.entry;
  if (!packed) return null;
  try {
    return decode(packed);
  } catch {
    // A record whose entry will not unpack is not a record we can show. Say
    // nothing rather than guess at what was in it.
    return null;
  }
}

/**
 * Who wrote a record.
 *
 * Holochain 0.7 splits an action into a header and its per-variant data, and
 * the author moved into the header. Read from the old place it is undefined,
 * so every contribution was filed under nobody: the circle could not tell
 * that you had introduced yourself, and offered you the form again.
 */
const authorOf = (record) => record?.signed_action?.hashed?.content?.header?.author;

async function call(fnName, payload, cellId) {
  return client.callZome({
    ...(cellId ? { cell_id: cellId } : { role_name: ROLE }),
    zome_name: ZOME,
    fn_name: fnName,
    payload,
  });
}

/**
 * A read that is allowed to come back with nothing.
 *
 * **Reaching nobody and finding nothing look identical from here**, and this
 * app has already decided which of the two to believe: being away is not a
 * failure, and a circle whose other members are asleep is not broken.
 *
 * Every list on the circle screen is fetched over the network. Somebody who
 * has just joined has no peers yet — the whole point of `latency.md` is that
 * it takes about a minute and a half before anybody finds anybody — so those
 * reads can simply not answer. Left to throw, that put a person who had done
 * everything right on the error page, moments after joining, with a message
 * about a request timing out.
 *
 * So a read that fails is a read that found nothing yet. The screen already
 * knows how to say that, the twenty-second re-read will ask again, and
 * "Check again" is there for somebody who does not want to wait.
 *
 * Deliberately only for reads. Writing something and being told it worked
 * when it did not is a different matter entirely, and those still fail loudly.
 */
async function orNothingYet(work, nothing) {
  try {
    return await work;
  } catch (error) {
    console.warn("Nobody answered; treating as nothing yet.", error);
    return nothing;
  }
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

  // "in their own words" needs no apostrophe; "in Margaret Smythe own words"
  // does. The sentence changes shape depending on whether we know her name.
  $("suggest-whose").textContent = name?.trim() ? `${who}'s` : "their";
  $("wellness-whom").textContent = name?.trim() ? `${who}'s` : "Their";
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
    entryOf(record?.current?.record)?.display_name?.trim();
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

/*
 * What this device calls a circle.
 *
 * The name a circle is created with is fixed for the life of the cell, so two
 * circles about the same person were two identical rows in the list, and
 * "Take this off my device" was no use when you could not tell which one to
 * take off.
 *
 * Kept here rather than written to the circle because it is nobody else's
 * business: this is what you call her on your own machine. Changing it costs
 * nothing, tells nobody, and is the only thing that can tell two otherwise
 * identical rows apart.
 */
const labelKey = (cellId) => `hearth:label:${asText(cellId?.[0])}`;

function labelFor(cellId, fallback) {
  try {
    return localStorage.getItem(labelKey(cellId))?.trim() || fallback;
  } catch {
    return fallback;
  }
}

function setLabelFor(cellId, label) {
  try {
    const trimmed = label.trim();
    if (trimmed) localStorage.setItem(labelKey(cellId), trimmed);
    else localStorage.removeItem(labelKey(cellId));
  } catch {
    // Same as above: a convenience, not a rule.
  }
}

/**
 * Forget what this device remembered about a circle.
 *
 * The two things above are small, but one of them is the person's name, kept
 * in the clear so that two circles about the same person can be told apart in
 * a list. When somebody asks for a circle to be off her device, her name
 * should go off it too.
 *
 * `forgetTheCircle` already empties the screen, on the reasoning that hidden
 * is not gone. This is the same reasoning applied one step further out — to
 * the thing that survives closing the app.
 */
function forgetWhatThisDeviceKnew(cellId) {
  try {
    localStorage.removeItem(labelKey(cellId));
    localStorage.removeItem(ownRecordKey(cellId));
  } catch {
    // Nothing to do about a browser that will not let us clear its own store.
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

/*
 * What the circle screen is currently doing: reading, writing, or saying you
 * have read it.
 *
 * These three are exclusive, and they used to be worked out backwards — by
 * asking the page whether a form happened to be open, in six different places,
 * and then setting six elements from the answer. Every bug that produced was
 * the same bug: the twenty-second re-read arriving while somebody was typing
 * and putting a button back underneath them. "Write it" appearing below Save
 * and Cancel. "Read this over" above "Nothing has been written yet".
 *
 * They were fixed one at a time, and each fix guarded one element. Storing the
 * mode instead of reading it back off the screen removes the question rather
 * than answering it again.
 *
 * `showCircleMode` below is the only thing allowed to set these six elements.
 */
const READING = "reading";
const WRITING = "writing";
const SAYING_I_READ_IT = "acknowledging";

let circleMode = READING;

/**
 * Put the circle screen into a mode. The only place these six are set.
 *
 * Called both when the mode changes and at the end of every re-read, so a
 * refresh arriving mid-form redraws the mode somebody is actually in rather
 * than resetting them to the beginning.
 */
function showCircleMode(mode = circleMode) {
  circleMode = mode;

  const amHolder = isHolder();
  const written = showingSomething;

  // Writing.
  $("record-form").hidden = mode !== WRITING;

  // The things underneath it, which must not reappear while it is open.
  $("record-actions").hidden = mode === WRITING;
  $("edit-record").hidden = !amHolder;
  $("edit-record").textContent = written ? "Change this" : "Write it";

  // Saying you have read it. Offered to everybody but the holder, and only
  // once there is something to have read.
  const mayAcknowledge = !amHolder && written;
  $("acknowledge-form").hidden = mode !== SAYING_I_READ_IT;
  $("acknowledge").hidden = !mayAcknowledge || mode === SAYING_I_READ_IT;

  // Somebody who cannot acknowledge cannot be part way through acknowledging.
  if (!mayAcknowledge && mode === SAYING_I_READ_IT) {
    circleMode = READING;
    $("acknowledge-form").hidden = true;
  }
}

/** Whether the last load put a written record on the screen. */
let showingSomething = false;

function renderRecord(current) {
  const entry = entryOf(current?.record);

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
    // So a suggestion can find the words it is about, and sit next to them
    // rather than in a pile at the bottom of the page.
    dt.dataset.field = key;

    const dd = document.createElement("dd");
    dd.textContent = entry[key];
    dd.dataset.fieldValue = key;

    /*
     * Each question kept with its own answer.
     *
     * A run of headings and paragraphs down a page reads as one long thing,
     * and these are four separate answers about a person. A dl may hold a div
     * around each term-and-definition group, so the grouping is in the markup
     * rather than drawn on top of it — which also means a suggestion opened
     * against a field lands inside that field's box instead of floating below
     * the lot.
     */
    const group = document.createElement("div");
    group.className = "record-field";
    group.append(dt, dd);
    list.append(group);
  }

  // Said plainly, and only when there is somebody to name. "Supported to
  // write this by" is the standard's way of admitting that the person whose
  // record this is may not be the person who typed it.
  const supporter = entry.supported_to_write_this_by?.trim();
  const supportedBy = $("supported-by-note");
  supportedBy.hidden = !supporter;
  supportedBy.textContent = supporter ? `Supported to write this by ${supporter}.` : "";

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
    const entry = entryOf(r);
    if (!entry) continue;
    const li = document.createElement("li");
    // Never "Read by District Nurse" — that implies a credential nobody
    // checked. The claim and the claimant are shown as separate facts.
    const who = describe(authorOf(r));
    li.textContent = `${who} read this. Role claimed: ${entry.role}`;
    list.append(li);
  }
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/*
 * One reading of the circle at a time.
 *
 * Three separate things ask the screen to look again: the twenty-second timer,
 * every signal that arrives, and every action that writes something. They
 * overlap constantly — three people joining while the timer fires used to
 * start four complete re-readings of the same circle at once, each of them
 * fetching everything again.
 *
 * That is wasteful on the machine of somebody who holds the circle and
 * genuinely slow for somebody who does not, because for them every one of
 * those is a trip out to another device.
 *
 * So a request that arrives while a reading is already under way does not
 * start another one. It waits, and then gets a reading of its own that begins
 * afterwards — which matters, because most callers have just written something
 * and need to see it. Handing them the reading already in flight would show
 * them the circle as it was before they wrote.
 *
 * Everything below still calls `loadCircle()` and does not need to know.
 */
let readingNow = null;
let readingNext = null;

function loadCircle() {
  if (!readingNow) {
    readingNow = readTheCircle();
    return readingNow;
  }

  // Somebody else is already waiting for a fresh pass. Wait for the same one
  // rather than queueing a third.
  if (!readingNext) {
    readingNext = readingNow
      // Whatever went wrong with the pass in flight was reported to whoever
      // asked for it. It must not stop this one from happening.
      .catch(() => {})
      .then(() => {
        readingNext = null;
        readingNow = readTheCircle();
        return readingNow;
      });
  }

  return readingNext;
}

async function readTheCircle() {
  try {
    // Left the circle while a reading was queued. Nothing to draw.
    if (!circle) return;
    await drawTheCircle();
  } finally {
    readingNow = null;
  }
}

async function drawTheCircle() {
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

  /*
   * No special case for "but I only just wrote it".
   *
   * These two used to ask the network alone, so for a few seconds after
   * saving the answer was "nothing" — which is how the screen came to say
   * "Nothing has been written yet" directly above "Read this over", about
   * words that had just been typed into it. The zome now reads the network
   * for everybody and my own chain for me, so there is nothing left here to
   * work around.
   */
  const originals = await orNothingYet(
    call("get_circle_about_me", null, circle.cellId),
    [],
  );
  const original = originals[0] ?? null;

  const current = original
    ? await orNothingYet(
        call("get_current_about_me", original, circle.cellId),
        null,
      )
    : null;

  const entry = entryOf(current?.record);
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
  // What the screen is actually showing, for anything that needs to agree
  // with it rather than with what was typed a moment ago.
  showingSomething = written;

  record = haveIt ? { original, current } : null;
  renderRecord(haveIt ? current : null);

  $("no-record-empty").hidden = written || !amHolder;
  $("no-record-waiting").hidden = written || amHolder;

  /*
   * A refresh must not put the button back underneath the open form.
   *
   * Opening the form hides these actions; the twenty-second re-read showed
   * them again, unconditionally, so "Write it" reappeared below Save and
   * Cancel and stayed there — pressing it did nothing anybody could see,
   * because the form it opens was already open. It looked like a button with
   * no purpose, which is exactly what it had become.
   *
   * The acknowledge button was already guarded against this. The record form
   * was not, and the two need the same rule: while somebody is part-way
   * through writing, the screen underneath them holds still.
   */
  // Whatever she was already doing, drawn again from what it is rather than
  // worked out backwards from what happens to be on screen. See showCircleMode.
  showCircleMode();

  // Only offer this while there is actually something to wait for.
  $("check-again").hidden = amHolder || written;


  renderReaders(
    written
      ? await orNothingYet(
          call(
            "get_acknowledgements",
            current.record.signed_action.hashed.hash,
            circle.cellId,
          ),
          [],
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

  suggestions = await orNothingYet(
    call("get_suggestions", null, circle.cellId),
    [],
  );
  renderSuggestions();
}

function fillForm() {
  const entry = entryOf(record?.current?.record);
  // In the order they appear on the form.
  $("what-matters").value = entry?.what_matters_to_me ?? "";
  $("people-who-matter").value = entry?.people_who_matter ?? "";
  $("how-to-communicate").value = entry?.how_to_communicate_with_me ?? "";
  $("my-wellness").value = entry?.my_wellness ?? "";
  $("please-do").value = entry?.please_do_and_please_do_not ?? "";
  $("how-to-support").value = entry?.how_to_support_me ?? "";
  $("also-worth-knowing").value = entry?.also_worth_knowing ?? "";
  $("supported-by").value = entry?.supported_to_write_this_by ?? "";
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
    /*
     * Somebody writing their own record is still in it, and used to be the
     * only person who never appeared in "Who is in this circle" — which, with
     * nobody else there either, hid the whole list, and with it the "Check
     * again" button, and with that any way of ever noticing that somebody had
     * arrived.
     *
     * So she introduces herself too, under her own name and with no
     * relationship, because she is not related to herself.
     */
    const carerName = myOwn ? fullName : $("carer-name").value.trim();
    const carerRelationship = myOwn ? "" : $("carer-relationship").value.trim();

    /*
     * Checked here rather than left to the conductor.
     *
     * A second person whose identifier cannot be read closes the circle to
     * everybody, the holder included, so the failure is safe — but it arrives
     * as a genesis error, which says nothing anybody could act on. One look at
     * the shape of it turns that into a sentence about the thing they pasted.
     */
    /*
     * Nobody is named here any more, and nothing is asked for.
     *
     * The person used to be written into the circle's identity, so their
     * identifier had to be collected before the circle could exist — which put
     * the hardest step of all right at the beginning. Now the circle carries
     * only the rule, and she picks somebody from the people in it whenever she
     * has somebody to pick.
     */
    const wantsASecondYes = $("decides-two-of-us").checked;

    const cell = await whileWorking(
      $("create-circle-submit"),
      "Making the circle…",
      () =>
        call("create_circle", {
          founder: asText(me),
          name: label,
          network_seed: crypto.randomUUID(),
          /*
           * The rule, not the person. Any non-empty value sets the flag; the
           * circle names nobody, and who agrees is written inside it later
           * and can be written again.
           */
          // The rule, which is what the circle is built from. The person is
          // chosen later and is not part of its identity.
          requires_second_yes: wantsASecondYes,
        }),
    );
    circle = { cellId: cell.cell_id };
    holder = asText(me);
    markAsOwnRecord(circle.cellId, myOwn);
    $("circle-heading").textContent = label;

    /*
     * A door for it, opened at the same moment.
     *
     * One address, made once, that works for everybody and never changes. It
     * is what replaces asking each person for their identifier before they
     * can be invited — the step that defeats an elderly holder and made
     * inviting anybody a chore.
     *
     * Its own seed, so it is a different network from the circle: the room is
     * open and the circle is not, and they must never be the same place.
     *
     * If it fails, the circle is still perfectly good and she can still
     * invite by identifier. So this does not throw.
     */
    try {
      const room = {
        holder: asText(me),
        seed: crypto.randomUUID(),
        about: fullName,
      };
      await call("enter_waiting_room", {
        holder: room.holder,
        network_seed: room.seed,
        name: `${label} — door`,
      });
      rememberRoom(circle.cellId, room);
    } catch (error) {
      console.error("Could not open a waiting room for this circle.", error);
    }

    // Start the record with their name in it, so it is never nameless.
    await call(
      "create_about_me",
      {
        display_name: fullName,
        what_matters_to_me: "",
        people_who_matter: "",
        how_to_communicate_with_me: "",
        my_wellness: "",
        please_do_and_please_do_not: "",
        how_to_support_me: "",
        also_worth_knowing: "",
        // Whoever is setting this up for somebody else has just given their
        // name, and they are by definition the person supporting it being
        // written. Said here rather than asked again, and editable.
        supported_to_write_this_by: carerName,
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

    circles.push({ cellId: circle.cellId, name: label, madeWith: label });
    alwaysAWayBack();
    show("circle");
    announce(`Circle made for ${fullName}. Now write what people should know.`);
    await loadCircle();

    // Straight into writing it. An empty circle is not much use to anybody,
    // and inviting people to nothing is worse.
    nameHer(fullName);
    fillForm();
    showCircleMode(WRITING);
    $("what-matters").focus();
  } catch (error) {
    problem(error);
  }
});

$("edit-record").addEventListener("click", () => {
  fillForm();
  showCircleMode(WRITING);
  $("what-matters").focus();
});

$("cancel-edit").addEventListener("click", () => {
  showCircleMode(READING);
  $("edit-record").focus();
});

$("record-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    const aboutMe = {
      display_name: personName(),
      what_matters_to_me: $("what-matters").value,
      people_who_matter: $("people-who-matter").value,
      how_to_communicate_with_me: $("how-to-communicate").value,
      my_wellness: $("my-wellness").value,
      please_do_and_please_do_not: $("please-do").value,
      how_to_support_me: $("how-to-support").value,
      also_worth_knowing: $("also-worth-knowing").value,
      supported_to_write_this_by: $("supported-by").value.trim(),
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

    showCircleMode(READING);
    await loadCircle();

    // The holder has just written it, so show it back to them to check before
    // anybody else is asked to rely on it — but only if there is something to
    // read. Saving a form with every box empty is a real thing to do, usually
    // by accident, and "Read this over" pointing at a bare name is the screen
    // telling somebody to check work that does not exist.
    //
    // Asked of the screen, not of the form. Judging it by what was typed put
    // "Read this over" above "Nothing has been written yet" — one sentence
    // describing the form and the other describing the page, disagreeing in
    // public.
    const somethingToRead = showingSomething;

    if (isHolder() && somethingToRead) {
      $("check-it-over").hidden = false;
      announce("Saved. Read it over, then give people the address.");
    } else if (isHolder()) {
      announce(`Saved. Nothing has been written about ${aboutMe.display_name.trim() || "them"} yet.`);
    } else {
      announce("Saved.");
    }
  } catch (error) {
    problem(error);
  }
});

$("acknowledge").addEventListener("click", () => {
  // Start from what they have already told this circle they are. It is their
  // own sentence about themselves, so offering it back is not the app putting
  // words in anybody's mouth — and they can change it before it is written.
  const mine = members.get(asText(me));
  if (!$("ack-role").value.trim()) {
    $("ack-role").value = mine?.relationship?.trim() ?? "";
  }

  showCircleMode(SAYING_I_READ_IT);
  $("ack-role").focus();
  $("ack-role").select();
});

$("cancel-acknowledge").addEventListener("click", () => {
  showCircleMode(READING);
  $("acknowledge").focus();
});

$("acknowledge-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    const role = $("ack-role").value.trim();

    await whileWorking($("acknowledge-form").querySelector("button[type=submit]"), "Saying so…", () =>
      call(
        "acknowledge",
        {
          about_me: record.current.record.signed_action.hashed.hash,
          role,
        },
        circle.cellId,
      ),
    );

    showCircleMode(READING);
    announce(
      role
        ? `Marked as read, as ${role}.`
        : "Marked as read.",
    );
    await loadCircle();
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

/*
 * Look again, quietly, at the circle that is open.
 *
 * The signal that says somebody has joined is sent the moment they join,
 * which is the moment the two machines have only just found each other — so
 * it is the message most likely to be lost. Rather than make the arrival
 * depend on it, the open circle re-reads its own list every so often.
 *
 * Gossip is already running underneath this whether we look or not. Nothing
 * here syncs anything; it decides when the screen looks again. So there is no
 * spinner, no "last checked", and nothing that implies anybody has fallen
 * behind — and in particular nothing that says "up to date", which is the one
 * thing this cannot know. With no operator there is nobody to ask; finding
 * nothing and reaching nobody look identical from here.
 *
 * Only the circle actually on screen, and only while the window is being
 * looked at. Never all of them at once: a district nurse reading without
 * storing is an authority for nothing, so for her every one of these is a
 * real call over the network, and thirty of them on a timer is a different
 * proposition entirely.
 */
const LOOK_AGAIN_EVERY = 20000;

function watchForArrivals() {
  setInterval(async () => {
    if (document.hidden) return;

    /*
     * Waiting at somebody's door, with no circle to re-read yet.
     *
     * The screen says the circle will open on its own when they say yes, and
     * that was resting entirely on a signal — sent the moment she answers,
     * which is the moment two machines in a brand new empty network have only
     * just found each other. The message most likely to be lost, carrying the
     * one thing the person is waiting for.
     *
     * The comment below already says not to do this, about a different
     * arrival, and I did it anyway. So the knock is looked at again on the
     * same rhythm as everything else, and the signal goes back to being what
     * it should have been: a way to find out sooner, not the only way.
     */
    if (!circle) {
      try {
        await lookForMyAdmission();
      } catch {
        // They have not answered, or nobody is reachable. Both are ordinary
        // and neither is worth a screen.
      }
      return;
    }

    try {
      /*
       * The whole circle, not only the people in it.
       *
       * This used to look at the members and the suggestions and nothing
       * else, so a reader watching somebody's record never saw it change.
       * Editing the record sends no signal — the only thing that would have
       * told them was this, and it was not looking.
       *
       * It goes through `loadCircle`, so it costs nothing when something else
       * is already reading, and it will not land underneath somebody who is
       * halfway through typing.
       */
      await loadCircle();
    } catch {
      // Not being able to reach anybody is not an error worth a screen. It is
      // Tuesday, and somebody's laptop is shut.
    }
  }, LOOK_AGAIN_EVERY);
}

async function start() {
  /*
   * Longer than the client's own default, which is sixty seconds.
   *
   * `docs/latency.md` says joining a circle takes about ninety seconds, and it
   * is right. So the library's default and this app's slowest ordinary
   * operation were sixty and ninety, and the result was
   * "Request timed out in 60000 ms: call_zome" thrown at somebody who had done
   * nothing wrong and whose circle was, in fact, about to work.
   *
   * The delay is peer discovery — a brand new circle is an empty space and
   * nobody is looking in it yet. Until that is properly fixed, waiting is the
   * honest behaviour, and the reads that pass through `orNothingYet` mean a
   * long wait shows an empty circle rather than an error.
   */
  const LONGER_THAN_JOINING_TAKES = 180_000;

  client = await withTimeout(
    AppWebsocket.connect({ defaultTimeout: LONGER_THAN_JOINING_TAKES }),
    20,
    "Connecting",
  );

  const info = await client.appInfo();
  me = info.agent_pub_key;

  await loadCircles();

  /*
   * Anything already waiting at a door, collected before anything else.
   *
   * Somebody who knocked, closed the app, and was let in while it was shut
   * should find the circle open when they come back — not a front page with
   * no sign that anything happened.
   */
  try {
    await lookForMyAdmission();
  } catch (error) {
    console.error("Could not look for an answer at a door.", error);
  }

  watchForArrivals();

  // Someone read the record. Told to us by their device, not by a server.
  client.on("signal", async (signal) => {
    /*
     * A signal arrives as { type: "app", value: { cell_id, zome_name, payload } }.
     * Read from signal.payload it is undefined, so every handler below was
     * dead: somebody suggested something and the screen it was meant for
     * never heard. Same shape mistake as the author on an action, in a third
     * place — worth saying out loud, because it is silent every time.
     */
    const payload = signal?.value?.payload ?? signal?.payload ?? signal;

    /*
     * Only about the circle actually on screen.
     *
     * Every circle on this device is running at once. A district nurse could
     * be in thirty of them, so a signal arrives from whichever one had
     * something happen — not from the one she is looking at. Without this,
     * somebody reading Margaret's record was told "Someone read this" about a
     * different person entirely, and the page reloaded underneath her.
     *
     * The zome sends these to one person on purpose, for the same reason. This
     * is the other half of it.
     *
     * The two below are the exception, and have to be dealt with before this
     * check rather than after it: a waiting room is a different network from
     * the circle it serves, so its signals never come from the cell on screen.
     * Somebody knocking has no circle open at all.
     */
    if (payload?.kind === "Knocked") {
      const who = payload.name?.trim() || "Somebody";
      const said = payload.relationship?.trim();
      announce(
        said
          ? `${who} is asking to join. They say they are ${said}.`
          : `${who} is asking to join.`,
      );
      if (circle) await loadCircle();
      return;
    }
    if (payload?.kind === "Admitted") {
      announce("You have been let in.");
      await lookForMyAdmission();
      return;
    }

    const from = asText(signal?.value?.cell_id?.[0]);
    if (!circle || from !== asText(circle.cellId?.[0])) return;

    /*
     * Her invitation was taken up. This is the one arrival worth interrupting
     * somebody for, and it goes only to the person who sent the invitation —
     * she is the one who has been waiting to hear.
     *
     * "They say" rather than "is": the relationship is what he wrote about
     * himself, and nothing checked it. Nobody is given a pronoun they have
     * not chosen either.
     */
    if (payload?.kind === "Introduced") {
      const who = payload.name?.trim() || "Somebody";
      const said = payload.relationship?.trim();
      announce(
        payload.joined
          ? said
            ? `${who} has joined. They say: ${said}.`
            : `${who} has joined.`
          : `${who} has changed what they say about themselves.`,
      );
      if (circle) await loadCircle();
    }
    if (payload?.kind === "Acknowledged") {
      announce(`Someone read this. They said they are: ${payload.role}`);
      if (circle) await loadCircle();
    }
    if (payload?.kind === "Suggested") {
      announce("Someone has suggested something for the record.");
      if (circle) await loadCircle();
    }
    /*
     * The two halves of an admission, arriving without anybody carrying them.
     *
     * Both are nudges rather than the thing itself: the proposal and the
     * agreement are written in the circle either way, and the list shows them
     * whenever anybody next looks. A signal that goes missing costs a wait,
     * never a decision.
     */
    if (payload?.kind === "Proposed") {
      const who = payload.name?.trim();
      announce(
        who
          ? `You are asked to agree to letting ${who} in.`
          : "You are asked to agree to letting somebody in.",
      );
      if (circle) await loadCircle();
    }
    if (payload?.kind === "Endorsed") {
      announce("Agreed. They are being let in.");
      if (circle) await loadCircle();
    }

    /*
     * You have been asked to agree to who joins.
     *
     * Worth interrupting somebody for: it is a question addressed to them
     * personally, and until it is answered nobody can join at all. The panel
     * it puts on screen stays there until they answer, because a sentence
     * that scrolls away is a sentence somebody did not read.
     */
    if (payload?.kind === "Appointed") {
      const who = members.get(asText(payload.by))?.name?.trim();
      announce(
        who
          ? `${who} has asked you to agree to who joins this circle.`
          : "You have been asked to agree to who joins this circle.",
      );
      if (circle) {
        await loadCircle();
        // The appointment itself is still on its way. See watchForTheAsking.
        if (!whoAgrees) watchForTheAsking();
      }
    }

    // And the answer, to the person who asked. A no she does not hear is the
    // same to her as no answer at all.
    if (payload?.kind === "Answered") {
      const who = members.get(asText(payload.by))?.name?.trim() || "They";
      announce(
        payload.willing
          ? `${who} agreed. Nobody new can join unless you both say yes.`
          : `${who} would rather not. Ask somebody else by pressing their name.`,
      );
      if (circle) await loadCircle();
    }
  });
}

start().catch(problem);

// ---------------------------------------------------------------------------
// Suggestions
// ---------------------------------------------------------------------------

const FIELD_LABELS = {
  WhatMattersToMe: ["what_matters_to_me", "What matters most to me"],
  PeopleWhoMatter: ["people_who_matter", "People who matter to me"],
  HowToCommunicateWithMe: ["how_to_communicate_with_me", "How to talk with me"],
  MyWellness: ["my_wellness", "My wellness"],
  PleaseDoAndPleaseDoNot: [
    "please_do_and_please_do_not",
    "Please do, and please do not",
  ],
  HowToSupportMe: ["how_to_support_me", "How and when to support me"],
  AlsoWorthKnowing: ["also_worth_knowing", "Also worth knowing about me"],
};

let suggestions = [];

function renderSuggestions() {
  const list = $("suggestions-list");
  list.replaceChildren();

  for (const item of suggestions) {
    const card = suggestionCard(item);
    if (card) list.append(card);
  }

  /*
   * The list at the bottom is now the fallback, not the main event.
   *
   * Everything in it also appears beside the words it is about, which is
   * where somebody would look for it. It is still here for the case where
   * there is no record on screen to attach anything to — nothing written yet,
   * or not arrived on this device.
   */
  markSuggestionsOnTheRecord();
  $("suggestions-section").hidden =
    suggestions.length === 0 || !$("record").hidden;
}

/*
 * Put what has been offered next to the words it is about.
 *
 * A suggestion is always about one part of the record, and reading it at the
 * bottom of the page means holding the field in your head while you scroll.
 * Beside the heading it is obvious what is being proposed and what it would
 * replace.
 *
 * Folded away rather than shown open: the record is the thing on this page,
 * and somebody else's proposal should not push it down the screen until she
 * asks to see it.
 */
function markSuggestionsOnTheRecord() {
  const list = $("record-fields");
  if (!list) return;

  for (const stale of list.querySelectorAll(".suggestion-marker, .field-suggestions")) {
    stale.remove();
  }
  if ($("record").hidden) return;

  const byField = new Map();
  for (const item of suggestions) {
    const entry = entryOf(item.suggestion);
    if (!entry) continue;
    const [key] = FIELD_LABELS[entry.field] ?? [];
    if (!key) continue;
    if (!byField.has(key)) byField.set(key, []);
    byField.get(key).push(item);
  }

  for (const [key, items] of byField) {
    const dt = list.querySelector(`dt[data-field="${key}"]`);
    const dd = list.querySelector(`dd[data-field-value="${key}"]`);
    if (!dt || !dd) continue;

    // Only the ones nobody has decided about are worth flagging. A suggestion
    // already added or set aside is still readable, it just does not ask for
    // anything.
    const waiting = items.filter((item) => !item.outcome).length;

    const marker = document.createElement("button");
    marker.type = "button";
    marker.className = "suggestion-marker";
    marker.setAttribute("aria-expanded", "false");
    marker.textContent = waiting
      ? waiting === 1
        ? "1 suggestion"
        : `${waiting} suggestions`
      : "Suggested before";
    dt.append(" ", marker);

    const panel = document.createElement("div");
    panel.className = "field-suggestions";
    panel.hidden = true;
    for (const item of items) {
      const card = suggestionCard(item);
      if (card) panel.append(card);
    }
    dd.after(panel);

    marker.addEventListener("click", () => {
      panel.hidden = !panel.hidden;
      marker.setAttribute("aria-expanded", String(!panel.hidden));
    });
  }
}

/** One suggestion, as it is shown wherever it is shown. */
function suggestionCard(item) {
  const amHolder = isHolder();

  const entry = entryOf(item.suggestion);
  if (!entry) return null;

  const author = authorOf(item.suggestion);
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

  const outcome = entryOf(item.outcome);
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

    /*
     * Held while the writing happens, like every other button that writes.
     *
     * Without this, two quick presses wrote the decision twice and appended
     * the same sentence to the record twice — because nothing on the card
     * changes until the reload at the end. It is the same fault as the three
     * circles called "Mam": the press was never the mistake, a button that
     * stays silent while it works is.
     */
    const accept = document.createElement("button");
    accept.type = "button";
    accept.textContent = "Add this";

    const setAside = document.createElement("button");
    setAside.type = "button";
    setAside.className = "secondary";
    setAside.textContent = "Not this one";

    // Both of them, not just the one pressed. They are two answers to one
    // question, and pressing the second while the first is still being written
    // would decide the same suggestion twice.
    const once = (button, working, accepted) =>
      button.addEventListener("click", () => {
        const other = button === accept ? setAside : accept;
        other.disabled = true;
        whileWorking(button, working, () => decide(item, entry, accepted))
          .catch(problem)
          .finally(() => {
            other.disabled = false;
          });
      });

    once(accept, "Adding…", true);
    once(setAside, "Setting aside…", false);

    actions.append(accept, setAside);
    li.append(actions);
  } else {
    const waiting = document.createElement("p");
    waiting.className = "who";
    waiting.textContent = "Not looked at yet.";
    li.append(waiting);
  }

  return li;
}

/**
 * Accept or set aside. Accepting also puts the words into the record.
 *
 * The record is written FIRST, and the decision only afterwards.
 *
 * The other way round is what this used to do, and it could tell somebody a
 * plain untruth: if adding the words failed, the decision was already saved as
 * accepted, so the card read "Added to the record." about something that was
 * not in it. Worse, if the record had not loaded, the words were skipped in
 * complete silence and the card still said they had gone in.
 *
 * This order can only fail the safe way. Losing the decision means she is asked
 * to decide again, which is a small annoyance. Losing somebody's words while
 * telling her they were kept is not.
 */
async function decide(item, entry, accepted) {
  const hash = item.suggestion.signed_action.hashed.hash;

  if (accepted) {
    // Not a state anybody should be able to reach — the buttons only exist on
    // a circle that is showing a record — but saying so is better than
    // silently dropping what somebody took the trouble to notice.
    if (!record) {
      throw new Error(
        "The record has not finished loading, so there is nowhere to put this " +
          "yet. Try again in a moment.",
      );
    }

    const [key] = FIELD_LABELS[entry.field] ?? [];
    const current = entryOf(record.current.record);
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

  await call("decide_on_suggestion", { suggestion: hash, accepted }, circle.cellId);

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

/**
 * Everybody in the circle, as they describe themselves.
 *
 * Deliberately not "2 members" and not a count anywhere. A district nurse
 * could be in thirty of these; a number to compare against is the beginning
 * of a queue. Names, and what each person says they are.
 *
 * Hidden while nobody has said who they are, because an empty heading answers
 * nothing. It is not hidden for a circle of one — seeing only yourself listed
 * is the honest answer to "has anybody joined yet".
 */
/*
 * Always a way out of a circle, even the only one there is.
 *
 * This used to appear only once somebody had two, on the reasoning that a
 * button back to a list of one is noise. It is not: it left a person at the
 * bottom of a page of invitation text with nowhere to go, and no way to make
 * the page look again for anything that had arrived since.
 */
function alwaysAWayBack() {
  $("back-to-circles").hidden = false;
}

/*
 * Who was here last time we looked.
 *
 * An arrival is announced from the list itself rather than from the signal
 * that goes with it. The signal is sent the instant somebody joins — the one
 * moment the two machines have only just found each other — so it is exactly
 * the message most likely to go missing. Noticing the change works whether it
 * arrived or not, which makes the signal a convenience rather than the thing
 * everything depends on.
 */
let peopleLastSeen = new Set();

function sayWhoIsNew() {
  const now = new Set(members.keys());
  const firstLook = peopleLastSeen.size === 0;

  const arrived = [...now].filter(
    (key) => !peopleLastSeen.has(key) && key !== asText(me),
  );
  peopleLastSeen = now;

  // Opening a circle is not everybody arriving at once.
  if (firstLook || arrived.length === 0) return;

  const names = arrived.map((key) => members.get(key)?.name?.trim() || "Somebody");
  announce(
    names.length === 1
      ? `${names[0]} has joined.`
      : `${names.join(" and ")} have joined.`,
  );
}

/*
 * Who this circle asks to agree to who joins, and how far that has got.
 *
 * Three states, not two: nobody asked, asked and not yet answered, answered.
 * "Has not got round to it" and "said no" are the same silence from outside,
 * and telling them apart is the whole reason the answer is written down.
 */
let whoAgrees = null;

async function readWhoAgrees() {
  whoAgrees = await call("who_agrees_here", null, circle.cellId);
  seconderHere = whoAgrees?.agrees ?? null;

  // The list was drawn before this read came back, from the last pass. It says
  // who has been asked and how far it has got, so it is drawn again now that
  // those are known rather than showing the previous circle’s answer.
  renderPeople();

  await loadPending();
  await loadTheDoor();
  // Anybody now agreed to has their invitation left at the door. She decided
  // when she pressed "Let them in"; this is the consequence of that and the
  // second agreement, and doing it here is what removes the errand of coming
  // back to press something again.
  await deliverAnythingAgreed();

  askTheQuestionIfItIsMine();
}

/*
 * The question, put to the person it is about.
 *
 * Shown until it is answered with a yes, which includes after a no: somebody
 * who declined can change their mind, and the alternative is a screen with no
 * way back to a decision they have already made.
 */
function askTheQuestionIfItIsMine() {
  const panel = $("asked-to-agree");
  const mine = whoAgrees && whoAgrees.agrees === asText(me);

  panel.hidden = !mine || whoAgrees.willing === true;
  if (panel.hidden) return;

  // The holder, by name where she has given one. She made the circle, so she
  // usually has, but nothing here depends on it.
  $("asked-by").textContent =
    members.get(holder)?.name?.trim() || "The person who holds this circle";
  $("asked-already-said-no").hidden = whoAgrees.willing !== false;
}

/*
 * The signal outruns the appointment, and the screen used to wait for gossip.
 *
 * Found by walking it, and confirmed by asking both conductors rather than
 * guessing: the person asked was told at once — the signal goes straight to
 * them — and then had nothing to press. An appointment is an entry the holder
 * wrote, so it reaches them by gossip, which is seconds or tens of seconds
 * behind. Until it arrived, `who_agrees_here` answered "nobody", and the only
 * thing that ever looked again was the twenty-second re-read.
 *
 * So: a notification saying you have been asked, above a screen with no way
 * to answer. The same shape as the holder who let somebody in and was told by
 * her own screen that she had not.
 *
 * This looks again, briefly and often, and stops the moment there is
 * something to show. Deliberately not a guess from the signal's own payload:
 * the button that panel offers writes an entry whose validation must read the
 * appointment, so offering it before the appointment can be read would trade
 * a missing button for a button that fails.
 */
let watchingForTheAsking = null;

function watchForTheAsking() {
  if (watchingForTheAsking) return;

  const inThisCircle = asText(circle?.cellId?.[0]);
  let looks = 0;

  watchingForTheAsking = setInterval(async () => {
    looks += 1;
    const elsewhere = asText(circle?.cellId?.[0]) !== inThisCircle;

    // Twenty looks is forty seconds, after which the ordinary re-read is as
    // good as this and there is no reason to keep asking.
    if (whoAgrees || elsewhere || looks > 20) {
      clearInterval(watchingForTheAsking);
      watchingForTheAsking = null;
      return;
    }

    // A failed look is not a failure. The next one tries again, and the
    // twenty-second re-read is still there underneath.
    try {
      await readWhoAgrees();
    } catch (error) {
      console.error(error);
    }
  }, 2000);
}

async function answerTheAsking(willing) {
  const button = willing ? $("agree-to-agree") : $("decline-to-agree");
  await whileWorking(button, willing ? "Agreeing…" : "Answering…", () =>
    call(
      "answer_appointment",
      { appointment: whoAgrees.appointment, willing },
      circle.cellId,
    ),
  );
  announce(
    willing
      ? "Agreed. Nobody new can join unless you say so too."
      : "Answered. They will see that you would rather not, and can ask somebody else.",
  );
  await loadCircle();
}

$("agree-to-agree").addEventListener("click", () =>
  answerTheAsking(true).catch(problem),
);
$("decline-to-agree").addEventListener("click", () =>
  answerTheAsking(false).catch(problem),
);

function renderPeople() {
  const list = $("people-list");
  list.replaceChildren();

  /*
   * Shown even when it is empty, which it should not be any more but can be
   * for a circle made before people introduced themselves on the way in.
   *
   * Hiding it took "Check again" with it, so a circle with nobody in it had no
   * way to look for anybody — the one screen where looking again is the whole
   * point. An empty list with an honest line under it is better than no list.
   */
  $("people").hidden = false;
  $("people-empty").hidden = members.size > 0;

  // Only the holder appoints, and only where there is somebody to appoint.
  const amHolder = isHolder();
  $("appoint-hint").hidden = !amHolder || members.size < 2;

  for (const [key, entry] of members) {
    const li = document.createElement("li");
    const who = entry.name?.trim() || "Somebody";
    const said = entry.relationship?.trim();

    const line = document.createElement("p");
    // Never "Dave Smythe, Nephew" as though the circle had checked. The
    // relationship is what he said about himself, and the sentence says so.
    line.textContent = said ? `${who} — ${said}` : who;
    if (key === asText(me)) line.textContent += " (you)";
    li.append(line);

    const theirs = whoAgrees?.agrees === key;
    if (theirs) li.append(howFarTheAskingHasGot(who));

    // Asking somebody else replaces whoever is asked now, so the person
    // already asked needs no button of their own — pressing another name is
    // the whole of changing your mind.
    if (amHolder && key !== asText(me) && !theirs) {
      li.append(askThem(key, who));
    }

    list.append(li);
  }
}

/**
 * What the person asked has said, in a sentence rather than a state.
 *
 * "Asked" and "agreed" are different facts and the holder acts differently on
 * each, so neither is allowed to stand in for the other.
 */
function howFarTheAskingHasGot(who) {
  const said = document.createElement("p");
  said.className = whoAgrees.willing === false ? "notice" : "hint";
  said.textContent =
    whoAgrees.willing === true
      ? "Agrees to who joins, along with you."
      : whoAgrees.willing === false
        ? "Asked, and would rather not. Nobody new can join until somebody " +
          "else is asked."
        : `Asked. ${who} has not answered yet, and nobody new can join until ` +
          `they do.`;
  return said;
}

/**
 * The act itself: press a name, and they are asked.
 *
 * It writes one line in the circle. It used to mean building a whole new
 * circle and everybody joining it again, because the person was part of the
 * circle's identity and identity cannot be edited. The circle now carries the
 * rule and an entry carries the person, so changing who it is costs nothing
 * and nobody is inconvenienced — which matters on the day it is needed, when
 * somebody has died or lost the device their keys were on.
 *
 * Nothing is erased by it. Who was asked, and when, stays in the circle where
 * everybody can see it, and that visibility is what the safeguard now rests
 * on.
 */
function askThem(key, who) {
  const actions = document.createElement("div");
  actions.className = "actions";

  const button = document.createElement("button");
  button.type = "button";
  button.className = "linky";
  button.textContent = whoAgrees
    ? `Ask ${who} instead`
    : `Ask ${who} to agree to who joins`;

  button.addEventListener("click", () =>
    whileWorking(button, "Asking…", async () => {
      await call("appoint", key, circle.cellId);
      announce(`${who} has been asked. Nobody new can join until they agree.`);
      await loadCircle();
    }).catch(problem),
  );

  actions.append(button);
  return actions;
}

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
  const records = await orNothingYet(
    call("get_members", null, circle.cellId),
    [],
  );
  members = new Map();
  for (const r of records) {
    const entry = entryOf(r);
    if (!entry) continue;
    // Latest introduction wins; people correct how they describe themselves.
    members.set(asText(authorOf(r)), entry);
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
   * The list above includes me from the moment I introduce myself, because
   * the zome reads my own chain alongside the network. It did not always, and
   * this section reappeared empty on the screen I had just filled in.
   */
  sayWhoIsNew();
  renderPeople();
  // Awaited. It makes a call of its own and writes to the screen, so left
  // unawaited a failure became an unhandled rejection with nothing shown, and
  // it could finish drawing after the screen had already moved somewhere else.
  await readWhoAgrees();

  const mine = members.get(asText(me));
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
        "That is somebody's identifier, not a way in. An identifier only says " +
          "who a person is. Ask whoever holds the circle to send you its " +
          "address, and paste that here instead.",
      );
    }

    /*
     * An address, so knock and wait.
     *
     * This is the ordinary way in now. It used to be a separate screen with a
     * separate box, which meant somebody had to know which of two long lines
     * of characters they had been sent before they could begin — and pasting
     * one into the other's box produced a raw error about a missing
     * signature.
     *
     * The app can tell them apart. There is no reason a person should have
     * to.
     */
    if (looksLikeARoomAddress(pasted)) {
      const room = addressToRoom(pasted);
      const name = $("joiner-name").value.trim();
      const relationship = $("joiner-relationship").value.trim();

      await whileWorking($("join-circle-submit"), "Asking…", async () => {
        myRoomCell = await cellForRoom(room, room.about || "A circle");
        await call("knock", { name, relationship }, myRoomCell);
      });

      showJoining(true);
      announce("Asked. They will see your name when they next open Hearth.");
      await lookForMyAdmission();
      return;
    }

    if (!looksLikeAnInvitation(pasted)) {
      throw new Error(
        "That is not something this app can read. It should be one long " +
          "line, pasted whole — it is easy to catch only part of it.",
      );
    }

    const bundle = tokenToInvitation(pasted);
    const label = $("join-label").value.trim() || bundle.about || "Circle";

    /*
     * Have I been here before?
     *
     * Taking a circle off this device disables the clone rather than deleting
     * it, so the cell is still there and its id is still taken. Joining again
     * therefore tried to build something that already existed and failed with
     * "Tried to create a cell with an existing id" — a wasm error in front of
     * somebody whose only mistake was changing their mind.
     *
     * Matched on what the invitation carries rather than on a hash we would
     * have to compute: a circle is its founder, its seed and whoever it asks
     * to agree, and those three are exactly what the DNA is built from.
     */
    const known = await circleAlreadyHere(bundle);

    const cell = await whileWorking(
      $("join-circle-submit"),
      known ? "Coming back…" : "Joining…",
      () =>
        known
          ? // Intact, with everything that was in it. It was only switched off.
            call("rejoin_circle", known.cellId[0])
          : call("join_circle", {
              founder: bundle.founder,
              name: label,
              network_seed: bundle.network_seed,
              invitation: bundle.invitation,
              // Out of the invitation, because it forms part of the DNA hash:
              // get this wrong and you compute a different circle and arrive
              // nowhere.
              requires_second_yes: Boolean(bundle.requires_second_yes),
              // Not part of the hash. Only so a plainly wrong invitation is
              // refused before a cell is built from it.
              seconder: bundle.seconder ?? null,
            }),
    );

    circle = { cellId: cell.cell_id };
    holder = bundle.founder; // already text, out of the invitation
    // What she typed on the way in is what this device should call it, and it
    // has to be written down or it lasts only until the app is closed. The
    // name the cell was made with belongs to whoever made it, not to her.
    setLabelFor(cell.cell_id, label);
    $("circle-heading").textContent = label;
    circles.push({ cellId: circle.cellId, name: label, madeWith: label });

    /*
     * Say who you are in the same breath as arriving.
     *
     * These were two separate acts, and only the second one told anybody: a
     * nephew joined, stopped there, and the person who had invited him saw
     * nothing at all, because nothing on the chain had his name on it. It is
     * also what tells her the invitation was taken up — the signal goes out
     * from here.
     */
    // Not when coming back to one you left. You introduced yourself the first
    // time, and saying it again with whatever happens to be in the form would
    // overwrite what you said with blanks.
    if (!known) {
      await call(
        "introduce_myself",
        {
          name: $("joiner-name").value.trim(),
          relationship: $("joiner-relationship").value.trim(),
        },
        circle.cellId,
      );
    }

    alwaysAWayBack();
    show("circle");
    announce(
      known
        ? `${bundle.about || "The circle"} is back on this device.`
        : `You have joined ${bundle.about || "the circle"}.`,
    );
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


// ---------------------------------------------------------------------------
// The list of people
// ---------------------------------------------------------------------------

/** Every circle this person belongs to. Circles are clones of the lobby. */
/**
 * A circle already on this device that this invitation would rebuild.
 *
 * Returns the cell whether it is switched on or off: coming back to one you
 * left and being invited again to one you are already in are the same thing
 * from here, and neither should try to create anything.
 */
async function circleAlreadyHere(bundle) {
  const info = await client.appInfo();

  for (const raw of info.cell_info[ROLE] ?? []) {
    const c = raw?.value ?? raw?.cloned ?? raw;
    if (!c?.clone_id) continue;

    const seed = c.dna_modifiers?.network_seed;
    if (seed !== bundle.network_seed) continue;

    // The founder is in the properties, which arrive as packed bytes.
    let props = c.dna_modifiers?.properties;
    if (props instanceof Uint8Array) {
      try {
        props = decode(props);
      } catch {
        continue;
      }
    }
    if (props?.founder !== bundle.founder) continue;

    return { cellId: c.cell_id, enabled: c.enabled !== false, name: c.name };
  }

  return null;
}

async function loadCircles() {
  const info = await client.appInfo();
  const cells = info.cell_info[ROLE] ?? [];

  circles = cells
    .map((c) => c?.value ?? c?.cloned ?? c)
    .filter((c) => c?.clone_id || c?.original_dna_hash)
    // A waiting room is a cell on this device but it is not a circle: it
    // holds no record and nobody is in it. Listing it beside real circles
    // would put a door in the phone book.
    .filter((c) => !propertiesOf(c)?.waiting_for)
    // A circle taken off this device is disabled, not deleted, so the
    // conductor still lists it. It should not be on her screen.
    .filter((c) => c.enabled !== false)
    // What this device calls it wins over the name the cell was made with,
    // which cannot be changed afterwards.
    .map((c) => ({
      cellId: c.cell_id,
      name: labelFor(c.cell_id, c.name || "Circle"),
      // Kept so clearing the label can go back to it. The label overrides
      // this; it does not replace it.
      madeWith: c.name || "Circle",
    }));

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

  // A different circle has different people in it, and none of them is "new".
  peopleLastSeen = new Set();

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
  alwaysAWayBack();
  show("circle");
  await loadCircle();
}

/*
 * Put down everything we know about the circle we were in.
 *
 * Leaving used to set `circle` and `holder` to null and stop there, which
 * left the record, the members, the suggestions and every rendered element on
 * the screen exactly where they were. The screen was hidden rather than
 * emptied, so what she had just taken off her device was still sitting in it,
 * waiting to be shown again the moment anything else was opened.
 *
 * For a page about somebody's private record, "hidden" is not "gone". If she
 * has asked for a circle to be off this device, it should not be in the
 * running program either.
 */
function forgetTheCircle() {
  circle = null;
  holder = null;
  record = null;
  members = new Map();
  // Who this circle asked, and who it asked to agree. Carried into the next
  // circle these would be somebody else's answers on somebody else's screen.
  whoAgrees = null;
  seconderHere = null;
  if (watchingForTheAsking) {
    clearInterval(watchingForTheAsking);
    watchingForTheAsking = null;
  }
  suggestions = [];
  peopleLastSeen = new Set();
  showingSomething = false;
  knownName = "";

  // And the screen, which is the part she can actually see.
  $("record-name").textContent = "";
  $("record-fields").replaceChildren();
  $("people-list").replaceChildren();
  $("suggestions-list").replaceChildren();
  $("circle-heading").textContent = "";
  $("record").hidden = true;
  circleMode = READING;
  $("record-form").hidden = true;
  $("acknowledge-form").hidden = true;
  $("asked-to-agree").hidden = true;
  $("appoint-hint").hidden = true;
}

$("leave-circle").addEventListener("click", async () => {
  try {
    const leaving = $("circle-heading").textContent;
    // Held before anything is switched off, because forgetTheCircle drops it.
    const leavingCell = circle.cellId;

    await whileWorking($("leave-circle"), "Taking it off…", () =>
      // Sent to the lobby cell, not to the circle: a cell cannot be the one to
      // switch itself off.
      call("leave_circle", circle.cellId[0]),
    );

    // Her name and what this device called this circle, gone from the store as
    // well as from the screen.
    forgetWhatThisDeviceKnew(leavingCell);
  // And the door to it, which is hers alone and means nothing without it.
  forgetRoom(leavingCell);
    forgetTheCircle();
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

/*
 * Done inviting: back up to the record, not out of the circle.
 *
 * Somebody who has just sent an invitation has finished a job, and the thing
 * they want next is the page they were on before it — not a list of people
 * and not a wall of base64 with nowhere to go.
 */
/*
 * Appoint a second yes by re-forming the circle around them.
 *
 * Everything here already exists: a circle can be made with a seconder, and
 * the holder may author the record into a circle she founds. So this is the
 * two of them in order, with the words carried across — no new machinery, and
 * nothing that could not be done by hand.
 *
 * The old circle is left alone rather than taken away. People are still in it,
 * and pulling it out from under them because she has made a new one would be
 * the software deciding something on their behalf. She removes it herself when
 * everybody has moved.
 */
$("check-people").addEventListener("click", async () => {
  try {
    await whileWorking($("check-people"), "Looking…", () => loadMembers());
    // Says something either way. A button that sometimes does nothing visible
    // is a button people press over and over.
    if (!$("announcer").textContent) announce("Nobody new yet.");
  } catch (error) {
    problem(error);
  }
});

$("rename-circle").addEventListener("click", () => {
  $("circle-label").value = $("circle-heading").textContent;
  $("rename-form").hidden = false;
  $("rename-circle").hidden = true;
  $("circle-label").focus();
  $("circle-label").select();
});

$("cancel-rename").addEventListener("click", () => {
  $("rename-form").hidden = true;
  $("rename-circle").hidden = false;
  $("rename-circle").focus();
});

$("rename-form").addEventListener("submit", (event) => {
  event.preventDefault();

  // Blank goes back to the name the circle was made with, rather than leaving
  // somebody with an unnamed row they cannot fix.
  const inList = circles.find(
    (c) => asText(c.cellId?.[0]) === asText(circle.cellId?.[0]),
  );
  setLabelFor(circle.cellId, $("circle-label").value);

  const now = labelFor(circle.cellId, inList?.madeWith || "Circle");
  $("circle-heading").textContent = now;
  if (inList) inList.name = now;

  $("rename-form").hidden = true;
  $("rename-circle").hidden = false;
  $("rename-circle").focus();
  announce(`Called ${now} on this device.`);
});

/*
 * Every form has a way out. Getting somewhere by accident should cost one
 * press to undo, not a restart.
 *
 * This was here, and was deleted by accident: it sat between two lines that
 * were being cut out together, and went with them. Nothing caught it. It has
 * no id, so the check for "a name in the code with no element on the page"
 * could not see it, and neither could the one for functions nobody calls —
 * the buttons were still on the page, still looked like buttons, and did
 * nothing at all when pressed.
 *
 * Which left somebody on the create screen with the wrong option chosen and
 * no way back to the two buttons they started from.
 */
for (const button of document.querySelectorAll(".back-to-choose")) {
  button.addEventListener("click", () => show("choose"));
}

$("back-to-circles").addEventListener("click", () => {
  renderCircles();
  show("circles");
});

$("add-circle").addEventListener("click", () => {
  show("choose");
});

wireCopyButton(
  "copy-door-address",
  () => $("door-address-output").textContent,
  "Address copied",
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

/*
 * The invitation carries her name, so use it the moment it is pasted.
 *
 * Two questions on this form are about her — how you are connected to her,
 * and what you call her — and "them" stops meaning anything once there is a
 * name available. Quietly ignores anything that will not parse: this fires on
 * every keystroke of a long paste, and half an invitation is not an error,
 * just an unfinished one.
 */
$("invitation-in").addEventListener("input", () => {
  let about = "";
  try {
    about = tokenToInvitation($("invitation-in").value)?.about?.trim() ?? "";
  } catch {
    about = "";
  }
  $("join-relationship-whom").textContent = about || "them";
  $("join-label-whom").textContent = about || "the person this circle is about";
});

$("go-back").addEventListener("click", () => {
  // Back to where they were, with whatever they typed still in the fields.
  // A mistyped character should cost a correction, not a restart.
  show(lastGoodScreen);
});

// ---------------------------------------------------------------------------
// Choosing what to do
// ---------------------------------------------------------------------------

/*
 * The create screen, with or without a second person.
 *
 * One screen and one submit handler for both, because they differ by a single
 * field. The alternative — a second form — would be two places to fix every
 * time the questions change, and they have changed a lot.
 */
function goToCreate() {
  // Always shown. Whether a circle asks two people to agree is one question
  // about the circle being made, not a different kind of circle reached by a
  // different button.
  $("decides-just-me").checked = true;
  show("create");
  $("person-name").focus();
}

$("choose-create").addEventListener("click", goToCreate);

/*
 * A second join starts empty, and does not know her name yet.
 *
 * Coming here from the front page used to show whatever was left from the
 * last time: a spent invitation still in the box, a name, a relationship, and
 * — worst of it — the question "How are you connected to Margaret Smythe?"
 * asked before anything had been pasted. The app appeared to know who the
 * circle was about, when in truth it was quoting the previous one back.
 *
 * That is the same mistake as leaving a circle without putting it down, and
 * it is worse here: it invites somebody to answer a question about the wrong
 * person, and their answer is written into the record as how they describe
 * themselves.
 *
 * Note this is deliberately not done in `go-back`, which returns to a screen
 * somebody was already filling in. A mistyped character should cost a
 * correction, not a restart. Arriving fresh is a different thing from coming
 * back.
 */
function showJoining(asked) {
  $("join-form").hidden = asked;
  $("join-steps").hidden = asked;
  $("nothing-to-paste").hidden = asked;
  $("knocked").hidden = !asked;
}

function aFreshJoin() {
  $("invitation-in").value = "";
  $("joiner-name").value = "";
  $("joiner-relationship").value = "";
  showJoining(false);
  // Back to "them", until an invitation actually says otherwise.
  knownName = "";
  nameHer("");
}

$("choose-join").addEventListener("click", () => {
  aFreshJoin();
  show("join");
  $("invitation-in").focus();
});

/*
 * Agreeing to who joins, from the front page and from inside a circle.
 *
 * Both go to the same screen. Nothing here needs a circle to be open — the
 * signature is this key over that person's key — so somebody who guards a
 * circle they are not in gets there from the front page, and somebody who is
 * both a member and the second yes gets there from where they were standing.
 */
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

// ---------------------------------------------------------------------------
// People put forward, waiting on the second agreement
// ---------------------------------------------------------------------------
//
// One list, read from both ends. The holder sees how far each one has got and,
// once both agreements exist, the invitation to send. The person who has to
// agree sees a name and a button.
//
// This is what replaced the posting back and forth. Two devices already in the
// same circle can simply tell each other, and the only message that still has
// to leave is the last one, to the person joining, who is outside by
// definition.

let pending = [];

/** Who this circle asks to agree, as text, or null. Set on every re-read. */
let seconderHere = null;

async function loadPending() {
  const section = $("pending-members");
  const amSeconder = Boolean(seconderHere) && seconderHere === asText(me);

  // Nobody else has any business with this list: it names people who are not
  // in the circle yet, and what the holder calls them.
  if (!seconderHere || !(isHolder() || amSeconder)) {
    section.hidden = true;
    pending = [];
    $("pending-list").replaceChildren();
    return;
  }

  pending = await orNothingYet(
    call("get_pending_members", null, circle.cellId),
    [],
  );

  renderPending(amSeconder);
}

function renderPending(amSeconder) {
  const section = $("pending-members");
  const list = $("pending-list");
  list.replaceChildren();

  // An empty list is not worth a heading. Nobody is waiting, which is the
  // ordinary state of a circle.
  section.hidden = pending.length === 0;
  $("pending-explains-holder").hidden = amSeconder || pending.length === 0;
  $("pending-explains-seconder").hidden = !amSeconder || pending.length === 0;

  if (!amSeconder) {
    $("pending-seconder-name").textContent =
      members.get(seconderHere)?.name?.trim() || "The second person";
  }

  for (const item of pending) {
    list.append(pendingCard(item, amSeconder));
  }
}

function pendingCard(item, amSeconder) {
  const li = document.createElement("li");
  li.className = "suggestion";

  const who = document.createElement("p");
  // Never dressed up as established. It is what the holder calls them.
  who.textContent = item.name?.trim() || "Somebody with no name given";
  li.append(who);

  const key = document.createElement("p");
  key.className = "hint";
  key.textContent = item.invitee;
  li.append(key);

  if (amSeconder && !item.agreed) {
    const check = document.createElement("p");
    check.className = "hint";
    check.textContent =
      "Nobody has checked that this is who they say it is. If you were " +
      "expecting this, compare the identifier with the one you were told.";
    li.append(check);

    const agree = document.createElement("button");
    agree.type = "button";
    agree.textContent = "I agree to this";
    agree.addEventListener("click", () =>
      whileWorking(agree, "Agreeing…", async () => {
        await call("endorse", item.proposed, circle.cellId);
        announce("Agreed. They can be let in now.");
        await loadCircle();
      }).catch(problem),
    );

    const actions = document.createElement("div");
    actions.className = "actions";
    actions.append(agree);
    li.append(actions);
    return li;
  }

  if (!item.agreed) {
    const waiting = document.createElement("p");
    waiting.className = "who";
    // No spinner and no elapsed time. Somebody has not got to it yet, which is
    // not a fault and not something to be anxious about.
    waiting.textContent = "Not agreed to yet.";
    li.append(waiting);
    return li;
  }

  const agreed = document.createElement("p");
  agreed.className = "outcome";
  agreed.textContent = amSeconder
    ? "You agreed to this."
    : "Agreed. They are being let in — nothing for you to send.";
  li.append(agreed);

  return li;
}

// ---------------------------------------------------------------------------
// The waiting room
// ---------------------------------------------------------------------------
//
// Joining used to begin with "send me the long line of characters from your
// app". A waiting room turns that round: the holder shares one address that
// never changes and works for everybody, and whoever has it knocks. They bring
// their own key by arriving.
//
// The room is a separate network. A circle is closed, so somebody outside
// cannot write to it — that is the membrane working, not a gap. The room is
// somewhere they can write, and it holds nothing but questions and answers.

/**
 * A room's address, as one line of text.
 *
 * Deliberately not shaped like an invitation. It is public, it is the same for
 * everybody, and it lets somebody ask rather than enter — so it should not
 * look like the thing that does let people in.
 */
function roomToAddress(room) {
  return btoa(JSON.stringify({ door: room.holder, seed: room.seed, about: room.about }));
}

function addressToRoom(text) {
  const parsed = JSON.parse(atob(text.trim()));
  if (typeof parsed?.door !== "string" || typeof parsed?.seed !== "string") {
    throw new Error("not a waiting room address");
  }
  return { holder: parsed.door, seed: parsed.seed, about: parsed.about ?? "" };
}

/*
 * Which room belongs to which circle, remembered per device.
 *
 * The room is a separate cell with its own seed, and nothing in the circle
 * points at it — a circle cannot hold a reference to a network its members
 * might not be in. So the holder's own device remembers, the same way it
 * remembers what she calls the circle.
 *
 * A consequence worth knowing: a holder who moves to a new device has the
 * circle but not the room, and would have to open a new one. Not fixed, and
 * noted in the docs rather than left to be discovered.
 */
const roomKey = (cellId) => `hearth:room:${asText(cellId?.[0])}`;

function rememberRoom(cellId, room) {
  try {
    localStorage.setItem(roomKey(cellId), JSON.stringify(room));
  } catch {
    // A convenience, not a rule. She can still invite by identifier.
  }
}

function roomFor(cellId) {
  try {
    const raw = localStorage.getItem(roomKey(cellId));
    return raw ? JSON.parse(raw) : null;
  } catch {
    return null;
  }
}

function forgetRoom(cellId) {
  try {
    localStorage.removeItem(roomKey(cellId));
  } catch {
    // Nothing to do about a browser that will not clear its own store.
  }
}

/** The properties a cloned cell was made with, unpacked. */
function propertiesOf(cell) {
  let props = cell?.dna_modifiers?.properties;
  if (props instanceof Uint8Array) {
    try {
      props = decode(props);
    } catch {
      return {};
    }
  }
  return props ?? {};
}

/** Every waiting-room cell on this device, by holder and seed. */
async function waitingRoomCells() {
  const info = await client.appInfo();
  const rooms = [];
  for (const raw of info.cell_info[ROLE] ?? []) {
    const c = raw?.value ?? raw?.cloned ?? raw;
    if (!c?.clone_id || c.enabled === false) continue;
    const props = propertiesOf(c);
    if (!props?.waiting_for) continue;
    rooms.push({
      cellId: c.cell_id,
      holder: props.waiting_for,
      seed: c.dna_modifiers?.network_seed ?? "",
    });
  }
  return rooms;
}

/** The cell for a room, opening it if this device is not in it yet. */
async function cellForRoom(room, name) {
  const already = (await waitingRoomCells()).find(
    (r) => r.holder === room.holder && r.seed === room.seed,
  );
  if (already) return already.cellId;

  const cell = await call("enter_waiting_room", {
    holder: room.holder,
    network_seed: room.seed,
    name,
  });
  return cell.cell_id;
}

// ---------------------------------------------------------------------------
// The holder's side: who is at the door
// ---------------------------------------------------------------------------

let knocking = [];

async function loadTheDoor() {
  const door = $("at-the-door");
  const address = $("door-address");
  const room = isHolder() ? roomFor(circle?.cellId) : null;

  if (!room) {
    door.hidden = true;
    address.hidden = true;
    currentRoomCell = null;
    knocking = [];
    $("knock-list").replaceChildren();
    return;
  }

  address.hidden = false;
  $("door-address-output").textContent = roomToAddress(room);

  let roomCell;
  try {
    roomCell = await cellForRoom(room, `${labelFor(circle.cellId, "Circle")} — door`);
  } catch (error) {
    /*
     * This was once a convenience sitting on top of inviting by identifier, so
     * losing it quietly was the right thing to do. It is now the only door
     * there is: if it cannot be opened, nobody can be let in and the holder
     * has to be told, not left looking at a screen that says nothing.
     */
    console.error(error);
    currentRoomCell = null;
    door.hidden = false;
    $("knock-list").replaceChildren();
    $("nobody-knocking").hidden = false;
    $("nobody-knocking").textContent =
      "The door could not be opened just now, so nobody waiting at it can be " +
      "seen. Nothing is lost — try again in a moment, or reopen the circle.";
    return;
  }

  /*
   * Held for anything later in this re-read that needs the door: delivering an
   * invitation somebody has just agreed to, and rebuilding the list after a
   * knock is put aside. Set here, where the room is actually opened, so it
   * survives a reload instead of depending on a button pressed earlier.
   */
  currentRoomCell = roomCell;

  knocking = await orNothingYet(call("get_knocks", null, roomCell), []);

  // Somebody who has already been answered is not still at the door.
  const waiting = knocking.filter((k) => !k.answered);

  door.hidden = false;
  $("nobody-knocking").hidden = waiting.length > 0;
  $("nobody-knocking").textContent =
    "Nobody is waiting. Give somebody the address below and they can ask from " +
    "their own Hearth.";

  const list = $("knock-list");
  list.replaceChildren();
  for (const item of waiting) {
    list.append(knockCard(item, roomCell));
  }
}

function knockCard(item, roomCell) {
  const li = document.createElement("li");
  li.className = "suggestion";

  const who = document.createElement("p");
  const said = item.relationship?.trim();
  // Never "Ronnie Smythe, her cousin" as though anybody had checked. Both
  // halves are what this person says about themselves.
  who.textContent = said
    ? `${item.name} — says they are ${said}`
    : item.name;
  li.append(who);

  /*
   * Somebody already in the circle, knocking again.
   *
   * One installation is one person, so knocking a second time under a
   * different name changes the label and not who they are. Offering "Let them
   * in" here offers a button that cannot do anything: their app declines to
   * join a circle it is already in, silently, and she is left pressing
   * something that does nothing.
   *
   * Found by walking it, and mistaken at first for a safeguard refusing a
   * second identity. It is not — nothing here refuses anybody, and a second
   * *installation* could knock under any name it liked. There was simply
   * nothing left to do, and the screen did not say so.
   */
  if (members.has(item.who)) {
    const already = document.createElement("p");
    already.className = "outcome";
    const known = members.get(item.who)?.name?.trim();
    already.textContent = known
      ? `${known} is already in this circle.`
      : "This person is already in this circle.";
    li.append(already);

    const sameKey = document.createElement("p");
    sameKey.className = "hint";
    sameKey.textContent =
      "One Hearth is one person, whatever name it asks under. Nothing to do.";
    li.append(sameKey);
    return li;
  }

  const key = document.createElement("p");
  key.className = "hint";
  key.textContent = item.who;
  li.append(key);

  const caution = document.createElement("p");
  caution.className = "hint";
  caution.textContent =
    "Anybody with the address can ask, and nothing here has been checked. " +
    "Only let in somebody you were expecting.";
  li.append(caution);

  const allow = document.createElement("button");
  allow.type = "button";
  allow.textContent = "Let them in";

  const ignore = document.createElement("button");
  ignore.type = "button";
  ignore.className = "secondary";
  ignore.textContent = "Not now";

  allow.addEventListener("click", () =>
    whileWorking(allow, "Letting them in…", async () => {
      ignore.disabled = true;
      try {
        await letThemIn(item, roomCell);
      } finally {
        ignore.disabled = false;
      }
    }).catch(problem),
  );

  /*
   * "Not now" writes nothing anywhere.
   *
   * A refusal recorded in an open room would be a public snub, readable by
   * everybody who has the address including the person refused. Nothing is
   * owed to somebody who knocked uninvited, and silence is the kindest
   * available answer as well as the safest.
   */
  ignore.addEventListener("click", () => {
    ignored.add(item.knock ? asText(item.knock) : item.who);
    renderIgnored();
    announce("Left where they are. Nothing was sent to them.");
  });

  const actions = document.createElement("div");
  actions.className = "actions";
  actions.append(allow, ignore);
  li.append(actions);

  return li;
}

/** Knocks put aside on this device, for this visit only. */
const ignored = new Set();

function renderIgnored() {
  const list = $("knock-list");
  list.replaceChildren();
  const waiting = knocking.filter(
    (k) => !k.answered && !ignored.has(asText(k.knock)) && !ignored.has(k.who),
  );
  $("nobody-knocking").hidden = waiting.length > 0;
  for (const item of waiting) {
    list.append(knockCard(item, currentRoomCell));
  }
}

let currentRoomCell = null;

/**
 * Let somebody in, by whichever route this circle uses.
 *
 * With a second person to agree, this puts them forward and the answer waits
 * on that agreement — the machinery already built for it, reused whole. With
 * nobody to agree, the invitation is made and left at the door immediately.
 */
async function letThemIn(item, roomCell) {
  if (seconderHere) {
    await call(
      "propose_member",
      { invitee: item.who, name: item.name },
      circle.cellId,
    );
    announce(
      `Put forward. ${
        members.get(seconderHere)?.name?.trim() || "The second person"
      } has to agree before they can be let in.`,
    );
    await loadCircle();
    return;
  }

  const invitation = await call(
    "invite",
    { invitee: item.who, name: item.name },
    circle.cellId,
  );
  await call(
    "admit",
    { knock: item.knock, invitation: invitationToToken(invitation) },
    roomCell,
  );
  announce(`${item.name} has been let in. Their app will open the circle.`);
  await loadCircle();
}

/**
 * Leave the invitation at the door for anybody now agreed to.
 *
 * She already decided when she pressed "Let them in"; the second person has
 * now agreed. Delivering it is the mechanical consequence of both, so it
 * happens rather than waiting for her to come back and press again — which is
 * the errand this whole thing exists to remove.
 */
async function deliverAnythingAgreed() {
  const room = isHolder() ? roomFor(circle?.cellId) : null;
  if (!room || !pending.length || !knocking.length) return;

  const roomCell = currentRoomCell;
  if (!roomCell) return;

  for (const person of pending) {
    if (!person.agreed || !person.invitation) continue;
    const theirKnock = knocking.find((k) => k.who === person.invitee && !k.answered);
    if (!theirKnock) continue;

    try {
      await call(
        "admit",
        {
          knock: theirKnock.knock,
          invitation: invitationToToken(person.invitation),
        },
        roomCell,
      );
      announce(`${person.name || "They"} have been let in.`);
    } catch (error) {
      // Reported to the console rather than the screen: she will see them
      // still listed, and the next re-read tries again.
      console.error(error);
    }
  }
}

// ---------------------------------------------------------------------------
// The other side: asking to be let in
// ---------------------------------------------------------------------------

let myRoomCell = null;

$("check-knock").addEventListener("click", async () => {
  try {
    await whileWorking($("check-knock"), "Looking…", lookForMyAdmission);
    if (!$("announcer").textContent) announce("Not yet. They have not looked.");
  } catch (error) {
    problem(error);
  }
});

/**
 * Have they said yes? If so, walk in.
 *
 * The invitation was left in the open room, which is safe because it is signed
 * over this key and is a useless blob to anybody else. So there is nothing to
 * paste and nothing to be sent: it is collected.
 */
/**
 * Every waiting room this device is standing in.
 *
 * Which room somebody knocked at was held in a variable, so closing the app or
 * the page reloading lost it — and with it the only thing looking for the
 * answer. They would have been left waiting at a door that had already been
 * opened, with no way back to it but knocking again.
 *
 * The conductor knows which rooms this device is in, and a knock is on their
 * own chain, so nothing actually needed remembering. Asked rather than stored.
 */
async function lookForMyAdmission() {
  if (myRoomCell) {
    await collectFrom(myRoomCell);
    return;
  }

  for (const room of await waitingRoomCells()) {
    if (circle) return;
    await collectFrom(room.cellId);
  }
}

async function collectFrom(roomCell) {
  const token = await orNothingYet(call("my_admission", null, roomCell), null);
  if (!token) return;

  /*
   * What they said when they knocked, read back from the knock itself.
   *
   * It used to be read off the form they typed it into, which is empty the
   * moment the page reloads — so somebody let in after a restart arrived
   * nameless, in a circle that then asked them who they were. They had
   * already said.
   */
  const knocks = await orNothingYet(call("get_knocks", null, roomCell), []);
  const mine = knocks.filter((k) => k.who === asText(me)).pop();

  const bundle = tokenToInvitation(token);
  const label = bundle.about?.trim() || "Their circle";

  /*
   * Already in, and only looking again because nothing said so.
   *
   * The answer stays in the room for good, and this device stays in the room
   * with it, so the same invitation is found again on every look. Without
   * this, coming back to the list of circles and waiting twenty seconds
   * produced "Tried to create a cell with an existing id" — a wasm error, in
   * front of somebody whose circle was working perfectly.
   */
  if (await circleAlreadyHere(bundle)) return;

  const cell = await call("join_circle", {
    founder: bundle.founder,
    name: label,
    network_seed: bundle.network_seed,
    invitation: bundle.invitation,
    requires_second_yes: Boolean(bundle.requires_second_yes),
    seconder: bundle.seconder ?? null,
  });

  circle = { cellId: cell.cell_id };
  holder = bundle.founder;
  setLabelFor(cell.cell_id, label);
  $("circle-heading").textContent = label;
  circles.push({ cellId: circle.cellId, name: label, madeWith: label });

  // Say who you are in the same breath as arriving, exactly as the invitation
  // route does — they already said it when they knocked, so do not ask again.
  const said = {
    name: mine?.name?.trim() || $("joiner-name").value.trim(),
    relationship: mine?.relationship?.trim() || $("joiner-relationship").value.trim(),
  };
  if (said.name) {
    await call("introduce_myself", said, circle.cellId);
  }

  alwaysAWayBack();
  show("circle");
  announce(`You are in ${label}.`);
  await loadCircle();
}
