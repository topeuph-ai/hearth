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

import {
  AppWebsocket,
  decodeHashFromBase64,
  encodeHashToBase64,
} from "@holochain/client";
// Records arrive with their entries still packed. Holochain speaks msgpack on
// the wire and does not unpack app entries for you.
import { decode } from "@msgpack/msgpack";
import QRCode from "qrcode";
import jsQR from "jsqr";

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
  // The new shape says what it is, and is worth recognising even when it
  // has a mistake in it — so the mistake can be pointed at, rather than the
  // whole thing being called unreadable.
  if (/^\s*hearth/i.test(text)) return true;
  try {
    const parsed = JSON.parse(atob(text.trim()));
    return typeof parsed?.door === "string" && typeof parsed?.seed === "string";
  } catch {
    return false;
  }
}

function invitationToToken(bundle) {
  const { seconded, appointment } = bundle.invitation;
  return btoa(
    JSON.stringify({
      ...bundle,
      invitation: {
        signature: bytesToBase64(bundle.invitation.signature),
        // Absent until the second person has agreed. An invitation without it,
        // to a circle that asks for one, is not a weak invitation — it is an
        // unfinished one.
        seconded: seconded ? bytesToBase64(seconded) : null,
        /*
         * Which appointment the second agreement was given under.
         *
         * Without it the door has nothing to check the second signature
         * against, and lets the invitation in on the holder's signature alone
         * — so every invitation collected from a waiting room arrived unchecked,
         * even ones two people had genuinely agreed to. Absent (null) is still
         * right for an invitation made before anybody was appointed.
         */
        appointment: appointment ? encodeHashToBase64(appointment) : null,
        // Part of what both people signed. Dropped, the signatures no longer
        // match and the door refuses the invitation.
        name: bundle.invitation.name ?? "",
      },
    }),
  );
}

function tokenToInvitation(token) {
  const parsed = JSON.parse(atob(token.trim()));
  const { appointment } = parsed.invitation;
  return {
    ...parsed,
    invitation: {
      signature: base64ToBytes(parsed.invitation.signature),
      seconded: parsed.invitation.seconded
        ? base64ToBytes(parsed.invitation.seconded)
        : null,
      // Tokens made before this was carried have no appointment at all, and
      // still read — as the holder's own invitation, which is what they were.
      appointment: appointment ? decodeHashFromBase64(appointment) : null,
      name: parsed.invitation.name ?? "",
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
// Where this device stands on keys: { epoch, mine, waiting_for }. The circle's
// newest key, the newest this device can use, and anybody whose encryption key
// has not reached the holder yet.
let keys = null;

const $ = (id) => document.getElementById(id);

/*
 * Say so, when this is the practice copy.
 *
 * `import.meta.env.DEV` is true when the interface is served by the development
 * server — which is what `npm run demo` does, and what an installed Hearth
 * never does: it is built, and serves files from inside the app.
 *
 * This exists because the two are the same interface and cannot be told apart
 * by looking. Three demo windows were once opened beside a real Hearth; a
 * circle was made in one of them, and an invitation to it was sent to somebody
 * in another town, who could never have reached it. The window said nothing,
 * so now it does. The title changes too, for the taskbar.
 */
if (import.meta.env.DEV) {
  const banner = $("practice-copy");
  banner.hidden = false;
  // And the one simplification worth naming: the demo does not ask for a
  // password, and a released Hearth does. Decided 23 September 2026.
  $("no-password-here").hidden = false;
  document.title = "Practice copy — Hearth demo";

  // The announcer is fixed to the top of the window and would land on the
  // banner. Tell it how far down to start — measured, because the banner wraps
  // to two lines on a narrow window.
  const measure = () =>
    document.documentElement.style.setProperty(
      "--practice-height",
      `${banner.offsetHeight}px`,
    );
  measure();
  window.addEventListener("resize", measure);
}

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
    "pass-reader",
    "circles",
    "circle",
    "problem",
  ]) {
    $(id).hidden = !ids.includes(id);
  }
  // The camera goes off with the screen that asked for it. A problem screen
  // shown over a failed join is the one exception: they will come back to it.
  // Guarded because a screen can be shown before the camera code further down
  // this file has run, and then there is no camera to stop.
  if (!ids.includes("join") && !ids.includes("problem")) {
    try {
      stopScanning();
    } catch {
      // Nothing was scanning.
    }
  }
  // Words read with a pass go when the screen does: nothing is kept.
  if (!ids.includes("pass-reader")) {
    try {
      stopScanningPass();
      aFreshPassReading();
    } catch {
      // Shown before the pass code further down has run.
    }
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
 * The words of the record, as this device can read them.
 *
 * The record is locked with the circle's key, so its entry holds sealed bytes
 * and the words come back opened beside it. Nothing on this side holds a key
 * or does any unlocking: the zome asks the keystore, and this asks the zome.
 *
 * Null where there is no record, and null where there is one this device
 * cannot open — a device that has been removed, or one whose keys have not
 * caught up. `current.locked_out` tells those two apart, and the screen says
 * so rather than showing an empty record as though nothing were written.
 *
 * Falls back to the entry for a circle written before encryption, where the
 * words really are in the open.
 */
function wordsOf(current) {
  return current?.about_me ?? entryOf(current?.record);
}

/**
 * What a suggestion says, as this device can read it.
 *
 * The same arrangement as the record: the suggestion and the why are locked
 * with the circle's key and come back opened beside the entry. Which section
 * it is about is in the open, so a suggestion this device cannot open is
 * skipped rather than shown as a blank card — which is why this returns
 * nothing at all when the words are missing.
 */
function suggestionWords(item) {
  if (item?.words) return item.words;
  // Falling back to the entry is right only for one written before
  // suggestions were locked. A locked one this device could not open used to
  // come back as its empty shell — a blank card, and an empty suggestion
  // carried into a moved circle. Found in review, 23 September 2026.
  const entry = entryOf(item?.suggestion);
  return entry && !entry.locked ? entry : null;
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

/*
 * Circles made under earlier rules, and the connection each one needs.
 *
 * When the rules a circle is built from change, the new version of the app is
 * installed beside the old one rather than replacing it, so both are running
 * in the same conductor against the same keystore — the same person, with two
 * sets of rules. The circles somebody already has live in the older one.
 *
 * Keyed by the circle's own identity, so `call` below can send each question
 * to the app that can answer it without anything else in here knowing that
 * two apps exist. See docs/upgrades.md.
 */
const earlierClients = new Map();

async function call(fnName, payload, cellId) {
  const where = cellId ? earlierClients.get(asText(cellId[0])) : null;
  const speaking = where?.client ?? client;

  return speaking.callZome({
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

  /*
   * The same name, where the sentence needs it to own something.
   *
   * "Remove Margaret Smythe circle from my device" is what you get without
   * this, and "Remove their circle" is what you get when there is no name
   * yet. Found by reading the button rather than the code.
   */
  for (const span of document.querySelectorAll(".about-whose")) {
    span.textContent = name?.trim() ? `${who}'s` : "their";
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
    wordsOf(record?.current)?.display_name?.trim();
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
 * `showCircleMode` below is the only thing allowed to set these elements.
 */
const READING = "reading";
const WRITING = "writing";
const SAYING_I_READ_IT = "acknowledging";

let circleMode = READING;

/** Whether this device has a door to this circle at all. See loadTheDoor. */
let theDoorIsHere = false;

/** Whether the circle on screen says it asks two people to agree. */
let circleAsksTwo = false;

/*
 * Which of the circle's four pages is showing.
 *
 * Reading the record, seeing who is in the circle, inviting somebody, and
 * looking at what people have offered are four different reasons to be here.
 * Stacked on one page, three of them were the bottom half of something else —
 * which is how somebody came to be offered "give somebody the address" above
 * a record they had not read yet.
 */
const CIRCLE_PAGES = ["record", "people", "invite", "suggestions"];
let circleShows = "record";

/*
 * Whether the record on screen has just this moment been written.
 *
 * Writing it is a run of pages that ends with reading the whole thing over,
 * and offering three other places to be in the middle of that is an
 * invitation to wander off before looking at it. So the tabs wait until she
 * has been to the end of that run and pressed on.
 *
 * Only for the person who wrote it, and only the once. Coming back to a
 * circle later, or opening somebody else's, starts with the tabs there.
 */
let justWroteIt = false;

/**
 * Put the circle screen into a mode. The only place these are set.
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
  /*
   * "Write it" — only until the pages have been through once.
   *
   * After that, every section has its own "Change this", and a second button
   * that walked all seven in a row was one nobody could explain: it was
   * called "Change all of it", and the first person to read it asked whether
   * it cleared everything and started again. A button that has to be
   * explained is a button that frightens somebody off it, or worse, one they
   * press expecting something else.
   *
   * It stays for the one case with no other way in: a circle whose pages were
   * never finished — the app closed halfway, say — which shows "Nothing has
   * been written yet" and nothing to press beside any section.
   */
  $("edit-record").hidden = !amHolder || beenThroughOnce;
  $("edit-record").textContent = "Write it";

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

  /*
   * Nothing to offer anybody until there is something to offer them.
   *
   * The list of people, the queue at the door, and the address to give out
   * were all on screen from the moment the circle existed — under a form that
   * had not been filled in yet. So the first thing the app said to somebody
   * who had just made a circle was "give somebody the address", and what they
   * would have been giving the address to was a name and seven empty
   * headings.
   *
   * This rule is not new. It used to guard the panel that made invitations,
   * word for word: nothing to invite anybody to until something has been
   * written. That panel was deleted when invitations stopped being something
   * people see, and the rule went with it instead of moving to the door that
   * replaced it.
   *
   * The second half is the same rule the buttons above follow: while somebody
   * is part-way through writing, the screen underneath them holds still.
   */
  // Once the holder has been through the pages — whatever is in them.
  const somethingToShowPeople = beenThroughOnce && mode !== WRITING;

  /*
   * The tabs, and which page they lead to.
   *
   * Nothing but the record until something is written: three tabs offering
   * people, invitations and suggestions about a record that does not exist
   * yet is three ways to be disappointed. And nothing at all while the form
   * is open, which is the rule the buttons above already follow.
   */
  const mayRoam = somethingToShowPeople && !justWroteIt;
  $("circle-tabs").hidden = !mayRoam;

  // Only the holder invites anybody, so only she has the tab for it.
  const inviteTab = document.querySelector('.circle-tab[data-page="invite"]');
  inviteTab.hidden = !amHolder;

  // Somewhere to be, whatever has just been hidden underneath you.
  if (!mayRoam) circleShows = "record";
  if (circleShows === "invite" && !amHolder) circleShows = "record";

  for (const page of CIRCLE_PAGES) {
    $(`circle-${page}`).hidden = page !== circleShows;
  }
  for (const tab of document.querySelectorAll(".circle-tab")) {
    const here = tab.dataset.page === circleShows;
    tab.classList.toggle("here", here);
    tab.setAttribute("aria-current", here ? "page" : "false");
  }

  // What a member may do with the record, said above it.
  $("members-can-suggest").hidden = amHolder || !beenThroughOnce;
  sayWhoDecidesSuggestions();

  $("people").hidden = !somethingToShowPeople;
  $("at-the-door").hidden = !somethingToShowPeople || !theDoorIsHere;
  $("door-address").hidden = !somethingToShowPeople || !theDoorIsHere;
  // A pass reads through the same door, so it is offered wherever that is.
  $("passes").hidden = !somethingToShowPeople || !theDoorIsHere;

  /*
   * The way on from the record, for the holder who has just written it.
   *
   * A member sees "I have read this" here instead, which is the thing they
   * came to do. Both were on screen at once, side by side and both in the
   * same weight, so the two buttons looked like the same button twice.
   */
  $("carry-on").hidden =
    !somethingToShowPeople || circleShows !== "record" || !amHolder;

  /*
   * Whose invitation should go first.
   *
   * Only while it is still true: once somebody has agreed to be the second
   * person, the sentence has done its job and saying it again would be the
   * screen nagging about something already done.
   */
  $("ask-them-first").hidden = !circleAsksTwo || whoAgrees?.willing === true;

}

function showCirclePage(which) {
  circleShows = which;
  showCircleMode();
  // Opening the tab is the answer to being asked. The number stays: it is a
  // fact about who is outside, not a bell still ringing.
  sayWhoIsWaiting();
  // A box that was hidden measured nothing, so ask again now it can be seen.
  if (which === "record") sayWhichAnswersRunLong();
  window.scrollTo({ top: 0 });
}

for (const tab of document.querySelectorAll(".circle-tab")) {
  tab.addEventListener("click", () => {
    if (tab.dataset.page === "suggestions") suggestingAboutOneSection(false);
    showCirclePage(tab.dataset.page);
  });
}

// Carrying on from the record goes where the holder has to go next: the
// address, and who to send it to first. It is also the moment she has read
// the record over, which is what the rest of the circle was waiting for.
$("carry-on").addEventListener("click", () => {
  justWroteIt = false;
  showCirclePage("invite");
});

/** Whether the last load put a written record on the screen. */
let showingSomething = false;

/*
 * Whether the holder has been through the writing pages once — with or
 * without writing anything.
 *
 * The rest of the circle used to wait until something had been written. That
 * put pressure in the wrong place: making the circle is the job that has to
 * be done now, and what matters to somebody is often the thing a family needs
 * a week to think about. Leaving every box empty and pressing Save is a real
 * choice, and it now lands on the same page as filling all seven in.
 *
 * Read from the record rather than remembered, so it survives a reload and is
 * the same on every device. Making a circle saves a first version holding
 * only the name; going through the pages saves another on top of it. So the
 * record being an update is the answer — checked against a running conductor
 * before being relied on. Anything already written counts too, for circles
 * made before this.
 */
let beenThroughOnce = false;

const hasBeenThroughOnce = (current) =>
  current?.record?.signed_action?.hashed?.content?.data?.type === "Update" ||
  hasBeenWritten(wordsOf(current));

/**
 * When a record was written, in the words a person would use.
 *
 * Holochain stamps every action with microseconds since 1970, in the action's
 * header — `signed_action.hashed.content.header.timestamp` in 0.7, the same
 * place the author moved to. It can arrive as a number or a BigInt depending
 * on the client, so it is converted rather than assumed.
 *
 * "Today" and "yesterday" are compared as dates on this device's calendar,
 * not as twenty-four hours, because "yesterday" at ten past midnight means
 * the day before, not an hour ago.
 */
function whenItWasWritten(record, now = new Date()) {
  const raw = record?.signed_action?.hashed?.content?.header?.timestamp;
  if (raw === undefined || raw === null) return null;

  const written = new Date(Number(raw) / 1000);
  if (Number.isNaN(written.getTime())) return null;

  const day = (d) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
  const daysAgo = Math.round((day(now) - day(written)) / 86_400_000);

  if (daysAgo === 0) return "today";
  if (daysAgo === 1) return "yesterday";

  return written.toLocaleDateString("en-GB", {
    day: "numeric",
    month: "long",
    year: "numeric",
  });
}

/**
 * Open the suggestion box already pointed at one section.
 *
 * The sections in the box's list are in the standard's order, and so are the
 * sections of the record — so the section beside the button is the option at
 * the same position. Checked rather than trusted: if the two lists ever stop
 * lining up, the box opens without choosing, rather than choosing wrongly.
 */
function suggestAbout(index) {
  const choose = $("suggest-field");
  showCirclePage("suggestions");

  if (choose.options.length === FIELDS.length) {
    choose.selectedIndex = index;
    $("suggesting-about-section").textContent =
      `“${choose.options[index].textContent.trim()}”`;
    suggestingAboutOneSection(true);
  } else {
    // The lists no longer line up. Ask, rather than choose wrongly.
    suggestingAboutOneSection(false);
  }

  $("suggest-section").scrollIntoView({ block: "start" });
  $("suggest-text").focus();
}

/** Whether the section is already chosen (arrived from it) or to be asked. */
function suggestingAboutOneSection(fixed) {
  $("suggesting-about").hidden = !fixed;
  $("suggest-field-choice").hidden = fixed;
}

$("suggest-another-section").addEventListener("click", () => {
  suggestingAboutOneSection(false);
  $("suggest-field").focus();
});

function renderRecord(current) {
  const entry = wordsOf(current);

  // Her name is hers whether or not anything has been written yet, and every
  // question on this screen is phrased around it.
  if (entry?.display_name) nameHer(entry.display_name);

  /*
   * A record this device cannot open is not an empty record, and must never
   * look like one. It means one of two things — this device has been removed
   * from the circle, or its keys have not caught up yet — and either way the
   * honest thing is to say so and show nothing.
   */
  // A circle from before the rules changed, and only the person who holds it
  // can carry it across. Everybody else simply reads it as it is.
  $("older-rules-circle").hidden = !(circle?.olderRules && isHolder());

  $("locked-out").hidden = !current?.locked_out;
  if (current?.locked_out) {
    $("no-record").hidden = true;
    $("record").hidden = true;
    return;
  }

  if (!hasBeenThroughOnce(current)) {
    $("no-record").hidden = false;
    $("record").hidden = true;
    return;
  }

  $("no-record").hidden = true;
  $("record").hidden = false;
  $("record-name").textContent = entry.display_name;

  // The standard's "Date last updated". Hidden rather than guessed at if the
  // action somehow arrives without one.
  const when = whenItWasWritten(current.record);
  $("last-updated").hidden = !when;
  $("last-updated").textContent = when
    ? when === "today" || when === "yesterday"
      ? `Last updated ${when}.`
      : `Last updated on ${when}.`
    : "";

  const list = $("record-fields");
  list.replaceChildren();
  for (const [index, [key, label]] of FIELDS.entries()) {
    const dt = document.createElement("dt");
    dt.textContent = label;
    // So a suggestion can find the words it is about, and sit next to them
    // rather than in a pile at the bottom of the page.
    dt.dataset.field = key;

    /*
     * Every section, written or not.
     *
     * Empty sections used to be left off, which was tidy while there was
     * always something in most of them. Now a record can be all seven empty
     * on purpose, and a page with nothing on it would give the holder nowhere
     * to press "Change this" when she is ready — and give everybody else
     * nowhere to offer a suggestion about the thing that is missing, which is
     * exactly when a suggestion is most useful.
     */
    const dd = document.createElement("dd");
    const said = entry[key]?.trim();
    dd.textContent = said ? entry[key] : "Nothing written here yet.";
    if (!said) dd.classList.add("nothing-yet");
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

    // Photos, sound and video beside this section, filled in by showMedia.
    const media = document.createElement("div");
    media.className = "media-here";
    media.dataset.mediaFor = key;
    group.append(media);

    /*
     * Longer than the box, and a way to see the rest.
     *
     * "How and when to support me" is the section that runs long — routines,
     * mealtimes, what changes in hospital — and a box that silently cuts it
     * off is worse than one that scrolls, because nothing tells you there is
     * more. The box scrolls either way; this says out loud that it needs to.
     *
     * Made now and shown later. Whether the words overflow depends on the
     * window, the font and the reader's own text size, so it has to be
     * measured rather than guessed at from how many characters there are —
     * and it cannot be measured until the page has been laid out. See
     * sayWhichAnswersRunLong.
     */
    const more = document.createElement("button");
    more.type = "button";
    more.className = "linky show-all";
    more.hidden = true;
    more.textContent = "Show all of it";
    more.addEventListener("click", () => {
      const open = dd.classList.toggle("all-of-it");
      more.textContent = open ? "Show less" : "Show all of it";
    });
    group.append(more);

    /*
     * Change this one, from here.
     *
     * Fixing one line used to mean "Change all of it" and pressing Continue
     * until you reached the right page. The sections are in the standard's
     * order and so are the pages, so the one beside this heading is the one
     * to open.
     */
    if (isHolder()) {
      const change = document.createElement("button");
      change.type = "button";
      change.className = "linky change-one";
      change.textContent = "Change this";
      change.addEventListener("click", () => changeOneSection(index));
      group.append(change);

      const photo = document.createElement("button");
      photo.type = "button";
      photo.className = "linky change-one";
      photo.textContent = "Add a photo";
      photo.addEventListener("click", () => pickAPhoto(key));
      group.append(photo);

      const sound = document.createElement("button");
      sound.type = "button";
      sound.className = "linky change-one";
      sound.textContent = "Add sound";
      sound.addEventListener("click", () => pickASound(key));
      group.append(sound);

      const video = document.createElement("button");
      video.type = "button";
      video.className = "linky change-one";
      video.textContent = "Add video";
      video.addEventListener("click", () => pickAVideo(key));
      group.append(video);
    } else {
      /*
       * And for everybody else, the same place to start from.
       *
       * Offering a suggestion meant going to another tab and choosing, from a
       * list, the section you had just been reading. The person reading a
       * section is the person who has noticed something about it, so the way
       * to say so belongs beside it.
       */
      const suggest = document.createElement("button");
      suggest.type = "button";
      suggest.className = "linky change-one";
      // They cannot change anything, so the button does not say "change".
      suggest.textContent = "Offer a suggestion";
      suggest.addEventListener("click", () => suggestAbout(index));
      group.append(suggest);
    }

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
  sayWhichAnswersRunLong();

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

/*
 * Which answers are longer than the box they are in.
 *
 * Asked of the laid-out page, one frame after it is drawn. Asked any earlier
 * every answer measures zero against zero — the record may not even be on
 * screen yet when it is built — and the button would never appear on
 * anything.
 */
function sayWhichAnswersRunLong() {
  requestAnimationFrame(() => {
    for (const group of document.querySelectorAll(".record-field")) {
      const dd = group.querySelector("dd");
      const more = group.querySelector(".show-all");
      if (!dd || !more) continue;
      // Nothing to say while it is open: the button says "Show less" and is
      // the only way back.
      const open = dd.classList.contains("all-of-it");
      more.hidden = !open && dd.scrollHeight <= dd.clientHeight + 2;
    }
  });
}

/*
 * Who has read the record.
 *
 * Each item is the acknowledgement itself and the role its author claimed,
 * opened with the circle's key by the zome — the role is locked in the circle,
 * so it cannot be read off the entry here.
 */
function renderReaders(allReads, earlier = []) {
  const section = $("readers");
  const list = $("readers-list");
  list.replaceChildren();
  // Nothing written by somebody after they were removed. See writtenWhileGone.
  const records = allReads.filter((item) => !writtenWhileGone(item.record));

  if (!records.length && !earlier.length) {
    section.hidden = true;
    return;
  }
  section.hidden = false;

  // Read before the circle moved, while the words were the same as now.
  for (const item of earlier) {
    const li = document.createElement("li");
    li.textContent =
      `${item.who} read this before the circle moved` +
      (item.when ? `, on ${new Date(item.when).toLocaleDateString("en-GB")}` : "") +
      `. Role claimed: ${item.role}`;
    list.append(li);
  }

  for (const item of records) {
    const li = document.createElement("li");
    // Never "Read by District Nurse" — that implies a credential nobody
    // checked. The claim and the claimant are shown as separate facts.
    const who = describe(authorOf(item.record));
    // A role this device cannot open is said as that, not left blank: that
    // somebody read the record is the evidence, and it is not in doubt.
    li.textContent = item.locked_out
      ? `${who} read this. What they said they were cannot be read on this device.`
      : `${who} read this. Role claimed: ${item.role}`;
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
    await keepKeysUpToDate();
    await drawTheCircle();
  } finally {
    readingNow = null;
  }
}

/**
 * Keys, before anything is read or written.
 *
 * The record and everything else in the circle is locked, so a device with no
 * key has nothing to show and nothing it can write. This publishes this
 * device's encryption key, takes up whatever the holder has sealed to it, and —
 * on the holder's own device — seals the circle's keys to everybody owed one.
 * It writes nothing when nothing has changed, which is almost every time.
 *
 * Never allowed to stop the screen being drawn. A circle whose keys have not
 * arrived yet still has people in it, a name at the top, and things to say
 * about what cannot be opened — see the locked-out note.
 */
async function keepKeysUpToDate() {
  try {
    keys = await call("keep_keys_up_to_date", null, circle.cellId);

    /*
     * Deliberately not on the screen. From inside this device, "my key is
     * older than the circle's" is the same fact whether somebody has just
     * joined or has been removed, and the app must not guess which and
     * reassure the wrong person. The locked-out note says both readings out
     * loud instead. This is here for whoever is looking at a log.
     */
    if (keys && keys.mine < keys.epoch) {
      console.info(
        `This device can use key ${keys.mine} of ${keys.epoch} in this circle.`,
      );
    }
  } catch (error) {
    console.error("Could not bring this device's keys up to date.", error);
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
  // Who has been removed, before anything is drawn — and if it is me, the
  // circle comes off this device and there is nothing to draw.
  if (await readDepartures()) return;

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

  const entry = wordsOf(current);
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
  beenThroughOnce = haveIt && hasBeenThroughOnce(current);

  record = haveIt ? { original, current } : null;
  renderRecord(haveIt ? current : null);
  // Not awaited: pictures arrive after the words, and the words should not
  // wait for them.
  showMedia().catch((error) => console.error("Could not show media.", error));

  $("no-record-empty").hidden = beenThroughOnce || !amHolder;
  $("no-record-waiting").hidden = beenThroughOnce || amHolder;

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


  sayItMovedIfItDid();

  /*
   * Who read it before the move, for as long as it is still the same words.
   *
   * An acknowledgement is of one version. The copy that opened this circle is
   * the words they read, so their reading still stands beside it; the first
   * change after the move makes it a different version, and it stops being
   * shown, exactly as an acknowledgement in the old circle would have.
   */
  const unchangedSinceTheMove =
    current?.record?.signed_action?.hashed?.content?.data?.type === "Create";
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
    written && unchangedSinceTheMove ? historyFor(circle.cellId)?.readers ?? [] : [],
  );

  await loadMembers();
  await loadSuggestions();
  await readSuccession();
}

/*
 * A suggestion that was set aside is shown only to the holder, who decided,
 * and to whoever offered it, who deserves to know what became of it.
 *
 * Everybody else used to see every one, forever. Ceri's reasons for stopping
 * that, both of them: a circle of ten loving, enthusiastic people will have a
 * great deal to suggest, and the box fills with things that went nowhere; and
 * something sensitive a support worker offered, which the holder turned down,
 * stayed readable by the whole circle anyway.
 *
 * What this is and is not, the same as removal: every member's app hides it,
 * and it is still on their devices, because nothing written in a circle can be
 * unwritten. A modified app could show it. See docs/hard-questions.md.
 */
function stillWorthShowing(item) {
  if (writtenWhileGone(item.suggestion)) return false;
  if (isHolder()) return true;
  const outcome = entryOf(item.outcome);
  if (!outcome || outcome.accepted) return true;
  return asText(authorOf(item.suggestion)) === asText(me);
}

async function loadSuggestions() {
  const amHolder = isHolder();

  // The person whose circle it is edits the record directly; everyone else
  // offers. Both see the list, so a carer can tell that what she noticed was
  // used.
  $("suggest-section").hidden = amHolder;

  const everything = await orNothingYet(
    call("get_suggestions", null, circle.cellId),
    [],
  );
  suggestions = everything.filter(stillWorthShowing);
  renderSuggestions();
}

function fillForm() {
  const entry = wordsOf(record?.current);
  // In the order they appear on the form.
  $("what-matters").value = entry?.what_matters_to_me ?? "";
  $("people-who-matter").value = entry?.people_who_matter ?? "";
  $("how-to-communicate").value = entry?.how_to_communicate_with_me ?? "";
  $("my-wellness").value = entry?.my_wellness ?? "";
  $("please-do").value = entry?.please_do_and_please_do_not ?? "";
  $("how-to-support").value = entry?.how_to_support_me ?? "";
  $("also-worth-knowing").value = entry?.also_worth_knowing ?? "";
  $("supported-by").value = entry?.supported_to_write_this_by ?? "";
  // A section written before the word limit existed says so the moment it is
  // opened, rather than when somebody tries to save it.
  for (const id of WORD_LIMITED) sayHowManyWords($(id));
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
    showRecordPage(0);
  } catch (error) {
    problem(error);
  }
});

/*
 * Writing the record, one section to a page.
 *
 * It was seven boxes in a column, and the bottom of it was below the fold.
 * Worse, nothing said what any of them wanted — so the honest thing to do was
 * leave them empty, and an empty About Me helps nobody. Each page now has one
 * section, what that section is for, and the standard's own prompts for it.
 *
 * Nothing is submitted between pages. This is one form; the pages are only
 * what is on screen, so moving about costs nothing and nothing is written
 * until Save.
 */
const recordPages = () => [...document.querySelectorAll(".record-page")];
let recordPage = 0;

/*
 * Whether this visit to the form is about one section or about all of them.
 *
 * Pressing "Change" beside a section is a small job with a small ending: save
 * that one and go straight back to the record. Walking the whole form is a
 * different job, and offering "save this one" half way through it invites
 * somebody to save a record they are in the middle of writing.
 */
let editingOneSection = false;

function showRecordPage(which) {
  const pages = recordPages();
  recordPage = Math.max(0, Math.min(which, pages.length - 1));

  pages.forEach((page, i) => {
    page.hidden = i !== recordPage;

    // Working through the whole form: forward, back, and back to the start,
    // with nowhere to go back to from the first page.
    for (const button of page.querySelectorAll(".record-back, .record-restart")) {
      button.hidden = editingOneSection || recordPage === 0;
    }
    for (const button of page.querySelectorAll(".record-next")) {
      button.hidden = editingOneSection;
    }

    // Changing one section: save it, or go back without changing. Nothing else.
    for (const button of page.querySelectorAll(".record-save-one")) {
      button.hidden = !editingOneSection;
    }

    /*
     * Going back without changing only means something when there is
     * something written to go back to.
     *
     * Writing it for the first time, straight after making the circle, there
     * is nothing — so "go back without changing" was a way out to an empty
     * page, under every one of the eight. Every section can already be passed
     * over with Continue, so there is no trap in leaving it off.
     *
     * Asked of what has been *written*, not of whether a record exists. The
     * first attempt asked the second, and a record always exists by then:
     * making a circle saves one straight away with only the person's name in
     * it, so that it is never nameless. That made every first pass look like
     * an edit, and the button stayed.
     */
    for (const button of page.querySelectorAll(".record-cancel, #cancel-edit")) {
      button.hidden = !beenThroughOnce;
    }
  });

  // Where you are, in words. A form with no end in sight is a form people
  // abandon, and eight pages with nothing to say how many is worse than one
  // long one. Changing a single section is not eight pages and does not need
  // counting.
  $("record-progress").hidden = editingOneSection;
  $("record-progress").textContent = `Page ${recordPage + 1} of ${pages.length}`;

  /*
   * Bring the section into view, and let it be seen arriving.
   *
   * This used to put the cursor in the box and then jump to the top of the
   * page — which undid the scroll the cursor had just caused. Pressing
   * "Change this" on a section left the heading, the hint and four prompts
   * filling the window, with the box that had just opened sitting below the
   * bottom edge and nothing to say it was there.
   *
   * So the page now glides to the section itself, stopping just under the
   * name bar that stays at the top, which puts the section's heading, its
   * box and the button under it on screen together. The movement is part of
   * the point: it shows somebody that something opened. For anybody whose
   * computer asks for less motion, it goes straight there instead.
   */
  const page = pages[recordPage];
  const box = page?.querySelector("textarea, input");
  if (page) {
    const bar = document.querySelector(".circle-bar");
    const under = (bar?.getBoundingClientRect().height ?? 0) + 12;
    const still = window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    window.scrollTo({
      top: Math.max(0, page.getBoundingClientRect().top + window.scrollY - under),
      behavior: still ? "auto" : "smooth",
    });
  }
  // Focused without scrolling, so it does not fight the scroll above.
  if (box) box.focus({ preventScroll: true });
}

for (const button of document.querySelectorAll(".record-next")) {
  button.addEventListener("click", () => showRecordPage(recordPage + 1));
}
for (const button of document.querySelectorAll(".record-back")) {
  button.addEventListener("click", () => showRecordPage(recordPage - 1));
}
for (const button of document.querySelectorAll(".record-restart")) {
  button.addEventListener("click", () => showRecordPage(0));
}

/*
 * Every way out of this form is the same way out.
 *
 * There is one on the last page with an id, because that is the one the rest
 * of the app already talks to; the others are the same button repeated on
 * every page, so that leaving never means walking to the end first.
 */
for (const button of document.querySelectorAll(".record-cancel")) {
  button.addEventListener("click", () => $("cancel-edit").click());
}

$("edit-record").addEventListener("click", () => {
  editingOneSection = false;
  fillForm();
  showCircleMode(WRITING);
  showRecordPage(0);
});

/**
 * Change one section, from the record itself.
 *
 * Everything about the form is the same — it is one form and it saves all
 * seven either way. What changes is what is on screen: the page you asked
 * for, a button that saves and comes back, and nothing that invites you to
 * walk through the other six.
 */
function changeOneSection(which) {
  editingOneSection = true;
  fillForm();
  showCircleMode(WRITING);
  showRecordPage(which);
}

$("cancel-edit").addEventListener("click", () => {
  editingOneSection = false;
  showCircleMode(READING);
  // Somewhere visible to land. "Write it" is gone once the pages have been
  // through, so focus the record itself instead of a hidden button.
  const landing = beenThroughOnce ? $("record-name") : $("edit-record");
  if (landing === $("record-name")) landing.setAttribute("tabindex", "-1");
  landing.focus();
});

$("record-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    /*
     * Whether this save is the first time anything has been written.
     *
     * Read before saving, because saving is what changes the answer. And
     * asked of what has been written rather than whether a record exists,
     * for the reason given in showRecordPage: making a circle already saved
     * a record with a name and nothing else in it, so "is there a record?"
     * was always yes, and the tabs that are meant to wait for "Carry on"
     * after the first pass never waited.
     */
    const firstTimeWritten = !beenThroughOnce;
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
      // Nothing on this form edits coded values, so whatever the record
      // already carries is kept rather than quietly dropped on saving.
      codes: wordsOf(record?.current)?.codes ?? [],
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

    // Written for the first time, so the next thing is to read it over —
    // and the rest of the circle waits until that has happened.
    if (firstTimeWritten) justWroteIt = true;

    editingOneSection = false;
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
      // No pressure in the wording either: an empty record is a choice, and
      // the circle is ready to use without one.
      announce(
        "Saved. Your circle is ready. Fill in each part whenever you are ready — press Change this beside it.",
      );
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
    // Before the hidden check: somebody who moved a circle and then looked at
    // another window still needs the stragglers told.
    carryOnMoving().catch((error) => console.error(error));

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

  // Circles made before the rules changed live in the older app beside this
  // one. Reaching them is allowed to fail without stopping anything.
  await connectToEarlierRules();

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

  // A move that was not finished when the app last closed.
  carryOnMoving().catch((error) => console.error(error));

  watchForArrivals();

  // Someone read the record. Told to us by their device, not by a server.
  // When each knocker was last announced, and when the circle last reloaded
  // for a knock. See the Knocked branch below.
  const knockSaid = new Map();
  let knockReloaded = 0;

  const whenSomebodyTellsUsSomething = async (signal) => {
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
    /*
     * Held to a trickle, whatever arrives.
     *
     * Anybody with the door's address can send this without knocking at all:
     * a signal is not an entry, so the ten-knock limit every device enforces
     * does not touch it. Found in audit, 23 September 2026: a stranger could
     * make the holder's screen announce and reload the whole circle as fast as
     * they could send. The real knock is an entry, arrives by itself, and is
     * listed whatever happens here; this only decides how often to say so.
     */
    if (payload?.kind === "Knocked") {
      const now = Date.now();
      const from = asText(payload.by);
      if (now - (knockSaid.get(from) ?? 0) > 60_000) {
        knockSaid.set(from, now);
        const who = payload.name?.trim() || "Somebody";
        const said = payload.relationship?.trim();
        announce(
          said
            ? `${who} is asking to join. They say they are ${said}.`
            : `${who} is asking to join.`,
        );
      }
      if (circle && now - knockReloaded > 15_000) {
        knockReloaded = now;
        await loadCircle();
      }
      return;
    }
    // A pass was used at one of this device's doors. Only ever raised by this
    // device itself — the zome drops one arriving from anywhere else.
    if (payload?.kind === "PassUsed") {
      if (!rememberPassRead(asText(signal?.value?.cell_id?.[0]), payload)) return;
      announce(
        `${payload.for_whom} read ${payload.sections.map(sectionName).join(", ")} ` +
          `with a pass.`,
      );
      if (currentRoomCell) await loadPasses().catch(() => {});
      return;
    }
    if (payload?.kind === "Admitted") {
      announce("You have been let in.");
      await lookForMyAdmission();
      return;
    }
    // From a circle that may not be on screen, and is about to not exist
    // here at all. See followTheMove.
    if (payload?.kind === "Moved") {
      await followTheMove(signal?.value?.cell_id, payload).catch((error) =>
        console.error("Could not follow a circle that moved.", error),
      );
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
  };

  client.on("signal", whenSomebodyTellsUsSomething);

  /*
   * And the same ear on every earlier version of the rules still installed.
   *
   * Without this, the one signal that matters most across a version change
   * would be missed: the holder telling everybody that the circle has moved.
   * It is sent in the old circle, which belongs to the old app, so a member
   * running the new app would never hear it and would sit in a circle nobody
   * else was in any more.
   */
  const heard = new Set();
  for (const { client: older } of earlierClients.values()) {
    if (heard.has(older)) continue;
    heard.add(older);
    older.on("signal", whenSomebodyTellsUsSomething);
  }
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
   * Two places, on purpose, and no longer a fallback.
   *
   * Every suggestion appears beside the words it is about, on the record,
   * which is where somebody reading the record would look for it. The list
   * here is the suggestion box: a page of its own, for somebody who came to
   * see what has been offered rather than to read the record.
   *
   * It used to hide itself whenever the record was on screen, because it was
   * the same page and saying everything twice would have been noise. They are
   * different pages now, so the only question left is whether there is
   * anything in it.
   */
  markSuggestionsOnTheRecord();

  // What was decided before the circle moved, kept as history. See
  // historyFor, and why these are words rather than the entries themselves.
  // The same rule as above for what was set aside before the move.
  const earlier = (historyFor(circle?.cellId)?.suggestions ?? []).filter(
    (item) => isHolder() || item.accepted || item.who === "You",
  );
  for (const item of earlier) list.append(earlierSuggestionCard(item));

  $("suggestions-section").hidden = suggestions.length === 0 && earlier.length === 0;
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
    const entry = suggestionWords(item);
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

  const entry = suggestionWords(item);
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
    const current = wordsOf(record.current);
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
    suggestingAboutOneSection(false);
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

/** The holder has asked to choose somebody else, though someone agreed. */
let choosingAnotherToAgree = false;

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

  if (await carryTheAgreementAcross()) return;

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

/*
 * Whether each member's chain is in good order — checked quietly, shown only
 * when it is not.
 *
 * Nothing is shown for a chain that is fine, and nothing for one this device
 * has not heard about yet, which is ordinary for somebody who has just
 * joined. Two things are worth saying, and both are things somebody's app has
 * actually done, proven by the network rather than suspected by this one:
 * the same identity running in two places at once, and something written
 * that other people's devices have said broke the rules.
 *
 * Shown to the holder only. She is the one who decides who stays in the
 * circle, and a warning about a named person, shown to everybody in it, would
 * be unkind where the cause is innocent — and a fork can be: an old device
 * left switched on after moving to a new one produces exactly the same proof.
 *
 * Remembered for five minutes per person, so the twenty-second re-read does
 * not turn into one network request per member every twenty seconds.
 */
const chainHealth = new Map(); // agent key text -> { status, warrants, at }
const askingAboutChain = new Set();
const CHAIN_HEALTH_FRESH_FOR = 5 * 60 * 1000;

function checkChainHealth(key) {
  const known = chainHealth.get(key);
  if (known && Date.now() - known.at < CHAIN_HEALTH_FRESH_FOR) return;
  if (askingAboutChain.has(key) || !circle) return;

  askingAboutChain.add(key);
  const inCircle = asText(circle.cellId?.[0]);

  orNothingYet(call("chain_health", key, circle.cellId), null)
    .then((health) => {
      askingAboutChain.delete(key);
      // Nothing came back, or the circle on screen changed while asking.
      if (!health || asText(circle?.cellId?.[0]) !== inCircle) return;
      const before = chainHealth.get(key);
      chainHealth.set(key, { ...health, at: Date.now() });
      // Only redraw when there is something new to say.
      if (
        chainNeedsSaying(health) !== Boolean(before && chainNeedsSaying(before))
      ) {
        renderPeople();
      }
    })
    .catch(() => askingAboutChain.delete(key));
}

const chainNeedsSaying = (health) =>
  health.status === "forked" ||
  health.status === "invalid" ||
  Number(health.warrants) > 0;

function chainHealthNote(health) {
  const note = document.createElement("p");
  note.className = "notice";
  note.textContent =
    health.status === "forked"
      ? "This person's identity seems to be in use in two places at once. " +
        "That can happen innocently — an old device still switched on after " +
        "moving to a new one — or it can mean a copy of their app is being " +
        "run somewhere else."
      : "Something this person's app wrote broke this circle's rules, and " +
        "other people's devices have said so. The copy of the app they are " +
        "using may have been changed.";
  return note;
}

/** Name the holder in "… will decide whether to add it", once names are known. */
function sayWhoDecidesSuggestions() {
  $("who-decides-suggestions").textContent =
    members.get(holder)?.name?.trim() || "The person who holds this circle";
}

function renderPeople() {
  // Names have just arrived, and one of them may be the holder's.
  sayWhoDecidesSuggestions();
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
  $("people-empty").hidden = members.size > 0;

  // Only the holder appoints, and only where there is somebody to appoint.
  const amHolder = isHolder();
  // And the invitation to press a name goes quiet once somebody has agreed.
  const somebodyAgreed =
    Boolean(whoAgrees) && whoAgrees.willing !== false && !choosingAnotherToAgree;
  $("appoint-hint").hidden = !amHolder || members.size < 2 || somebodyAgreed;

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
    if (theirs) li.append(howFarTheAskingHasGot(who, key));

    // Only for the holder, only about somebody else, and only when something
    // is actually wrong. See checkChainHealth.
    if (amHolder && key !== asText(me)) {
      checkChainHealth(key);
      const health = chainHealth.get(key);
      if (health && chainNeedsSaying(health)) li.append(chainHealthNote(health));
    }

    // Asking somebody else replaces whoever is asked now, so the person
    // already asked needs no button of their own — pressing another name is
    // the whole of changing your mind.
    /*
     * Only while the role is open.
     *
     * "Ask Ronnie Smythe instead", beside every name, after Dave had already
     * said yes, read as though the role were still up for grabs — and it
     * offered a way to take it off somebody who had just agreed, one press
     * away, on a page she visits for other reasons. Once somebody has agreed,
     * the choice is behind one quiet link under their name, for the day it is
     * really needed.
     */
    const roleIsOpen =
      !whoAgrees || whoAgrees.willing === false || choosingAnotherToAgree;
    if (amHolder && key !== asText(me) && !theirs && roleIsOpen) {
      li.append(askThem(key, who));
    }

    if (amHolder && theirs && whoAgrees.willing !== false && !choosingAnotherToAgree) {
      const actions = document.createElement("div");
      actions.className = "actions";
      const change = document.createElement("button");
      change.type = "button";
      change.className = "linky";
      change.textContent = "Ask somebody else instead";
      change.addEventListener("click", () => {
        choosingAnotherToAgree = true;
        renderPeople();
        announce("Choose who to ask instead, from the names below.");
      });
      actions.append(change);
      li.append(actions);
    }

    // Removing somebody: the ordinary way, or by moving everybody else.
    if (amHolder && key !== asText(me)) {
      li.append(removeThem(key, who));
    }

    list.append(li);
  }

  // Only the holder sees who she has removed, and can let them back.
  if (amHolder) {
    for (const [key, entry] of removedMembers) {
      list.append(someoneRemoved(key, entry));
    }
  }
}

/**
 * What the person asked has said, in a sentence rather than a state.
 *
 * "Asked" and "agreed" are different facts and the holder acts differently on
 * each, so neither is allowed to stand in for the other.
 *
 * Written three times over, because the same fact is a different sentence
 * depending on who is reading it. "Agrees to who joins, along with you" was
 * written for the holder and shown to everybody — so an ordinary member read
 * that they were one of the two deciding, which they are not. That is not a
 * clumsy sentence; it is the screen telling somebody they have a power they
 * do not have.
 *
 * Everybody does see this, deliberately. Who was asked, and when, is what the
 * safeguard now rests on, and a safeguard nobody can see is not one. It just
 * has to say the true thing to each of them.
 */
function howFarTheAskingHasGot(who, theirKey) {
  const said = document.createElement("p");
  said.className = whoAgrees.willing === false ? "notice" : "hint";

  const amHolder = isHolder();
  const aboutMe = theirKey === asText(me);
  // The holder by name where she has given one, for the people who are
  // neither of the two.
  const theHolder = members.get(holder)?.name?.trim() || "the person who holds it";

  if (whoAgrees.willing === true) {
    said.textContent = amHolder
      ? "Agrees to who joins, along with you."
      : aboutMe
        ? `You agree to who joins, along with ${theHolder}.`
        : `Agrees to who joins, along with ${theHolder}.`;
    return said;
  }

  if (whoAgrees.willing === false) {
    said.textContent = aboutMe
      ? "You said you would rather not. Nobody new can join until somebody " +
        "else is asked."
      : "Asked, and would rather not. Nobody new can join until somebody " +
        "else is asked.";
    return said;
  }

  said.textContent = aboutMe
    ? "You have been asked, and have not answered yet. Nobody new can join " +
      "until you do."
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
      choosingAnotherToAgree = false;
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
  removedMembers = new Map();
  for (const r of records) {
    const entry = entryOf(r);
    if (!entry) continue;
    // Latest introduction wins; people correct how they describe themselves.
    const key = asText(authorOf(r));
    // Somebody removed is not in the circle, whatever they wrote before.
    if (gone.has(key)) removedMembers.set(key, entry);
    else members.set(key, entry);
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
      // AddressMistake says which row to look at; let it reach the screen.
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

/*
 * Connect to any earlier version of the rules still installed on this machine.
 *
 * The desktop app puts what it knows in `__HEARTH_EARLIER_APPS__`: one entry
 * per older version that is actually installed, with a token for it. Empty on
 * a first install, and empty in the browser demo, where there has never been
 * an older version to keep.
 *
 * Failing to connect to an old app must never stop the new one starting. The
 * circles in it are safe on disk either way, and saying so is better than a
 * blank screen.
 */
async function connectToEarlierRules() {
  const earlier = globalThis.__HEARTH_EARLIER_APPS__ ?? [];
  const port = globalThis.__HC_LAUNCHER_ENV__?.APP_INTERFACE_PORT;
  if (!earlier.length || !port) return;

  for (const app of earlier) {
    try {
      const older = await AppWebsocket.connect({
        url: new URL(`ws://127.0.0.1:${port}`),
        token: Uint8Array.from(app.token),
      });
      const info = await older.appInfo();
      for (const raw of info.cell_info[ROLE] ?? []) {
        const cell = raw?.value ?? raw?.cloned ?? raw;
        if (!cell?.cell_id) continue;
        earlierClients.set(asText(cell.cell_id[0]), {
          client: older,
          installedAppId: app.installed_app_id,
          info,
        });
      }
    } catch (error) {
      console.error(`Could not reach ${app.installed_app_id}.`, error);
    }
  }
}

/** The circles in every earlier version, marked as being on their way out. */
function circlesUnderEarlierRules() {
  const seen = new Set();
  const out = [];

  for (const { info } of earlierClients.values()) {
    if (!info || seen.has(info)) continue;
    seen.add(info);

    for (const raw of info.cell_info[ROLE] ?? []) {
      const cell = raw?.value ?? raw?.cloned ?? raw;
      if (!cell?.clone_id) continue;
      if (propertiesOf(cell)?.waiting_for) continue;
      if (cell.enabled === false) continue;
      out.push({
        cellId: cell.cell_id,
        name: cell.name ?? "A circle",
        asksTwo: Boolean(propertiesOf(cell)?.requires_second_yes),
        olderRules: true,
      });
    }
  }
  return out;
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
    // A circle she has moved away from stays switched on until everybody has
    // followed, because it is the only place she can still reach them. It is
    // not somewhere she should be able to wander back into.
    .filter((c) => !movingFrom(c.cell_id))
    // What this device calls it wins over the name the cell was made with,
    // which cannot be changed afterwards.
    .map((c) => ({
      cellId: c.cell_id,
      name: labelFor(c.cell_id, c.name || "Circle"),
      // Kept so clearing the label can go back to it. The label overrides
      // this; it does not replace it.
      madeWith: c.name || "Circle",
      // Whether this circle says it asks two people to agree. Part of what
      // the circle is, so it is read off the cell rather than asked for.
      asksTwo: Boolean(propertiesOf(c)?.requires_second_yes),
    }));

  /*
   * And the circles made before the rules changed, which live in the older
   * app beside this one. They are still hers, still readable, and still
   * waiting to be carried across — so they belong on the same list rather
   * than somewhere she has to go looking.
   */
  circles = circles.concat(
    circlesUnderEarlierRules().map((older) => ({
      ...older,
      name: labelFor(older.cellId, older.name),
      madeWith: older.name,
    })),
  );

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

    /*
     * The one exception, and it is not a badge about activity: a circle made
     * before the rules changed. It still opens and still reads; what it
     * cannot do is anything the newer rules added, and it will need carrying
     * across. Saying so on the list is kinder than letting her find out
     * inside. See docs/upgrades.md.
     */
    if (item.olderRules) {
      const note = document.createElement("span");
      note.className = "older-rules";
      note.textContent = "made with an older version";
      button.append(note);
    }
    button.addEventListener("click", () => openCircle(item).catch(problem));
    li.append(button);
    list.append(li);
  }
}

async function openCircle(item) {
  // Whether this circle was made before the rules changed is carried with it:
  // the screen offers to carry it across, and only the holder can.
  circle = { cellId: item.cellId, olderRules: Boolean(item.olderRules) };
  circleAsksTwo = Boolean(item.asksTwo);
  // Coming back to a circle is not writing one.
  justWroteIt = false;
  // Always the record first. Coming back to a circle to read it is the
  // ordinary reason for coming back to one.
  circleShows = "record";

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
  choosingAnotherToAgree = false;
  seconderHere = null;
  theDoorIsHere = false;
  circleAsksTwo = false;
  chainHealth.clear();
  askingAboutChain.clear();
  // Who was removed from this circle means nothing in the next one.
  gone = new Map();
  removedMembers = new Map();
  // Nor do its pictures, or who might take it over.
  forgetMedia();
  succession = null;
  $("taking-over-banner").hidden = true;
  $("succession").hidden = true;
  knocking = [];
  lastWaitingCount = 0;
  stopChiming();
  $("waiting-count").textContent = "";
  document
    .querySelector('.circle-tab[data-page="people"]')
    .classList.remove("asking");
  circleShows = "record";
  justWroteIt = false;
  if (watchingForTheAsking) {
    clearInterval(watchingForTheAsking);
    watchingForTheAsking = null;
  }
  suggestions = [];
  peopleLastSeen = new Set();
  showingSomething = false;
  beenThroughOnce = false;
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

/*
 * Nothing happens until the second press, and the second press is the one
 * that has the circle's name on it.
 */
$("leave-circle").addEventListener("click", () => {
  // "this's circle" is what the obvious version of this line produces on a
  // circle whose record has not arrived yet.
  const name = personName();
  $("really-leave-question").textContent = name
    ? `Are you sure you want to leave ${name}'s circle?`
    : "Are you sure you want to leave this circle?";
  const box = $("really-leave");
  box.showModal();
  $("stay-here").focus();
});

$("stay-here").addEventListener("click", () => {
  $("really-leave").close();
  $("leave-circle").focus();
});

$("leave-for-real").addEventListener("click", async () => {
  $("really-leave").close();
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

/*
 * A new door, for a flooded or leaked one.
 *
 * The limit on knocks is per key, and keys cost nothing to make, so somebody
 * determined can knock from many (docs/hard-questions.md, question 8). The
 * answer is not a cleverer limit but a door that can be left behind: a new
 * seed is a new room, and nobody who had the old address can find it. The
 * same thing a move does to a door, without moving the circle.
 */
$("new-door-address").addEventListener("click", () =>
  whileWorking($("new-door-address"), "Opening a new door…", async () => {
    const oldRoom = roomFor(circle?.cellId);
    const room = {
      holder: asText(me),
      seed: crypto.randomUUID(),
      about: oldRoom?.about ?? "",
    };
    const label = labelFor(circle.cellId, "Circle");
    await call("enter_waiting_room", {
      holder: room.holder,
      network_seed: room.seed,
      name: `${label} — door`,
    });
    rememberRoom(circle.cellId, room);

    // The old one, closed on this device. Nobody is there to answer it now.
    if (oldRoom) {
      const oldCell = (await waitingRoomCells()).find(
        (r) => r.holder === oldRoom.holder && r.seed === oldRoom.seed,
      );
      if (oldCell) {
        await call("leave_circle", oldCell.cellId[0]).catch((error) =>
          console.error("Could not close the old door.", error),
        );
      }
    }

    $("new-door").open = false;
    await loadTheDoor();
    announce(
      "The door has a new address. Send it to anybody you still want to ask to " +
        "join, and make new passes for anybody who needs one.",
    );
  }).catch(problem),
);

wireCopyButton(
  "copy-door-address",
  () => $("door-address-output").textContent,
  "Address copied",
);

/**
 * The whole message, ready to paste into whatever they use.
 *
 * What somebody receiving an address needs is the address *and* what to do
 * with it. Making the holder write that part herself — every time, for every
 * person — is the errand most likely to be done badly or skipped, and the
 * person on the other end is then left holding a long line of characters and
 * no idea what it is for.
 *
 * Deliberately plain text with no links in it. This goes into a text message
 * or a WhatsApp, and it has to survive being read on a phone by somebody who
 * has never heard of any of this.
 */
function anInvitationToSend() {
  /*
   * The name, if there is one, and English either way.
   *
   * personName already prefers what the record says over what was typed when
   * the circle was made, which is the right order: the record is what
   * everybody else reads. But it can be empty, and the first draft of this
   * filled the gap with a description — producing "part of the person this
   * circle is about's Hearth", which is not a sentence anybody would send.
   *
   * "them" throughout rather than a pronoun for the person. Nothing here has
   * been told which one they use, and guessing wrong in a message somebody
   * sends to their family is worse than the small stiffness of "them".
   */
  const name = personName();
  const whose = name ? `${name}'s` : "their";
  const whom = name || "them";
  const address = $("door-address-output").textContent;

  return [
    `I would like you to be part of ${whose} Hearth — a private record of ` +
      `what matters to them and how to look after them.`,
    "",
    "Here is the address of the circle:",
    "",
    address,
    "",
    "To get in:",
    "1. Open Hearth and press \u201CJoin a circle\u201D.",
    "2. Paste or type the address above into the box. Capitals and spaces do not matter.",
    `3. Put in your name and how you are connected to ${whom}.`,
    "4. Press \u201CAsk to join\u201D, then wait.",
    "",
    "I will see you waiting and let you in. It may not be straight away, and " +
      "it will open on its own once I have.",
  ].join("\n");
}

wireCopyButton(
  "copy-invitation-message",
  anInvitationToSend,
  "Message copied. Paste it into an email, a text, or WhatsApp.",
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
/*
 * Making a circle, one page at a time.
 *
 * It was one screen with five boxes and two fieldsets on it, and on a laptop
 * the bottom of it was below the fold. A screenful of boxes is where somebody
 * stops reading and starts guessing, and the guessing matters here: two of
 * those boxes name two different people and one of them is read by the whole
 * circle.
 *
 * Splitting it also buys room to say more, because each page now has room to
 * spare rather than needing every line shortened to fit.
 *
 * The first page is the question everything else depends on. Nothing else is
 * on it.
 */
const CREATE_PAGES = ["create-step-1", "create-step-2", "create-step-3"];

/** What to put the cursor in when each page opens. */
const CREATE_FOCUS = ["about-someone-else", "person-name", "decides-just-me"];
let createPage = 0;

function showCreatePage(which) {
  createPage = which;
  CREATE_PAGES.forEach((id, i) => {
    $(id).hidden = i !== which;
  });
  // Said once, on the page that introduces the thing. Repeating it above the
  // boxes would be three lines somebody has already read.
  $("create-intro").hidden = which !== 0;
  window.scrollTo({ top: 0 });
}

/*
 * A second circle starts empty, and starts as somebody else's.
 *
 * Only the "who decides" answer was being reset, so everything else was
 * whatever the last visit left: the name of the last person, and — the one
 * that actually bites — the answer to "Who is this circle about?".
 *
 * That answer decides whether three of the four questions on the next page
 * exist at all. Look at "Me — this is my own record" once, for any reason,
 * come back, and the form quietly stops asking how you are connected to them
 * and what you call them, for every circle you make after it. Nothing is
 * broken on screen; the questions are simply not there, and the circle is
 * made without the answers.
 *
 * The joining screen has had this since somebody was nearly invited to answer
 * questions about the wrong person. The same reasoning applies here and it
 * never got the same treatment.
 */
function goToCreate() {
  $("about-someone-else").checked = true;
  // Which hides or shows the three questions that depend on that answer.
  updateWhoseCircle();

  for (const id of [
    "person-name",
    "carer-name",
    "carer-relationship",
    "circle-name",
  ]) {
    $(id).value = "";
  }

  // Whether a circle asks two people to agree is one question about the
  // circle being made, not a different kind of circle reached by a different
  // button.
  $("decides-just-me").checked = true;

  showCreatePage(0);
  show("create");
  $("about-someone-else").focus();
}

$("choose-create").addEventListener("click", goToCreate);

/*
 * Forward, but not past an empty box that the last page will need.
 *
 * A required field on a page you have walked away from cannot be pointed at:
 * the browser refuses to report a problem on something it cannot show, so
 * "Create the circle" does nothing at all and says nothing about why. That is
 * a silent dead end, and it has caught this project once already.
 *
 * So each page checks its own before it lets you leave it, while the box is
 * still on screen to be pointed at.
 */
for (const button of document.querySelectorAll(".create-next")) {
  button.addEventListener("click", () => {
    const here = $(CREATE_PAGES[createPage]);
    for (const field of here.querySelectorAll("[required]")) {
      if (!field.checkValidity()) {
        field.reportValidity();
        return;
      }
    }
    showCreatePage(createPage + 1);
    $(CREATE_FOCUS[createPage]).focus();
  });
}

/*
 * One Back, meaning one step back.
 *
 * Two Backs on one screen, one of which quietly throws away what you have
 * typed, is a trap. So this is the only one, and what it does depends on
 * where you are: out to the menu from the first page, and back to the first
 * page from the second.
 */
$("create-back").addEventListener("click", () => {
  if (createPage > 0) {
    showCreatePage(createPage - 1);
    $(CREATE_FOCUS[createPage]).focus();
    return;
  }
  show("choose");
});

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

/*
 * A room's address, in a shape a person can carry.
 *
 * It used to be about two hundred characters of mixed-case base64 on one
 * line, and it failed Ceri's first offline test three ways. With the
 * internet off there was no email to send it by. Written down or typed in,
 * it was too long to get right. And one wrong character anywhere was simply
 * refused, with nothing to say where.
 *
 * So, now:
 *
 * - **About a hundred characters, in short rows**, from a 32-letter alphabet
 *   with no look-alikes: no O beside 0, no I or L beside 1, and capitals do
 *   not matter. An O typed for a 0 is read as the 0 it was meant to be.
 * - **Every row checks itself.** Two characters at the end of each row are
 *   worked out from the rest of it, so a wrong letter, or two letters
 *   swapped, is caught — and the person is told which row to look at.
 * - **A QR code beside it**, so most of the time nobody types anything.
 * - **No name in it.** The old address carried the person's full name in
 *   readable form to everybody it was passed to. Nothing needs it before
 *   somebody is let in, and the invitation they collect then carries it.
 *
 * What goes in: the holder's key and the room's seed, which are exactly what
 * the room is built from. It cannot honestly be shorter — the key alone is
 * thirty-nine bytes — without a way to look a short code up, which is
 * written down in docs/to-a-product.md.
 *
 * Old addresses are still read, so none already sent stops working.
 */
const ADDRESS_ALPHABET = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const ADDRESS_VERSION = 1;
const ADDRESS_ROW = 16;

function bytesToCode(bytes) {
  let out = "";
  let bits = 0;
  let value = 0;
  for (const byte of bytes) {
    value = (value << 8) | byte;
    bits += 8;
    while (bits >= 5) {
      out += ADDRESS_ALPHABET[(value >>> (bits - 5)) & 31];
      bits -= 5;
    }
  }
  if (bits > 0) out += ADDRESS_ALPHABET[(value << (5 - bits)) & 31];
  return out;
}

function codeToBytes(code, length) {
  const bytes = [];
  let bits = 0;
  let value = 0;
  for (const char of code) {
    value = ((value << 5) | ADDRESS_ALPHABET.indexOf(char)) & 0xffff;
    bits += 5;
    if (bits >= 8) {
      bytes.push((value >>> (bits - 8)) & 255);
      bits -= 8;
    }
  }
  return Uint8Array.from(bytes.slice(0, length));
}

/*
 * Two check characters for one row.
 *
 * Every position is weighted differently and the total is taken modulo 1021,
 * a prime just under 32 × 32. So changing any one character always changes
 * the total, and so does swapping two neighbours. The row's own number is
 * part of it too, so two whole rows written down in the wrong order are
 * caught as well.
 */
function rowCheck(row, rowNumber) {
  let total = 17 * (rowNumber + 1);
  [...row].forEach((char, i) => {
    total += (i + 1) * ADDRESS_ALPHABET.indexOf(char);
  });
  const c = total % 1021;
  return ADDRESS_ALPHABET[c >> 5] + ADDRESS_ALPHABET[c & 31];
}

const uuidToBytes = (uuid) =>
  Uint8Array.from(uuid.replace(/-/g, "").match(/../g).map((h) => parseInt(h, 16)));

const bytesToUuid = (bytes) => {
  const hex = [...bytes].map((b) => b.toString(16).padStart(2, "0")).join("");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
};

const isUuid = (text) => /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(text);

/** A mistake the reader of an address can be pointed at. */
class AddressMistake extends Error {
  constructor(row, rows) {
    super(
      rows > 1
        ? `Row ${row} of the address has a mistake in it. Check that row letter ` +
            `by letter against the one you were given — the rows before it are fine.`
        : "The address has a mistake in it. Check it letter by letter.",
    );
    this.row = row;
  }
}

function roomToAddress(room) {
  // A seed this app did not make itself cannot be packed, so it keeps the
  // old shape rather than being turned into something that will not work.
  if (!isUuid(room.seed)) {
    return btoa(JSON.stringify({ door: room.holder, seed: room.seed }));
  }

  const key = decodeHashFromBase64(room.holder);
  const packed = new Uint8Array(1 + key.length + 16);
  packed[0] = ADDRESS_VERSION;
  packed.set(key, 1);
  packed.set(uuidToBytes(room.seed), 1 + key.length);

  const code = bytesToCode(packed);
  const rows = [];
  for (let i = 0; i < code.length; i += ADDRESS_ROW) {
    const row = code.slice(i, i + ADDRESS_ROW);
    rows.push(row + rowCheck(row, rows.length));
  }

  // Groups of four within a row, so a finger or an eye can keep its place.
  return [
    "HEARTH",
    ...rows.map((row) => row.match(/.{1,4}/g).join(" ")),
  ].join("\n");
}

function addressToRoom(text) {
  if (/^\s*hearth/i.test(text)) {
    const code = text
      .toUpperCase()
      .replace(/^\s*HEARTH/, "")
      .replace(/[\s-]/g, "")
      .replace(/O/g, "0")
      .replace(/[IL]/g, "1");

    // 1 + 39 + 16 bytes, and the two check characters on every row.
    const dataLength = Math.ceil(((1 + 39 + 16) * 8) / 5);
    const rowCount = Math.ceil(dataLength / ADDRESS_ROW);
    const expected = dataLength + rowCount * 2;

    const unreadable = [...code].findIndex((c) => !ADDRESS_ALPHABET.includes(c));
    let data = "";
    for (let r = 0; r < rowCount; r++) {
      const start = r * (ADDRESS_ROW + 2);
      const width = Math.min(ADDRESS_ROW, dataLength - r * ADDRESS_ROW);
      const row = code.slice(start, start + width);
      const check = code.slice(start + width, start + width + 2);
      const broken =
        (unreadable >= 0 && unreadable < start + width + 2) ||
        row.length !== width ||
        check !== rowCheck(row, r);
      if (broken) throw new AddressMistake(r + 1, rowCount);
      data += row;
    }
    if (code.length !== expected) {
      throw new Error(
        code.length > expected
          ? "The address has something extra on the end. Check the last row."
          : "The address is missing something at the end. Check the last row.",
      );
    }

    const packed = codeToBytes(data, 1 + 39 + 16);
    if (packed[0] !== ADDRESS_VERSION) {
      throw new Error("This address was made by a newer Hearth. Update Hearth and try again.");
    }
    return {
      holder: encodeHashToBase64(packed.slice(1, 40)),
      seed: bytesToUuid(packed.slice(40, 56)),
      about: "",
    };
  }

  const parsed = JSON.parse(atob(text.trim()));
  if (typeof parsed?.door !== "string" || typeof parsed?.seed !== "string") {
    throw new Error("not a waiting room address");
  }
  return { holder: parsed.door, seed: parsed.seed, about: parsed.about ?? "" };
}

/*
 * The address as a picture, for a camera.
 *
 * Drawn from the same text, so scanning it and typing it arrive at exactly
 * the same place. Nothing leaves the device to draw it.
 */
function drawAddressCode(canvas, address) {
  QRCode.toCanvas(canvas, address, {
    errorCorrectionLevel: "M",
    margin: 2,
    width: 240,
    color: { dark: "#1b1b1b", light: "#ffffff" },
  }).catch((error) => {
    console.error("Could not draw the address as a code.", error);
    canvas.hidden = true;
  });
}

/*
 * Reading a code with the camera, on the joining screen.
 *
 * The camera is only switched on when somebody presses the button, and off
 * again the moment a code is read, the button is pressed again, or they
 * leave the screen. Every frame is read on this device; nothing is sent
 * anywhere.
 */
let scanning = null; // { stream, frame }

function stopScanning() {
  if (!scanning) return;
  cancelAnimationFrame(scanning.frame);
  for (const track of scanning.stream.getTracks()) track.stop();
  scanning = null;
  $("scan-video").srcObject = null;
  $("scan-area").hidden = true;
  $("scan-code").textContent = "Scan a code with the camera";
}

async function startScanning() {
  let stream;
  try {
    stream = await navigator.mediaDevices.getUserMedia({
      video: { facingMode: "environment" },
      audio: false,
    });
  } catch (error) {
    console.error(error);
    announce(
      "The camera could not be opened. If this computer has no camera, type " +
        "the address instead — capitals and spaces do not matter.",
    );
    return;
  }

  const video = $("scan-video");
  video.srcObject = stream;
  await video.play();
  $("scan-area").hidden = false;
  $("scan-code").textContent = "Stop the camera";

  const canvas = document.createElement("canvas");
  const context = canvas.getContext("2d", { willReadFrequently: true });
  scanning = { stream, frame: 0 };

  const look = () => {
    if (!scanning) return;
    if (video.readyState === video.HAVE_ENOUGH_DATA) {
      canvas.width = video.videoWidth;
      canvas.height = video.videoHeight;
      context.drawImage(video, 0, 0, canvas.width, canvas.height);
      const image = context.getImageData(0, 0, canvas.width, canvas.height);
      const found = jsQR(image.data, image.width, image.height);
      if (found?.data && looksLikeARoomAddress(found.data)) {
        stopScanning();
        $("invitation-in").value = found.data;
        $("invitation-in").dispatchEvent(new Event("input"));
        announce("Address read. Now put in your name.");
        $("joiner-name").focus();
        return;
      }
    }
    scanning.frame = requestAnimationFrame(look);
  };
  scanning.frame = requestAnimationFrame(look);
}

$("scan-code").addEventListener("click", () =>
  scanning ? stopScanning() : startScanning(),
);

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
// Passes: the outer ring
// ---------------------------------------------------------------------------
//
// A pass is a Holochain capability grant made in the circle's door, for one
// function that reads chosen sections. The reader enters the same door and
// presents it; the holder's device answers. See docs/outer-ring.md.
//
// What travels is everything the reader's app needs to find that door and
// knock on the right function: the holder's key, the door's seed, and the
// secret. Written the same way as an address — short rows that check
// themselves — so a typing mistake is pointed at rather than just refused.
// It starts "PASS" rather than "HEARTH", so neither screen mistakes one for
// the other.

const PASS_VERSION = 1;
const PASS_BYTES = 1 + 39 + 16 + 64;

/** The three sections somebody meeting her for the first time needs most. */
const PASS_SECTIONS_TICKED = [
  "HowToCommunicateWithMe",
  "PleaseDoAndPleaseDoNot",
  "HowToSupportMe",
];

function passToText(room, secret) {
  const key = decodeHashFromBase64(room.holder);
  const packed = new Uint8Array(PASS_BYTES);
  packed[0] = PASS_VERSION;
  packed.set(key, 1);
  packed.set(uuidToBytes(room.seed), 40);
  packed.set(secret, 56);

  const code = bytesToCode(packed);
  const rows = [];
  for (let i = 0; i < code.length; i += ADDRESS_ROW) {
    const row = code.slice(i, i + ADDRESS_ROW);
    rows.push(row + rowCheck(row, rows.length));
  }
  return ["PASS", ...rows.map((row) => row.match(/.{1,4}/g).join(" "))].join("\n");
}

const looksLikeAPass = (text) => /^\s*pass/i.test(text);

function textToPass(text) {
  if (!looksLikeAPass(text)) {
    throw new Error(
      "That does not look like a pass. A pass starts with the word PASS. " +
        "If it starts with HEARTH, it is the address of a circle — use Join a circle instead.",
    );
  }
  const code = text
    .toUpperCase()
    .replace(/^\s*PASS/, "")
    .replace(/[\s-]/g, "")
    .replace(/O/g, "0")
    .replace(/[IL]/g, "1");

  const dataLength = Math.ceil((PASS_BYTES * 8) / 5);
  const rowCount = Math.ceil(dataLength / ADDRESS_ROW);
  let data = "";
  for (let r = 0; r < rowCount; r++) {
    const start = r * (ADDRESS_ROW + 2);
    const width = Math.min(ADDRESS_ROW, dataLength - r * ADDRESS_ROW);
    const row = code.slice(start, start + width);
    const check = code.slice(start + width, start + width + 2);
    const unreadable = [...row + check].some((c) => !ADDRESS_ALPHABET.includes(c));
    if (unreadable || row.length !== width || check !== rowCheck(row, r)) {
      throw new Error(
        `Row ${r + 1} of the pass has a mistake in it. Check that row letter by ` +
          `letter — the rows before it are fine.`,
      );
    }
    data += row;
  }
  if (code.length !== dataLength + rowCount * 2) {
    throw new Error("The pass is the wrong length. Check the last row.");
  }

  const packed = codeToBytes(data, PASS_BYTES);
  if (packed[0] !== PASS_VERSION) {
    throw new Error("This pass was made by a newer Hearth. Update Hearth and try again.");
  }
  return {
    holder: encodeHashToBase64(packed.slice(1, 40)),
    seed: bytesToUuid(packed.slice(40, 56)),
    secret: packed.slice(56),
  };
}

/** When a pass should stop, in Holochain's microseconds, or null for never. */
function passRunsOutAt(choice) {
  if (choice === "stopped") return null;
  const end = new Date();
  if (choice === "today") {
    end.setHours(23, 59, 59, 0);
  } else {
    end.setDate(end.getDate() + 7);
  }
  return end.getTime() * 1000;
}

const sectionName = (section) => FIELD_LABELS[section]?.[1] ?? section;

const whenText = (micros) =>
  new Date(micros / 1000).toLocaleString(undefined, {
    weekday: "short",
    day: "numeric",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
  });

/*
 * When a pass was used, remembered on this device only.
 *
 * The holder's device is the only one that knows: the answer is sent from it,
 * and nothing is written to the circle. Letting the whole circle see "Ward 7
 * read this" would need a new kind of entry in the rules, which is a change to
 * the frozen file and is written down in docs/outer-ring.md as the next step.
 */
const PASS_READS_KEY = "hearth:pass-reads";

function passReads() {
  try {
    return JSON.parse(localStorage.getItem(PASS_READS_KEY) ?? "[]");
  } catch {
    return [];
  }
}

function rememberPassRead(door, payload) {
  const reads = passReads();
  // Reading again within a minute is one reading, not another line: somebody
  // holding a pass could otherwise read in a loop and push the real history
  // off the end of this list. Found in audit, 23 September 2026.
  const last = reads[0];
  if (
    last &&
    last.door === door &&
    last.for_whom === payload.for_whom &&
    Math.abs(payload.at - last.at) < 60_000_000
  ) {
    return false;
  }
  reads.unshift({
    door,
    for_whom: payload.for_whom,
    sections: payload.sections,
    at: payload.at,
  });
  try {
    localStorage.setItem(PASS_READS_KEY, JSON.stringify(reads.slice(0, 200)));
  } catch {
    // The read still happened; only the note of it is lost.
  }
  return true;
}

function buildPassSectionChoices() {
  const box = $("pass-sections");
  if (box.querySelector("input")) return;
  for (const [section, [, label]] of Object.entries(FIELD_LABELS)) {
    const row = document.createElement("label");
    row.className = "choice-row";
    const tick = document.createElement("input");
    tick.type = "checkbox";
    tick.value = section;
    tick.checked = PASS_SECTIONS_TICKED.includes(section);
    row.append(tick, ` ${label}`);
    box.append(row);
  }
}

async function loadPasses() {
  buildPassSectionChoices();
  const door = currentRoomCell;
  if (!door) {
    $("pass-list").replaceChildren();
    return;
  }

  const passes = await orNothingYet(call("passes_here", null, door), []);
  $("no-passes").hidden = passes.length > 0;
  $("pass-list").replaceChildren(
    ...passes.map((pass) => {
      const item = document.createElement("li");
      const what = pass.terms.sections.map(sectionName).join(", ");
      const lasts = pass.terms.until
        ? pass.run_out
          ? `ran out ${whenText(pass.terms.until)}`
          : `until ${whenText(pass.terms.until)}`
        : "until you stop it";
      const words = document.createElement("span");
      words.textContent = `${pass.terms.for_whom} — ${what}, ${lasts}. `;
      if (pass.run_out) item.classList.add("run-out");

      const stop = document.createElement("button");
      stop.type = "button";
      stop.className = "secondary";
      stop.textContent = pass.run_out ? "Take it off the list" : "Stop it";
      stop.addEventListener("click", () =>
        whileWorking(stop, "Stopping…", async () => {
          await call("stop_a_pass", pass.grant, door);
          announce(`The pass for ${pass.terms.for_whom} has stopped working.`);
          await loadPasses();
        }).catch(problem),
      );
      item.append(words, stop);
      return item;
    }),
  );

  const doorText = asText(door[0]);
  const reads = passReads().filter((r) => r.door === doorText);
  $("no-pass-reads").hidden = reads.length > 0;
  $("pass-reads").replaceChildren(
    ...reads.map((read) => {
      const item = document.createElement("li");
      item.textContent =
        `${whenText(read.at)}: ${read.for_whom} read ` +
        `${read.sections.map(sectionName).join(", ")}.`;
      return item;
    }),
  );
}

$("pass-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  const door = currentRoomCell;
  const room = roomFor(circle?.cellId);
  if (!door || !room) return;

  const forWhom = $("pass-for").value.trim();
  const sections = [...$("pass-sections").querySelectorAll("input:checked")].map(
    (tick) => tick.value,
  );
  if (!sections.length) {
    announce("Tick at least one part of the record for them to read.");
    return;
  }
  const lasts = document.querySelector('input[name="pass-lasts"]:checked')?.value;

  await whileWorking($("make-pass"), "Making the pass…", async () => {
    const made = await call(
      "make_a_pass",
      {
        circle: asText(circle.cellId[0]),
        sections,
        for_whom: forWhom,
        until: passRunsOutAt(lasts),
      },
      door,
    );
    const text = passToText(room, made.secret);
    $("pass-made-for").textContent = forWhom;
    $("pass-output").textContent = text;
    drawAddressCode($("pass-qr"), text);
    $("pass-made").hidden = false;
    $("pass-for").value = "";
    announce(`Pass made for ${forWhom}.`);
    await loadPasses();
  }).catch(problem);
});

wireCopyButton("copy-pass", () => $("pass-output").textContent, "Pass copied");

// The reader's side.

let passInHand = null;

async function readWithThePass() {
  $("pass-trouble").hidden = true;
  const pass = textToPass($("pass-in").value);
  passInHand = pass;
  const door = await cellForRoom(pass, "A pass");

  /*
   * Asked again for a couple of minutes if their device cannot be found yet.
   *
   * Entering the door is instant; finding who else is in it is not. A device
   * that has just arrived usually takes a minute or two to find anybody (see
   * docs/latency.md), so the first ask nearly always failed and said the other
   * device might be switched off — while it sat on the same desk. Found on two
   * machines, 23 September 2026. Any other answer, such as a pass that has
   * been stopped, is final and shown at once.
   */
  const button = $("read-pass");
  let words;
  for (let attempt = 1; ; attempt++) {
    try {
      words = await call(
        "ask_with_a_pass",
        { holder: pass.holder, secret: pass.secret },
        door,
      );
      break;
    } catch (error) {
      const notFoundYet = /could not be reached yet/.test(String(error?.message ?? error));
      if (!notFoundYet || attempt >= 8) throw error;
      button.textContent = "Finding their device… this can take a minute or two";
      announce("Finding their device. The first time can take a minute or two.");
      await new Promise((resolve) => setTimeout(resolve, 15000));
    }
  }

  $("pass-words-name").textContent = words.name
    ? `About ${words.name}`
    : "What they chose to show you";
  $("pass-words-list").replaceChildren(
    ...words.sections.flatMap(({ section, words: text }) => {
      const title = document.createElement("dt");
      title.textContent = sectionName(section);
      const body = document.createElement("dd");
      body.textContent = text.trim() || "Nothing written here yet.";
      return [title, body];
    }),
  );
  $("pass-reader-form").hidden = true;
  $("pass-words").hidden = false;
  $("pass-words-name").focus?.();
}

/*
 * What went wrong, in the zome's own words. Those arrive wrapped in the
 * runtime's: WasmError { ..., error: Guest("the words") }. Kept on screen
 * rather than announced, because an announcement fades and this is the thing
 * they need to act on.
 */
function passTrouble(error) {
  console.error(error);
  const said = String(error?.message ?? error);
  const guest = said.match(/Guest\("(.*?)"\)/s);
  $("pass-trouble").textContent = guest ? guest[1] : said;
  $("pass-trouble").hidden = false;
}

function aFreshPassReading() {
  $("pass-trouble").hidden = true;
  passInHand = null;
  $("pass-in").value = "";
  $("pass-reader-form").hidden = false;
  $("pass-words").hidden = true;
  $("pass-words-list").replaceChildren();
}

$("choose-pass").addEventListener("click", () => {
  aFreshPassReading();
  show("pass-reader");
  $("pass-in").focus();
});

$("pass-reader-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    await whileWorking($("read-pass"), "Asking their device…", readWithThePass);
  } catch (error) {
    // A pass that has stopped, or a device that is off, is not a crash: say
    // so on this screen, where they can try again.
    passTrouble(error);
  }
});

$("read-pass-again").addEventListener("click", async () => {
  if (!passInHand) return;
  try {
    await whileWorking($("read-pass-again"), "Asking again…", readWithThePass);
  } catch (error) {
    passTrouble(error);
  }
});

/*
 * The camera, for a pass on somebody else's screen. The same rules as the
 * address scanner: on only when asked, off the moment a pass is read.
 */
let scanningPass = null;

function stopScanningPass() {
  if (!scanningPass) return;
  cancelAnimationFrame(scanningPass.frame);
  for (const track of scanningPass.stream.getTracks()) track.stop();
  scanningPass = null;
  $("scan-pass-video").srcObject = null;
  $("scan-pass-area").hidden = true;
  $("scan-pass").textContent = "Scan a code with the camera";
}

$("scan-pass").addEventListener("click", async () => {
  if (scanningPass) return stopScanningPass();
  let stream;
  try {
    stream = await navigator.mediaDevices.getUserMedia({
      video: { facingMode: "environment" },
      audio: false,
    });
  } catch (error) {
    console.error(error);
    announce("The camera could not be opened. Paste the letters of the pass instead.");
    return;
  }
  const video = $("scan-pass-video");
  video.srcObject = stream;
  await video.play();
  $("scan-pass-area").hidden = false;
  $("scan-pass").textContent = "Stop the camera";

  const canvas = document.createElement("canvas");
  const context = canvas.getContext("2d", { willReadFrequently: true });
  scanningPass = { stream, frame: 0 };
  const look = () => {
    if (!scanningPass) return;
    if (video.readyState === video.HAVE_ENOUGH_DATA) {
      canvas.width = video.videoWidth;
      canvas.height = video.videoHeight;
      context.drawImage(video, 0, 0, canvas.width, canvas.height);
      const image = context.getImageData(0, 0, canvas.width, canvas.height);
      const found = jsQR(image.data, image.width, image.height);
      if (found?.data && looksLikeAPass(found.data)) {
        stopScanningPass();
        $("pass-in").value = found.data;
        $("pass-reader-form").requestSubmit();
        return;
      }
    }
    scanningPass.frame = requestAnimationFrame(look);
  };
  scanningPass.frame = requestAnimationFrame(look);
});

// ---------------------------------------------------------------------------
// The holder's side: who is at the door
// ---------------------------------------------------------------------------

let knocking = [];

async function loadTheDoor() {
  const room = isHolder() ? roomFor(circle?.cellId) : null;

  if (!room) {
    theDoorIsHere = false;
    showCircleMode();
    currentRoomCell = null;
    knocking = [];
    $("knock-list").replaceChildren();
    return;
  }

  theDoorIsHere = true;
  showCircleMode();
  const address = roomToAddress(room);
  if ($("door-address-output").textContent !== address) {
    $("door-address-output").textContent = address;
    drawAddressCode($("door-address-qr"), address);
  }

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
  loadPasses().catch((error) => console.error("Could not list the passes.", error));

  knocking =await orNothingYet(call("get_knocks", null, roomCell), []);

  // Somebody who has already been answered is not still at the door.
  const waiting = knocking.filter((k) => !k.answered);

  $("nobody-knocking").hidden = waiting.length > 0;
  $("nobody-knocking").textContent =
    "Nobody is waiting. Give somebody the address below and they can ask from " +
    "their own Hearth.";

  const list = $("knock-list");
  list.replaceChildren();
  for (const item of waiting) {
    list.append(knockCard(item, roomCell));
  }

  sayWhoIsWaiting();
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

  /*
   * Already put forward, and still sitting in the queue.
   *
   * Where a circle asks two people, "Let them in" does not let anybody in: it
   * writes a proposal and waits on the second agreement. Nothing marked the
   * knock as dealt with, so the person stayed in "People asking to join" with
   * a button under them — and pressing it again wrote a second proposal, for
   * the same person, which had to be agreed to all over again.
   *
   * Walked, and then confirmed by asking all three conductors: two proposals,
   * both agreed, both ready, for one arrival.
   *
   * So the card says where they have got to instead of offering the press
   * again. Nothing is refused here — she can still put them aside — but there
   * is no longer a button whose only effect is to make more work.
   */
  const putForward = pending.find((p) => p.invitee === item.who);
  if (putForward) {
    const where = document.createElement("p");
    where.className = "outcome";
    const second =
      members.get(seconderHere)?.name?.trim() || "the second person";
    where.textContent = putForward.agreed
      ? `Agreed by both of you. ${item.name} is being let in now.`
      : `Put forward. Waiting for ${second} to agree.`;
    li.append(where);
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

// ---------------------------------------------------------------------------
// Somebody is waiting at the door
// ---------------------------------------------------------------------------
//
// The one thing in this app where another person is on the other end of the
// wait. Everything else can be found when somebody next looks; this cannot,
// because until she looks, they are standing outside.
//
// Three things say so, in increasing order of rudeness: a number on the tab,
// which is visible from the other three pages; the tab flashing, which is
// visible from across the room; and a sound, which is the only one that
// reaches somebody not looking at the screen at all. The first two stop when
// she opens the tab. The number stays, because it is a fact rather than an
// alarm.

/** How many people are waiting and have not been put aside. */
function howManyWaiting() {
  return knocking.filter(
    (k) => !k.answered && !ignored.has(asText(k.knock)) && !ignored.has(k.who),
  ).length;
}

const SOUND_OFF = "hearth:knock-sound-off";
const soundIsOff = () => {
  try {
    return localStorage.getItem(SOUND_OFF) === "yes";
  } catch {
    return false;
  }
};

let chimeTimer = null;
let chimesLeft = 0;
let lastWaitingCount = 0;

/*
 * A chime, made rather than fetched.
 *
 * No audio file, which keeps the content security policy as tight as it is
 * and the .webhapp as small as it is. Two soft notes a fifth apart, short and
 * quiet — a doorbell, not an alarm, for a record about somebody who may be
 * asleep in the next room.
 */
function chime() {
  try {
    const Sound = window.AudioContext || window.webkitAudioContext;
    if (!Sound) return;
    const audio = new Sound();
    // Browsers refuse to make a sound until somebody has pressed something.
    // She has — she is in a circle — but a refused promise must not throw.
    audio.resume?.().catch(() => {});

    for (const [at, hz] of [
      [0, 587.33],
      [0.18, 880],
    ]) {
      const note = audio.createOscillator();
      const level = audio.createGain();
      note.type = "sine";
      note.frequency.value = hz;
      note.connect(level);
      level.connect(audio.destination);

      const start = audio.currentTime + at;
      level.gain.setValueAtTime(0, start);
      level.gain.linearRampToValueAtTime(0.08, start + 0.02);
      level.gain.exponentialRampToValueAtTime(0.0001, start + 0.5);
      note.start(start);
      note.stop(start + 0.55);
    }

    setTimeout(() => audio.close?.(), 1500);
  } catch {
    // A machine with no sound is not a machine with a problem.
  }
}

function stopChiming() {
  if (chimeTimer) clearInterval(chimeTimer);
  chimeTimer = null;
  chimesLeft = 0;
}

/*
 * Say that somebody is waiting, as loudly as the situation deserves.
 *
 * The sound repeats every thirty seconds and then gives up. An app that
 * chimes all night because somebody knocked at nine is an app whose sound
 * gets switched off for good, and then nobody is told about anything.
 * Somebody new arriving starts it again.
 */
function sayWhoIsWaiting() {
  const waiting = howManyWaiting();
  const looking = circleShows === "people";

  $("waiting-count").textContent = waiting ? ` (${waiting})` : "";

  const tab = document.querySelector('.circle-tab[data-page="people"]');
  tab.classList.toggle("asking", waiting > 0 && !looking);

  const moreThanBefore = waiting > lastWaitingCount;
  lastWaitingCount = waiting;

  if (!waiting || looking || soundIsOff()) {
    stopChiming();
    return;
  }

  if (moreThanBefore) {
    chimesLeft = 10;
    chime();
  }
  if (!chimeTimer && chimesLeft > 0) {
    chimeTimer = setInterval(() => {
      if (chimesLeft <= 0 || circleShows === "people" || soundIsOff()) {
        stopChiming();
        return;
      }
      chimesLeft -= 1;
      chime();
    }, 30_000);
  }
}

function sayWhetherTheSoundIsOn() {
  $("knock-sound").textContent = soundIsOff()
    ? "Turn the sound back on"
    : "Turn the sound off";
}

$("knock-sound").addEventListener("click", () => {
  const off = !soundIsOff();
  try {
    localStorage.setItem(SOUND_OFF, off ? "yes" : "no");
  } catch {
    // A device that will not remember it still obeys it for this visit.
  }
  sayWhetherTheSoundIsOn();
  announce(off ? "Sound off. The tab will still say." : "Sound on.");
  sayWhoIsWaiting();
});

sayWhetherTheSoundIsOn();

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

  // Somebody put aside is somebody no longer waiting, so the number and the
  // flashing go with them.
  sayWhoIsWaiting();
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
  // Somebody removed before, let in again: record that they are back, or
  // every copy of Hearth would go on hiding them — including their own, which
  // would take the circle straight off their device again.
  if (gone.has(item.who)) {
    await call("decide_departure", { who: item.who, removed: false }, circle.cellId);
    gone.delete(item.who);
  }

  if (seconderHere) {
    // Drawn from a list that was read a moment ago, so check again here. A
    // second proposal for one person is a second agreement for somebody to
    // give, for nothing.
    if (pending.some((p) => p.invitee === item.who)) {
      announce("Already put forward. Nothing more to do until they agree.");
      await loadCircle();
      return;
    }

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

  // One answer per knock, whatever the list holds. Two proposals naming the
  // same person would otherwise leave two invitations at one door.
  const answered = new Set();

  for (const person of pending) {
    if (!person.agreed || !person.invitation) continue;
    const theirKnock = knocking.find((k) => k.who === person.invitee && !k.answered);
    if (!theirKnock) continue;
    if (answered.has(asText(theirKnock.knock))) continue;
    answered.add(asText(theirKnock.knock));

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
   * Already here — but switched on, or switched off?
   *
   * The answer stays in the room for good and this device stays in the room
   * with it, so the same invitation is found on every look. Returning early
   * on any match is what stops that becoming "Tried to create a cell with an
   * existing id" every twenty seconds.
   *
   * But a circle somebody has taken off their device is still here too, only
   * disabled — and it matched, so somebody who left, knocked again and was
   * let in got nothing at all. No error, no circle, no way to tell why. Found
   * by asking whether coming back actually worked before writing on the
   * screen that it does.
   */
  const here = await circleAlreadyHere(bundle);
  if (here?.enabled) return;
  if (here) {
    // Intact, with everything that was in it. It was only switched off.
    const back = await call("rejoin_circle", here.cellId[0]);
    circle = { cellId: back.cell_id };
    holder = bundle.founder;
    alwaysAWayBack();
    show("circle");
    announce("You are back in.");
    await loadCircles();
    await loadCircle();
    return;
  }

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
  // "You are in Margaret Smythe." is not a sentence anybody should be shown.
  // Whose circle, then — and "their circle" when all there is to go on is
  // the fallback label.
  announce(
    label === "Their circle"
      ? "You are in their circle."
      : `You are in ${label}'s circle.`,
  );
  await loadCircle();
}

// ---------------------------------------------------------------------------
// Moving a circle: removing somebody by carrying everybody else across
// ---------------------------------------------------------------------------
//
// Nothing can take a person out of a circle they are already in. There is no
// operator to reach across and do it, and the network cannot tell one reader
// from another. What can be done is a new circle without them.
//
// That used to mean everybody joining again by hand. Here it means one press:
// the holder's app makes the new circle, copies the record, and hands every
// remaining member an invitation through the old circle, where she can still
// reach them. Their apps join, carry the name across, and switch the old one
// off. Nobody types anything, and the person removed is sent nothing at all.
//
// Coordinator and interface only. The frozen integrity zome is untouched, so
// this is also the machinery a future version will need to carry real circles
// onto new rules. See docs/how-it-works.md, "When re-forming is the right
// answer".

const decodeCellId = ([dna, agent]) => [
  decodeHashFromBase64(dna),
  decodeHashFromBase64(agent),
];
const encodeCellId = (cellId) => [asText(cellId[0]), asText(cellId[1])];

const historyKey = (cellId) => `hearth:history:${asText(cellId?.[0])}`;
const movingKey = (cellId) => `hearth:moving:${asText(cellId?.[0])}`;
const movedNoteKey = (cellId) => `hearth:moved-note:${asText(cellId?.[0])}`;
const carryAgreementKey = (cellId) => `hearth:carry-agreement:${asText(cellId?.[0])}`;

function readStored(key) {
  try {
    const raw = localStorage.getItem(key);
    return raw ? JSON.parse(raw) : null;
  } catch {
    return null;
  }
}

function store(key, value) {
  try {
    if (value === null) localStorage.removeItem(key);
    else localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // A convenience, not a rule. The move itself does not depend on it.
  }
}

/** Whether this device is part-way through moving everybody out of a circle. */
const movingFrom = (cellId) => Boolean(readStored(movingKey(cellId)));

/*
 * What happened before the move, kept as history.
 *
 * Acknowledgements and suggestions are signed by the people who wrote them, on
 * their own chains, in the old circle. A new circle cannot hold them as
 * theirs — only they could sign them again, and a professional who read the
 * record once is not going to be asked to read it twice because somebody else
 * was removed.
 *
 * So each device writes down what it could see, in words, at the moment it
 * moves: who read it and what they said they were, what was offered and what
 * became of it. A record of what happened rather than the signed thing itself,
 * and shown as exactly that. The originals are still in the old circle, which
 * is switched off on this device and not deleted.
 */
const historyFor = (cellId) => (cellId ? readStored(historyKey(cellId)) : null);

async function namesIn(cellId) {
  const records = await orNothingYet(call("get_members", null, cellId), []);
  const names = new Map();
  for (const r of records) {
    const entry = entryOf(r);
    if (entry) names.set(asText(authorOf(r)), entry);
  }
  return names;
}

function nameFrom(names, key) {
  if (key === asText(me)) return "You";
  const who = names.get(key);
  if (!who?.name?.trim()) return "Someone who was in the circle";
  return who.relationship?.trim() ? `${who.name} (${who.relationship})` : who.name;
}

const writtenAt = (record) => {
  const raw = record?.signed_action?.hashed?.content?.header?.timestamp;
  return raw === undefined || raw === null ? null : Number(raw) / 1000;
};

/**
 * Everything worth carrying out of a circle, read before it is switched off.
 *
 * Returns the history to keep, and this person's own suggestions that nobody
 * has decided on yet. Those are theirs to sign again, so they are offered
 * again in the new circle rather than frozen as history nobody can act on.
 */
async function historyOf(cellId, names) {
  const readers = [];
  const originals = await orNothingYet(call("get_circle_about_me", null, cellId), []);
  if (originals[0]) {
    const current = await orNothingYet(
      call("get_current_about_me", originals[0], cellId),
      null,
    );
    if (hasBeenWritten(wordsOf(current))) {
      const acks = await orNothingYet(
        call("get_acknowledgements", current.record.signed_action.hashed.hash, cellId),
        [],
      );
      for (const item of acks) {
        readers.push({
          who: nameFrom(names, asText(authorOf(item.record))),
          // Kept as words, because this is written down before the circle moves
          // and read back afterwards, when the old circle's keys are gone.
          role: item.locked_out ? "not readable on this device" : item.role,
          when: writtenAt(item.record),
        });
      }
    }
  }

  const suggestions = [];
  const unfinished = [];
  for (const item of await orNothingYet(call("get_suggestions", null, cellId), [])) {
    const entry = suggestionWords(item);
    if (!entry) continue;
    const author = asText(authorOf(item.suggestion));
    const outcome = entryOf(item.outcome);

    if (!outcome) {
      if (author === asText(me)) {
        unfinished.push({ field: entry.field, text: entry.text, because: entry.because ?? "" });
      }
      continue;
    }

    const [, label] = FIELD_LABELS[entry.field] ?? [null, entry.field];
    suggestions.push({
      who: nameFrom(names, author),
      label,
      text: entry.text,
      because: entry.because ?? "",
      accepted: Boolean(outcome.accepted),
      when: writtenAt(item.suggestion),
    });
  }

  return { history: { readers, suggestions }, unfinished };
}

function earlierSuggestionCard(item) {
  const li = document.createElement("li");
  li.className = "suggestion";

  const who = document.createElement("p");
  who.className = "who";
  who.textContent = `${item.who} suggested this for “${item.label}”, before the circle moved`;
  li.append(who);

  const text = document.createElement("p");
  text.textContent = item.text;
  li.append(text);

  if (item.because?.trim()) {
    const because = document.createElement("p");
    because.className = "because";
    because.textContent = item.because;
    li.append(because);
  }

  const decided = document.createElement("p");
  decided.className = "outcome";
  decided.textContent = item.accepted ? "Added to the record." : "Set aside.";
  li.append(decided);

  return li;
}

// ---------------------------------------------------------------------------
// The holder's side
// ---------------------------------------------------------------------------

let removing = null; // { key, who } while the question is on screen
let movingNow = false;

function removeThem(key, who) {
  const actions = document.createElement("div");
  actions.className = "actions";

  const button = document.createElement("button");
  button.type = "button";
  button.className = "linky";
  button.textContent = `Remove ${who} from the circle`;
  button.addEventListener("click", () => {
    removing = { key, who };
    const whose = personName();
    $("really-move-question").textContent = whose
      ? `Remove ${who} from ${whose}'s circle?`
      : `Remove ${who} from this circle?`;
    for (const span of document.querySelectorAll(".really-move-whom")) {
      span.textContent = who;
    }
    $("really-move-seconder").hidden = whoAgrees?.agrees !== key;
    $("move-reason").value = "";
    // The ordinary way, every time the box opens. Moving the circle is a
    // choice somebody makes on purpose, never one left over from last time.
    $("remove-ordinary").checked = true;
    $("move-details").hidden = true;
    $("really-move").showModal();
    $("stay-together").focus();
  });

  actions.append(button);
  return actions;
}

$("stay-together").addEventListener("click", () => {
  removing = null;
  $("really-move").close();
});

for (const id of ["remove-ordinary", "remove-and-move"]) {
  $(id).addEventListener("change", () => {
    $("move-details").hidden = !$("remove-and-move").checked;
  });
}

$("move-for-real").addEventListener("click", async () => {
  const leaving = removing;
  const reason = $("move-reason").value.trim();
  const move = $("remove-and-move").checked;
  $("really-move").close();
  removing = null;
  if (!leaving) return;

  // The ordinary way: a decision written in the circle, honoured by every
  // copy of Hearth, including theirs.
  if (!move) {
    try {
      await call(
        "decide_departure",
        { who: leaving.key, removed: true },
        circle.cellId,
      );
      announce(
        `${leaving.who} has been removed. The circle will come off their ` +
          `device the next time their Hearth looks.`,
      );
      await loadCircle();
    } catch (error) {
      problem(error);
    }
    return;
  }

  if (movingNow) {
    announce("A move is already under way. Wait for it to finish first.");
    return;
  }

  movingNow = true;
  announce("Moving the circle. This can take a minute.");
  try {
    await moveTheCircle(leaving.key, leaving.who, reason);
    announce(
      `${leaving.who} has been removed. Everybody else is being moved across, ` +
        `and nobody has to do anything.`,
    );
  } catch (error) {
    problem(error);
  } finally {
    movingNow = false;
  }
});

/**
 * Make the new circle, copy the record into it, and hand everybody else a way in.
 *
 * The order matters in one place. Invitations are made before anybody is
 * asked to agree to who joins in the new circle, because an invitation made
 * afterwards would need that second agreement — and the people being carried
 * across were agreed to already, in the circle they are leaving.
 */
/*
 * Photographs, sound and video, into the circle the people are moving to.
 *
 * They have to be carried piece by piece and written again: a piece is named
 * by a hash of its contents *within one circle's rules*, so the old names mean
 * nothing in the new place. There is no reference to copy, only bytes.
 *
 * This is the slowest part of a move by a wide margin — a two-minute video is
 * tens of megabytes through this function — and it is worth it, because media
 * is here for the people who cannot read a screen, and losing it in a move
 * would take their voice out of their own record.
 *
 * A file that will not come across does not stop the move. The words matter
 * more, and the old circle still has the file until the holder switches it
 * off.
 */
async function carryMediaAcross(from, to) {
  const here = await orNothingYet(call("get_media", null, from), []);
  if (!here.length) return;

  announce(
    here.length === 1
      ? "Carrying one photo, sound or video across."
      : `Carrying ${here.length} photos, sounds or videos across.`,
  );

  for (const item of here) {
    try {
      const pieces = [];
      for (const hash of item.media.pieces ?? []) {
        const bytes = await call("get_media_piece", hash, from);
        pieces.push(await call("add_media_piece", bytes, to));
      }
      if (!pieces.length) continue;

      await call(
        "add_media",
        {
          section: item.media.section,
          kind: item.media.kind,
          mime_type: item.media.mime_type,
          file_name: item.media.file_name ?? "",
          in_words: item.media.in_words ?? "",
          seconds: item.media.seconds ?? 0,
          pieces,
          size: item.media.size,
        },
        to,
      );
    } catch (error) {
      console.error("One file could not be carried across.", error);
    }
  }
}

async function moveTheCircle(removedKey, removedName, reason) {
  const from = circle.cellId;
  const entry = wordsOf(record?.current);
  if (!entry) {
    throw new Error(
      "The record has not arrived on this device yet, so there is nothing to " +
        "carry across. Try again in a moment.",
    );
  }

  // Everything read before anything is made, while this circle is still the
  // one on screen and everything below still describes it.
  const label = labelFor(from, $("circle-heading").textContent.trim() || "Circle");
  const everyone = new Map(members);
  const agreeing = whoAgrees;
  const mine = everyone.get(asText(me));
  const oldRoom = roomFor(from);
  const { history } = await historyOf(from, everyone);

  const made = await call("create_circle", {
    founder: asText(me),
    name: label,
    network_seed: crypto.randomUUID(),
    requires_second_yes: circleAsksTwo,
  });
  const to = made.cell_id;
  markAsOwnRecord(to, isOwnRecord(from));
  setLabelFor(to, label);
  store(historyKey(to), history);

  // A new door. The old address leads to a room nobody answers any more.
  try {
    const room = { holder: asText(me), seed: crypto.randomUUID(), about: entry.display_name };
    await call("enter_waiting_room", {
      holder: room.holder,
      network_seed: room.seed,
      name: `${label} — door`,
    });
    rememberRoom(to, room);
  } catch (error) {
    console.error("Could not open a waiting room for the new circle.", error);
  }

  await call(
    "create_about_me",
    {
      display_name: entry.display_name ?? "",
      what_matters_to_me: entry.what_matters_to_me ?? "",
      people_who_matter: entry.people_who_matter ?? "",
      how_to_communicate_with_me: entry.how_to_communicate_with_me ?? "",
      my_wellness: entry.my_wellness ?? "",
      please_do_and_please_do_not: entry.please_do_and_please_do_not ?? "",
      how_to_support_me: entry.how_to_support_me ?? "",
      also_worth_knowing: entry.also_worth_knowing ?? "",
      supported_to_write_this_by: entry.supported_to_write_this_by ?? "",
      codes: entry.codes ?? [],
    },
    to,
  );

  if (mine?.name?.trim()) {
    await call(
      "introduce_myself",
      { name: mine.name, relationship: mine.relationship ?? "" },
      to,
    );
  }

  await carryMediaAcross(from, to);

  const invitations = [];
  for (const [key, who] of everyone) {
    if (key === asText(me) || key === removedKey) continue;
    const bundle = await call("invite", { invitee: key, name: who.name ?? "" }, to);
    invitations.push({ key, name: who.name ?? "", token: invitationToToken(bundle) });
  }

  // Carried across, unless they are the one being removed. Their app agrees
  // again on its own if they had agreed in the old circle.
  if (agreeing && agreeing.agrees !== removedKey) {
    await call("appoint", agreeing.agrees, to);
  }

  store(movingKey(from), {
    from: encodeCellId(from),
    to: encodeCellId(to),
    reason,
    removed: removedName,
    invitations,
  });

  // The old door, closed on this device. Nobody is there to answer it.
  if (oldRoom) {
    const roomCell = (await waitingRoomCells()).find(
      (r) => r.holder === oldRoom.holder && r.seed === oldRoom.seed,
    );
    if (roomCell) {
      await call("leave_circle", roomCell.cellId[0]).catch((error) =>
        console.error(error),
      );
    }
    forgetRoom(from);
  }

  // Tell everybody now, rather than on the next tick.
  lastTold.delete(asText(from[0]));
  await carryOnMoving();

  forgetTheCircle();
  await loadCircles();
  const item = circles.find((c) => asText(c.cellId[0]) === asText(to[0]));
  if (item) await openCircle(item);
}

/*
 * Keep telling the people who have not moved yet.
 *
 * A signal reaches only somebody whose app is open, and says nothing about
 * whether it arrived. The only proof that somebody got the message is that
 * they are in the new circle. So until they are, they are told again, once a
 * minute, for as long as the holder's app is open.
 *
 * When everybody has arrived, the old circle is switched off here too. The
 * person removed is left in a circle nobody else is in any more.
 */
const TELL_AGAIN_EVERY = 60_000;
const lastTold = new Map(); // old circle dna -> when

async function carryOnMoving() {
  let keys;
  try {
    keys = Object.keys(localStorage).filter((k) => k.startsWith("hearth:moving:"));
  } catch {
    return;
  }

  for (const storageKey of keys) {
    const plan = readStored(storageKey);
    if (!plan?.from || !plan?.to) continue;

    const dna = storageKey.slice("hearth:moving:".length);
    if (Date.now() - (lastTold.get(dna) ?? 0) < TELL_AGAIN_EVERY) continue;
    lastTold.set(dna, Date.now());

    const from = decodeCellId(plan.from);
    const to = decodeCellId(plan.to);

    const arrived = await namesIn(to);
    const stillToCome = plan.invitations.filter((i) => !arrived.has(i.key));

    if (stillToCome.length === 0) {
      await call("leave_circle", from[0]).catch((error) => console.error(error));
      forgetWhatThisDeviceKnew(from);
      store(storageKey, null);
      lastTold.delete(dna);
      continue;
    }

    for (const person of stillToCome) {
      await call(
        "tell_them_it_moved",
        {
          to: person.key,
          invitation: person.token,
          reason: plan.reason ?? "",
          removed: plan.removed ?? "",
        },
        from,
      ).catch((error) => console.error(error));
    }
  }
}

// ---------------------------------------------------------------------------
// Everybody else's side
// ---------------------------------------------------------------------------

const following = new Set();

/**
 * The holder has moved this circle. Go with it.
 *
 * The zome has already refused this from anybody but the holder. What is
 * checked here is the other half: that the circle being moved *to* is hers as
 * well, so nobody can be led anywhere else.
 */
async function followTheMove(fromCellId, payload) {
  if (!fromCellId) return;
  const key = asText(fromCellId[0]);
  if (following.has(key)) return;
  following.add(key);

  try {
    const bundle = tokenToInvitation(payload.invitation);
    if (bundle.founder !== asText(payload.by)) return;

    /*
     * Moved by somebody other than the holder: only a successor whose taking
     * over stands. The zome has checked who; this checks when — the waiting
     * period, and the check on her — which each device answers for itself.
     */
    const oldHolder = asText(
      await orNothingYet(call("who_holds_this", null, fromCellId), null),
    );
    if (oldHolder && oldHolder !== asText(payload.by)) {
      const state = await orNothingYet(call("get_succession", null, fromCellId), null);
      const takeover = state ? decideTakeover(state) : null;
      if (!takeover?.ready || takeover.successor !== asText(payload.by)) return;
    }

    const label = labelFor(fromCellId, bundle.about?.trim() || "Their circle");

    // Read before the old circle is switched off, which is the last chance.
    const names = await namesIn(fromCellId);
    const mine = names.get(asText(me));
    const agreeingThere = await orNothingYet(
      call("who_agrees_here", null, fromCellId),
      null,
    );
    const { history, unfinished } = await historyOf(fromCellId, names);

    const here = await circleAlreadyHere(bundle);
    let to;
    if (here?.enabled) {
      to = here.cellId;
    } else if (here) {
      to = (await call("rejoin_circle", here.cellId[0])).cell_id;
    } else {
      to = (
        await call("join_circle", {
          founder: bundle.founder,
          name: label,
          network_seed: bundle.network_seed,
          invitation: bundle.invitation,
          requires_second_yes: Boolean(bundle.requires_second_yes),
          seconder: bundle.seconder ?? null,
        })
      ).cell_id;

      if (mine?.name?.trim()) {
        await call(
          "introduce_myself",
          { name: mine.name, relationship: mine.relationship ?? "" },
          to,
        );
      }
      for (const suggestion of unfinished) {
        await call("suggest", suggestion, to).catch((error) => console.error(error));
      }
    }

    setLabelFor(to, label);
    markAsOwnRecord(to, false);
    store(historyKey(to), history);
    store(movedNoteKey(to), {
      reason: payload.reason ?? "",
      removed: payload.removed ?? "",
      // Who did it, by the name they gave in the circle being left.
      by: names.get(asText(payload.by))?.name?.trim() ?? "",
    });
    if (agreeingThere?.agrees === asText(me) && agreeingThere.willing === true) {
      store(carryAgreementKey(to), true);
    }

    const wasOpen = asText(circle?.cellId?.[0]) === key;
    await call("leave_circle", fromCellId[0]);
    forgetWhatThisDeviceKnew(fromCellId);

    if (wasOpen) forgetTheCircle();
    await loadCircles();
    if (wasOpen) {
      const item = circles.find((c) => asText(c.cellId[0]) === asText(to[0]));
      if (item) await openCircle(item);
    }

    announce(
      label === "Their circle"
        ? "A circle you are in has moved, and you have been moved with it."
        : `${label}'s circle has moved, and you have been moved with it.`,
    );
  } finally {
    following.delete(key);
  }
}

/*
 * Said once, to somebody whose app moved them.
 *
 * The holder is not shown it: she did it. Everybody else is told that it
 * happened and, where she gave one, why — in her words, and said to be hers.
 */
function sayItMovedIfItDid() {
  const note = circle ? readStored(movedNoteKey(circle.cellId)) : null;
  $("moved-note").hidden = !note;
  if (!note) return;

  /*
   * Who was removed, by whom, and why — not that the circle moved.
   *
   * The move is how removal works, and nobody but the holder has anything to
   * do with it: they cannot invite, so a new door address is nothing to them.
   * The first version said "This circle has moved to a new private space",
   * which described the machinery and left out the one thing that had
   * actually happened to the people in it.
   */
  const by = note.by?.trim() || "The person who holds this circle";
  const removed = note.removed?.trim();
  /*
   * Moved with nobody removed: a successor taking over, or the circle being
   * carried to a newer version of Hearth. "Has moved this circle" is true of
   * both, and the reason underneath says which — where "now holds this
   * circle" was puzzling for somebody whose holder had not changed.
   */
  $("moved-note-text").textContent = removed
    ? `${by} has removed ${removed} from the circle.`
    : `${by} has moved this circle.`;

  const reason = note.reason?.trim();
  $("moved-note-reason").hidden = !reason;
  $("moved-note-reason").textContent = reason ? `The reason given: “${reason}”` : "";
}

/*
 * Carry a circle made under older rules across to the new ones.
 *
 * It is the move that already exists, with nobody removed — the same act as a
 * successor taking a circle over. What makes it a version change is only where
 * the new circle is founded: `create_circle` is asked of the app this
 * interface belongs to, which is the new one, while everything is read from
 * the old.
 */
$("carry-across").addEventListener("click", async () => {
  const button = $("carry-across");
  button.disabled = true;
  try {
    announce("Carrying this circle across. Please leave Hearth open.");
    await moveTheCircle(
      null,
      "",
      "This circle has been carried across to a newer version of Hearth. " +
        "Everything and everybody has come with it.",
    );
    announce("Carried across. Everybody is being moved over as their apps see it.");
  } catch (error) {
    problem(error);
  } finally {
    button.disabled = false;
  }
});

$("moved-note-done").addEventListener("click", () => {
  if (circle) store(movedNoteKey(circle.cellId), null);
  $("moved-note").hidden = true;
});

/**
 * Agree again, on their behalf, to what they had already agreed to.
 *
 * Only for somebody who had said yes to agreeing to who joins in the old
 * circle, only to the same holder asking the same thing, and only once. They
 * are told it was carried across, and can still change their mind the same
 * way as always. Returns true when it answered, so the question is not put
 * on screen for a moment first.
 */
async function carryTheAgreementAcross() {
  if (!circle || !readStored(carryAgreementKey(circle.cellId))) return false;
  if (!whoAgrees) return false; // Not arrived yet. Looked for on every re-read.

  const cellId = circle.cellId;
  store(carryAgreementKey(cellId), null);
  if (whoAgrees.agrees !== asText(me) || whoAgrees.willing !== null) return false;

  await call(
    "answer_appointment",
    { appointment: whoAgrees.appointment, willing: true },
    cellId,
  );
  announce(
    "You had agreed to who joins in the circle before it moved, so that has " +
      "been carried across.",
  );
  // Not awaited: this runs inside a re-read, and waiting for the next one
  // from inside it would wait for itself.
  loadCircle();
  return true;
}

// ---------------------------------------------------------------------------
// How much fits in one section
// ---------------------------------------------------------------------------
//
// Five hundred words a section, Ceri's number, and easy to raise if anybody
// asks.
//
// The reason is not tidiness. Everything written in a circle is copied to
// every member's device, and every change stores another whole copy. The
// only limit Holochain itself sets is four megabytes an entry — roughly a
// thousand pages — so without this, one confused or unkind member could
// fill other people's disks with a single paste.
//
// This is the interface's limit, so an ordinary copy of Hearth keeps to it
// and a modified one need not. The rule every peer checks belongs in the next
// version of the integrity zome. See docs/hard-questions.md.

const WORD_LIMIT = 500;

const WORD_LIMITED = [
  "what-matters",
  "people-who-matter",
  "how-to-communicate",
  "my-wellness",
  "please-do",
  "how-to-support",
  "also-worth-knowing",
  "suggest-text",
  "suggest-because",
];

const countWords = (text) => text.trim().split(/\s+/).filter(Boolean).length;

/*
 * Said only when it is close, and plainly when it is over.
 *
 * A running "12 of 500 words" under every box is a number to watch while
 * trying to think about somebody you love. So nothing is shown until the
 * last hundred words, and going over is a sentence, not a red box.
 */
function sayHowManyWords(box) {
  const note = $(`${box.id}-words`);
  const words = countWords(box.value);
  const over = words - WORD_LIMIT;

  if (over > 0) {
    const message =
      `This is ${words} words, which is ${over} more than fits. ` +
      `Each part holds up to ${WORD_LIMIT} words.`;
    note.textContent = message;
    note.className = "notice";
    note.hidden = false;
    return;
  }

  note.className = "hint";
  note.hidden = words < WORD_LIMIT - 100;
  note.textContent = `${words} of ${WORD_LIMIT} words.`;
}

for (const id of WORD_LIMITED) {
  const box = $(id);
  const note = document.createElement("p");
  note.id = `${id}-words`;
  note.className = "hint";
  note.hidden = true;
  note.setAttribute("aria-live", "polite");
  box.after(note);
  box.addEventListener("input", () => sayHowManyWords(box));
}

/** The first box on this page, or anywhere in this form, that is too long. */
function tooLongIn(container) {
  for (const id of WORD_LIMITED) {
    const box = $(id);
    if (!container.contains(box)) continue;
    sayHowManyWords(box);
    if (countWords(box.value) > WORD_LIMIT) return box;
  }
  return null;
}

/*
 * Say it where the box is, and put the cursor there.
 *
 * Deliberately not the browser's own validity bubble. A form holding a box the
 * browser considers invalid refuses to submit at all when that box is on a
 * page that is not showing, and says nothing about why — the silent dead end
 * the create pages already fell into once. The sentence under the box is on
 * screen and read out, which is all this needs.
 */
function pointAt(box) {
  const page = recordPages().findIndex((p) => p.contains(box));
  if (page >= 0 && page !== recordPage) showRecordPage(page);
  box.focus();
  $(`${box.id}-words`).scrollIntoView({ block: "center" });
}

/*
 * Not past a box that is too long, while it is still on screen to point at.
 *
 * The same lesson as the create pages: a browser will not report a problem on
 * a box it cannot show, so a save that fails on page three while you stand on
 * page seven does nothing and says nothing. So each page is checked as you
 * leave it, and the whole form is checked on saving, going back to the page
 * that needs shortening first.
 *
 * Capture listeners, so they run before the ones that move the page or save.
 */
for (const button of document.querySelectorAll(".record-next")) {
  button.addEventListener(
    "click",
    (event) => {
      const box = tooLongIn(recordPages()[recordPage]);
      if (!box) return;
      event.stopImmediatePropagation();
      pointAt(box);
    },
    { capture: true },
  );
}

$("record-form").addEventListener(
  "submit",
  (event) => {
    const box = tooLongIn($("record-form"));
    if (!box) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    pointAt(box);
  },
  { capture: true },
);

$("suggest-form").addEventListener(
  "submit",
  (event) => {
    const box = tooLongIn($("suggest-form"));
    if (!box) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    pointAt(box);
  },
  { capture: true },
);

// ---------------------------------------------------------------------------
// Somebody removed: the ordinary way (migration batch, item 4)
// ---------------------------------------------------------------------------
//
// The holder writes a decision into the circle. Every copy of Hearth honours
// it: the person drops out of every list, what they write afterwards is not
// shown, and their own copy takes the circle off their device. A modified app
// can ignore all of that, which is what moving the circle is for.

let gone = new Map(); // agent key text -> removed since, in microseconds
let removedMembers = new Map(); // agent key text -> their last introduction

/**
 * Read who stands where, and act on it if it is me.
 *
 * Returns true when this device has just been removed and the circle has come
 * off it — there is then nothing left to draw.
 */
async function readDepartures() {
  if (!circle) return false;
  const standings = await orNothingYet(
    call("get_departures", null, circle.cellId),
    null,
  );
  // Nobody answered: keep what was known rather than showing somebody removed
  // as back in, even for twenty seconds.
  if (!standings) return false;

  gone = new Map(
    standings.filter((s) => s.removed).map((s) => [s.who, Number(s.since)]),
  );

  if (gone.has(asText(me)) && !isHolder()) {
    await takeItOffThisDevice();
    return true;
  }
  return false;
}

/** Something somebody wrote after they were removed. */
function writtenWhileGone(record) {
  const since = gone.get(asText(authorOf(record)));
  if (since === undefined) return false;
  const at = Number(record?.signed_action?.hashed?.content?.header?.timestamp ?? 0);
  return at >= since;
}

/*
 * I have been removed. The circle comes off this device.
 *
 * Deleted, not only switched off: leaving keeps a circle in case somebody
 * changes their mind, but this was somebody else's decision, and what is on
 * this device goes with it. If they are ever let back in, they start fresh
 * with the record as it is then.
 *
 * Said plainly, once. Nobody should find a circle simply gone with no word.
 */
async function takeItOffThisDevice() {
  const cellId = circle.cellId;
  const label = labelFor(cellId, $("circle-heading").textContent.trim() || "");

  try {
    await call("forget_circle", cellId[0]);
  } catch (error) {
    // Deleting failed: at least switch it off, so it is not on screen.
    console.error("Could not delete the circle; switching it off.", error);
    await call("leave_circle", cellId[0]).catch((e) => console.error(e));
  }

  forgetWhatThisDeviceKnew(cellId);
  forgetRoom(cellId);
  forgetTheCircle();
  await loadCircles();

  announce(
    label
      ? `You are no longer in ${label}'s circle. The person who holds it has ` +
          `removed you, so it has been taken off this device.`
      : `You are no longer in this circle. The person who holds it has ` +
          `removed you, so it has been taken off this device.`,
  );
}

/** A line for somebody removed, with a way to let them back. Holder only. */
function someoneRemoved(key, entry) {
  const li = document.createElement("li");
  li.className = "removed";

  const who = entry.name?.trim() || "Somebody";
  const line = document.createElement("p");
  line.textContent = `${who} — removed`;
  li.append(line);

  const hint = document.createElement("p");
  hint.className = "hint";
  hint.textContent =
    "The circle has been taken off their device. Letting them back means they " +
    "can be let in at the door again, and start fresh.";
  li.append(hint);

  const actions = document.createElement("div");
  actions.className = "actions";
  const button = document.createElement("button");
  button.type = "button";
  button.className = "linky";
  button.textContent = `Let ${who} back`;
  button.addEventListener("click", () =>
    whileWorking(button, "Letting them back…", async () => {
      await call("decide_departure", { who: key, removed: false }, circle.cellId);
      announce(
        `${who} can be let in again. Send them the circle's address, and let ` +
          `them in when they ask.`,
      );
      await loadCircle();
    }).catch(problem),
  );
  actions.append(button);
  li.append(actions);

  return li;
}

// ---------------------------------------------------------------------------
// Photos beside the record (migration batch, item 7)
// ---------------------------------------------------------------------------
//
// Photos first, then sound, then video: the order Ceri chose, easiest first.
// The rules underneath already allow all three. See docs/multimedia.md.

/** The section key the record uses -> the name the zome uses. */
const SECTION_FOR_KEY = Object.fromEntries(
  Object.entries(FIELD_LABELS).map(([name, [key]]) => [key, name]),
);

/** Longest side of a photo, in pixels, after shrinking. */
const PHOTO_LONGEST_SIDE = 1600;
/** One piece, as the rules allow. */
const MOST_BYTES_IN_A_PIECE = 3_000_000;

let photoFor = null; // the section key a photo is being added to
let photoBytes = null; // the shrunk photo, waiting for "Add the photo"
let photoPreviewUrl = null;

function pickAPhoto(key) {
  photoFor = key;
  $("photo-file").value = "";
  $("photo-file").click();
}

/*
 * Shrink the photo here, before anything is written.
 *
 * Every member's device holds a copy of every photo, so a twelve-megapixel
 * phone picture written as it is would cost everybody several megabytes for a
 * picture shown a few inches wide. About 1600 pixels on the longest side is
 * sharp on any screen this will be read on, and a few hundred kilobytes.
 */
async function shrinkPhoto(file) {
  const bitmap = await createImageBitmap(file);
  const scale = Math.min(1, PHOTO_LONGEST_SIDE / Math.max(bitmap.width, bitmap.height));
  const canvas = document.createElement("canvas");
  canvas.width = Math.round(bitmap.width * scale);
  canvas.height = Math.round(bitmap.height * scale);
  canvas.getContext("2d").drawImage(bitmap, 0, 0, canvas.width, canvas.height);
  bitmap.close();

  for (const quality of [0.85, 0.7, 0.55]) {
    const blob = await new Promise((resolve) => canvas.toBlob(resolve, "image/jpeg", quality));
    if (blob && blob.size <= MOST_BYTES_IN_A_PIECE) {
      return new Uint8Array(await blob.arrayBuffer());
    }
  }
  throw new Error("That photo is too large even after shrinking it.");
}

$("photo-file").addEventListener("change", async () => {
  const file = $("photo-file").files?.[0];
  if (!file || !photoFor) return;
  try {
    photoBytes = await shrinkPhoto(file);
  } catch (error) {
    console.error(error);
    announce(
      "That picture could not be opened. Photos saved as JPEG or PNG work " +
        "best — some phones save a kind this cannot read.",
    );
    return;
  }
  if (photoPreviewUrl) URL.revokeObjectURL(photoPreviewUrl);
  photoPreviewUrl = URL.createObjectURL(new Blob([photoBytes], { type: "image/jpeg" }));
  $("photo-preview").src = photoPreviewUrl;
  $("photo-words").value = "";
  const [, label] = FIELD_LABELS[SECTION_FOR_KEY[photoFor]] ?? [null, "this section"];
  $("add-photo-question").textContent = `Add this photo to “${label}”?`;
  $("add-photo").showModal();
  $("photo-words").focus();
});

function putThePhotoDown() {
  $("add-photo").close();
  photoBytes = null;
  photoFor = null;
}

$("cancel-photo").addEventListener("click", putThePhotoDown);

$("save-photo").addEventListener("click", () =>
  whileWorking($("save-photo"), "Adding…", async () => {
    const bytes = photoBytes;
    const section = SECTION_FOR_KEY[photoFor];
    const inWords = $("photo-words").value.trim();
    if (!bytes || !section) return;

    const piece = await call("add_media_piece", bytes, circle.cellId);
    await call(
      "add_media",
      {
        section,
        kind: "Photo",
        mime_type: "image/jpeg",
        file_name: $("photo-file").files?.[0]?.name ?? "",
        in_words: inWords,
        seconds: 0,
        pieces: [piece],
        size: bytes.length,
      },
      circle.cellId,
    );
    putThePhotoDown();
    announce("Photo added.");
    shownMedia = "";
    await showMedia();
  }).catch(problem),
);

/*
 * Draw what is beside each section.
 *
 * Pieces are fetched once and kept for as long as the circle is open, as
 * pictures in memory. The twenty-second re-read redraws only when the list
 * has actually changed, so a photo does not flicker every time it looks.
 */
const pieceUrls = new Map(); // entry hash text -> object URL
let shownMedia = ""; // what is drawn now, to skip redrawing the same thing

async function urlForMedia(media) {
  const key = media.pieces.map(asText).join(",");
  if (pieceUrls.has(key)) return pieceUrls.get(key);
  const parts = [];
  for (const piece of media.pieces) {
    parts.push(await call("get_media_piece", piece, circle.cellId));
  }
  const url = URL.createObjectURL(new Blob(parts, { type: media.mime_type }));
  pieceUrls.set(key, url);
  return url;
}

async function showMedia() {
  if (!circle) return;
  const inThisCircle = asText(circle.cellId[0]);
  const all = await orNothingYet(call("get_media", null, circle.cellId), null);
  if (!all || asText(circle?.cellId?.[0]) !== inThisCircle) return;

  const fingerprint = all.map((m) => asText(m.item)).join(",");
  const boxes = document.querySelectorAll(".media-here");
  // Nothing new, and the boxes still hold what was drawn: leave them.
  if (fingerprint === shownMedia && [...boxes].some((b) => b.childElementCount)) return;
  shownMedia = fingerprint;

  for (const box of boxes) box.replaceChildren();

  for (const here of all) {
    const [key] = FIELD_LABELS[here.media.section] ?? [];
    const box = document.querySelector(`.media-here[data-media-for="${key}"]`);
    if (!box) continue;
    const kind = here.media.kind;
    if (kind !== "Photo" && kind !== "Sound" && kind !== "Video") continue;

    const figure = document.createElement("figure");
    figure.className = "media";

    let img = null;
    let audio = null;
    if (kind === "Photo") {
      img = document.createElement("img");
      // The words are the picture for anybody who cannot see it.
      img.alt = here.media.in_words || "A photo, with no description given";
      figure.append(img);
    } else {
      /*
       * Sound, and it never plays by itself. Somebody opening a record on a
       * busy ward has not chosen to have it heard by everybody near them.
       */
      // A video player for video, a sound player for sound. Both are held in
      // `audio` below, because everything done to them is the same.
      audio = document.createElement(kind === "Video" ? "video" : "audio");
      audio.controls = true;
      audio.preload = "metadata";
      if (kind === "Video") audio.setAttribute("playsinline", "");
      const noun = kind === "Video" ? "Video" : "Sound";
      audio.setAttribute(
        "aria-label",
        here.media.in_words
          ? `${noun}: ${here.media.in_words}`
          : `${noun}, with no words given`,
      );
      figure.append(audio);
    }

    /*
     * The words straight under the thing they describe, and the length after
     * them. Each item is its own box, with space between boxes.
     *
     * The first version put the length line between the player and its words,
     * and the words sat closer to the next player than to their own — so a
     * sound's description read as if it belonged to the video below it. Found
     * the first time sound and video were on one section together.
     */
    if (here.media.in_words) {
      const caption = document.createElement("figcaption");
      caption.textContent = here.media.in_words;
      figure.append(caption);
    }

    if (kind !== "Photo" && here.media.seconds) {
      const length = document.createElement("p");
      length.className = "hint media-length";
      const noun = kind === "Video" ? "video" : "sound";
      length.textContent = `${lengthInWords(here.media.seconds)} of ${noun}.`;
      figure.append(length);
    }

    const what = kind === "Photo" ? "photo" : kind === "Video" ? "video" : "sound";

    if (isHolder()) {
      const remove = document.createElement("button");
      remove.type = "button";
      remove.className = "linky";
      remove.textContent = `Remove this ${what}`;
      remove.addEventListener("click", () => {
        if (!confirm(`Remove this ${what} from the record?`)) return;
        whileWorking(remove, "Removing…", async () => {
          await call("remove_media", here.item, circle.cellId);
          announce(`${what[0].toUpperCase()}${what.slice(1)} removed.`);
          shownMedia = "";
          await showMedia();
        }).catch(problem);
      });
      figure.append(remove);
    }

    box.append(figure);

    // The file itself arrives after the frame; a piece not here yet is not
    // an error, just not here yet.
    urlForMedia(here.media)
      .then((url) => {
        if (img) img.src = url;
        if (audio) audio.src = url;
      })
      .catch(() => {
        const notYet = `This ${what} has not arrived on this device yet.`;
        if (img) img.alt = notYet;
        if (audio) audio.setAttribute("aria-label", notYet);
      });
  }
}

/** "1 minute 5 seconds", "40 seconds". */
function lengthInWords(seconds) {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  const minutes = m ? `${m} minute${m === 1 ? "" : "s"}` : "";
  const secs = s ? `${s} second${s === 1 ? "" : "s"}` : "";
  return [minutes, secs].filter(Boolean).join(" ") || "Less than a second";
}

/** Put down the pictures of a circle that is no longer open. */
function forgetMedia() {
  for (const url of pieceUrls.values()) URL.revokeObjectURL(url);
  pieceUrls.clear();
  shownMedia = "";
}

// ---------------------------------------------------------------------------
// Sound beside the record (migration batch, item 7)
// ---------------------------------------------------------------------------
//
// Recorded here or chosen from a file, two minutes at most, heard back before
// it is saved. Speech-quality recording keeps two minutes well under a
// megabyte, so it fits in one piece on every member's device.

const MOST_SECONDS_OF_MEDIA = 120;
/** File types the rules accept for sound, and what some systems call them. */
const SOUND_TYPES = {
  "audio/webm": "audio/webm",
  "audio/ogg": "audio/ogg",
  "audio/mpeg": "audio/mpeg",
  "audio/mp3": "audio/mpeg",
  "audio/mp4": "audio/mp4",
  "audio/x-m4a": "audio/mp4",
  "audio/m4a": "audio/mp4",
};

let soundFor = null;
let soundBytes = null;
let soundType = null;
let soundSeconds = 0;
let soundName = "";
let soundUrl = null;
let recording = null; // { recorder, stream, started, timer }

function showTheSound(bytes, type, seconds, name) {
  soundBytes = bytes;
  soundType = type;
  soundSeconds = seconds;
  soundName = name;
  if (soundUrl) URL.revokeObjectURL(soundUrl);
  soundUrl = URL.createObjectURL(new Blob([bytes], { type }));
  $("sound-preview").src = soundUrl;
  $("sound-preview").hidden = false;
  $("save-sound").disabled = false;
  $("sound-status").textContent =
    `${lengthInWords(seconds)}. Play it back to check it before adding it.`;
}

function pickASound(key) {
  soundFor = key;
  soundBytes = null;
  $("sound-preview").hidden = true;
  $("sound-preview").removeAttribute("src");
  $("save-sound").disabled = true;
  $("sound-status").textContent = "";
  $("sound-words").value = "";
  $("record-sound").textContent = "Start recording";
  const [, label] = FIELD_LABELS[SECTION_FOR_KEY[key]] ?? [null, "this section"];
  $("add-sound-question").textContent = `Add sound to “${label}”?`;
  $("add-sound").showModal();
  $("record-sound").focus();
}

function stopRecording() {
  if (!recording) return;
  clearInterval(recording.timer);
  if (recording.recorder.state !== "inactive") recording.recorder.stop();
  for (const track of recording.stream.getTracks()) track.stop();
}

$("record-sound").addEventListener("click", async () => {
  if (recording) {
    stopRecording();
    return;
  }

  let stream;
  try {
    stream = await navigator.mediaDevices.getUserMedia({ audio: true });
  } catch (error) {
    console.error(error);
    $("sound-status").textContent =
      "The microphone could not be opened. Check it is plugged in and that " +
      "Hearth is allowed to use it, or choose a sound file instead.";
    return;
  }

  const type = MediaRecorder.isTypeSupported("audio/webm;codecs=opus")
    ? "audio/webm;codecs=opus"
    : "audio/webm";
  // Speech quality: clear for a voice, and small on every member's device.
  const recorder = new MediaRecorder(stream, { mimeType: type, audioBitsPerSecond: 32_000 });
  const chunks = [];
  recorder.addEventListener("dataavailable", (e) => {
    if (e.data.size) chunks.push(e.data);
  });
  recorder.addEventListener("stop", async () => {
    const seconds = Math.max(1, Math.round((Date.now() - recording.started) / 1000));
    recording = null;
    $("record-sound").textContent = "Record again";
    const bytes = new Uint8Array(await new Blob(chunks, { type: "audio/webm" }).arrayBuffer());
    if (!bytes.length) {
      $("sound-status").textContent = "Nothing was recorded. Try again.";
      return;
    }
    showTheSound(bytes, "audio/webm", Math.min(seconds, MOST_SECONDS_OF_MEDIA), "recording.webm");
  });

  recording = { recorder, stream, started: Date.now(), timer: 0 };
  recorder.start();
  $("save-sound").disabled = true;
  $("sound-preview").hidden = true;

  // Says how long, and stops itself at two minutes rather than failing after.
  const tick = () => {
    const seconds = Math.round((Date.now() - recording.started) / 1000);
    $("record-sound").textContent = `Stop recording (${lengthInWords(seconds)})`;
    if (seconds >= MOST_SECONDS_OF_MEDIA) {
      stopRecording();
      $("sound-status").textContent = "Two minutes is the most. It has stopped itself.";
    }
  };
  tick();
  recording.timer = setInterval(tick, 1000);
});

$("choose-sound").addEventListener("click", () => {
  stopRecording();
  $("sound-file").value = "";
  $("sound-file").click();
});

/** How long a sound file is, read from the file itself. */
function durationOf(url) {
  return new Promise((resolve, reject) => {
    const probe = new Audio();
    probe.preload = "metadata";
    probe.addEventListener("loadedmetadata", () => resolve(probe.duration));
    probe.addEventListener("error", () => reject(new Error("unreadable")));
    probe.src = url;
  });
}

$("sound-file").addEventListener("change", async () => {
  const file = $("sound-file").files?.[0];
  if (!file) return;

  const type = SOUND_TYPES[file.type];
  if (!type) {
    $("sound-status").textContent =
      "That kind of file cannot be added. MP3, M4A, OGG and WebM sound files work.";
    return;
  }
  if (file.size > MOST_BYTES_IN_A_PIECE) {
    $("sound-status").textContent =
      "That file is more than three megabytes. Recording it here instead " +
      "keeps it small enough.";
    return;
  }

  const bytes = new Uint8Array(await file.arrayBuffer());
  const url = URL.createObjectURL(new Blob([bytes], { type }));
  let seconds;
  try {
    seconds = await durationOf(url);
  } catch {
    URL.revokeObjectURL(url);
    $("sound-status").textContent = "That file could not be played. Try another.";
    return;
  }
  URL.revokeObjectURL(url);
  if (!Number.isFinite(seconds) || seconds > MOST_SECONDS_OF_MEDIA) {
    $("sound-status").textContent =
      "That is longer than two minutes. Two minutes is the most, so the " +
      "people reading this will listen to all of it.";
    return;
  }
  showTheSound(bytes, type, Math.max(1, Math.round(seconds)), file.name);
});

function putTheSoundDown() {
  stopRecording();
  if ($("add-sound").open) $("add-sound").close();
  soundBytes = null;
  soundFor = null;
}

$("cancel-sound").addEventListener("click", putTheSoundDown);
// Closed with Escape: the microphone goes off too.
$("add-sound").addEventListener("close", stopRecording);

$("save-sound").addEventListener("click", () =>
  whileWorking($("save-sound"), "Adding…", async () => {
    const bytes = soundBytes;
    const section = SECTION_FOR_KEY[soundFor];
    if (!bytes || !section) return;

    const piece = await call("add_media_piece", bytes, circle.cellId);
    await call(
      "add_media",
      {
        section,
        kind: "Sound",
        mime_type: soundType,
        file_name: soundName,
        in_words: $("sound-words").value.trim(),
        seconds: soundSeconds,
        pieces: [piece],
        size: bytes.length,
      },
      circle.cellId,
    );
    putTheSoundDown();
    announce("Sound added.");
    shownMedia = "";
    await showMedia();
  }).catch(problem),
);

// ---------------------------------------------------------------------------
// Video beside the record (migration batch, item 7)
// ---------------------------------------------------------------------------
//
// 480p and two minutes, Ceri's numbers. At about 900 kilobits a second that
// is roughly thirteen megabytes for two minutes: five pieces of three, well
// inside the rules' ten pieces and thirty megabytes.

const VIDEO_HEIGHT = 480;
const VIDEO_BITS_PER_SECOND = 900_000;
const MOST_BYTES_IN_A_VIDEO = 30_000_000;
const VIDEO_TYPES_KEPT_AS_THEY_ARE = ["video/webm", "video/mp4"];

let videoFor = null;
let videoBytes = null;
let videoType = null;
let videoSeconds = 0;
let videoName = "";
let videoUrl = null;
let filming = null; // { recorder, stream, started, timer, kind }

function showTheVideo(bytes, type, seconds, name) {
  if (bytes.length > MOST_BYTES_IN_A_VIDEO) {
    $("video-status").textContent =
      "That video is still too large. Try a shorter one, or record it here.";
    return;
  }
  videoBytes = bytes;
  videoType = type;
  videoSeconds = seconds;
  videoName = name;
  if (videoUrl) URL.revokeObjectURL(videoUrl);
  videoUrl = URL.createObjectURL(new Blob([bytes], { type }));
  $("video-live").hidden = true;
  $("video-preview").src = videoUrl;
  $("video-preview").hidden = false;
  $("save-video").disabled = false;
  const mb = (bytes.length / 1_000_000).toFixed(1);
  $("video-status").textContent =
    `${lengthInWords(seconds)}, ${mb} megabytes. Play it back to check it before adding it.`;
}

function pickAVideo(key) {
  videoFor = key;
  videoBytes = null;
  $("video-preview").hidden = true;
  $("video-preview").removeAttribute("src");
  $("video-live").hidden = true;
  $("save-video").disabled = true;
  $("video-status").textContent = "";
  $("video-words").value = "";
  $("record-video").textContent = "Start recording";
  const [, label] = FIELD_LABELS[SECTION_FOR_KEY[key]] ?? [null, "this section"];
  $("add-video-question").textContent = `Add video to “${label}”?`;
  $("add-video").showModal();
  $("record-video").focus();
}

function stopFilming() {
  if (!filming) return;
  clearInterval(filming.timer);
  if (filming.recorder.state !== "inactive") filming.recorder.stop();
  for (const track of filming.stream.getTracks()) track.stop();
  if (filming.source) filming.source.pause();
}

/** The best WebM this device can record. */
function aVideoRecordingType() {
  for (const type of ["video/webm;codecs=vp8,opus", "video/webm;codecs=vp9,opus", "video/webm"]) {
    if (MediaRecorder.isTypeSupported(type)) return type;
  }
  return "";
}

/*
 * Record a stream at 480p and hand back the bytes. Used both for the camera
 * and for shrinking a chosen file, which is the same job: something plays,
 * and this writes it down smaller.
 */
function recordStream(stream, { onTick, onDone, kind, source }) {
  const type = aVideoRecordingType();
  const recorder = new MediaRecorder(stream, {
    mimeType: type || undefined,
    videoBitsPerSecond: VIDEO_BITS_PER_SECOND,
    audioBitsPerSecond: 48_000,
  });
  const chunks = [];
  recorder.addEventListener("dataavailable", (e) => {
    if (e.data.size) chunks.push(e.data);
  });
  recorder.addEventListener("stop", async () => {
    const seconds = Math.max(1, Math.round((Date.now() - filming.started) / 1000));
    filming = null;
    const bytes = new Uint8Array(await new Blob(chunks, { type: "video/webm" }).arrayBuffer());
    onDone(bytes, Math.min(seconds, MOST_SECONDS_OF_MEDIA));
  });
  filming = { recorder, stream, started: Date.now(), timer: 0, kind, source };
  recorder.start(1000);
  const tick = () => {
    const seconds = Math.round((Date.now() - filming.started) / 1000);
    onTick(seconds);
    if (seconds >= MOST_SECONDS_OF_MEDIA) stopFilming();
  };
  tick();
  filming.timer = setInterval(tick, 1000);
}

$("record-video").addEventListener("click", async () => {
  if (filming) {
    stopFilming();
    return;
  }

  let stream;
  try {
    stream = await navigator.mediaDevices.getUserMedia({
      video: { height: { ideal: VIDEO_HEIGHT }, width: { ideal: 854 }, facingMode: "user" },
      audio: true,
    });
  } catch (error) {
    console.error(error);
    $("video-status").textContent =
      "The camera could not be opened. Check it is connected and that Hearth " +
      "is allowed to use it, or choose a video file instead.";
    return;
  }

  // What the camera sees while recording, silent so it does not echo.
  $("video-preview").hidden = true;
  $("video-live").srcObject = stream;
  $("video-live").hidden = false;
  await $("video-live").play().catch(() => {});
  $("save-video").disabled = true;

  recordStream(stream, {
    kind: "camera",
    onTick: (seconds) => {
      $("record-video").textContent = `Stop recording (${lengthInWords(seconds)})`;
      if (seconds >= MOST_SECONDS_OF_MEDIA) {
        $("video-status").textContent = "Two minutes is the most. It has stopped itself.";
      }
    },
    onDone: (bytes, seconds) => {
      $("video-live").srcObject = null;
      $("record-video").textContent = "Record again";
      if (!bytes.length) {
        $("video-status").textContent = "Nothing was recorded. Try again.";
        return;
      }
      showTheVideo(bytes, "video/webm", seconds, "recording.webm");
    },
  });
});

$("choose-video").addEventListener("click", () => {
  stopFilming();
  $("video-file").value = "";
  $("video-file").click();
});

/** A video element playing a file, ready to be read. */
function videoFrom(url) {
  return new Promise((resolve, reject) => {
    const video = document.createElement("video");
    video.preload = "auto";
    video.playsInline = true;
    video.addEventListener("loadedmetadata", () => resolve(video), { once: true });
    video.addEventListener("error", () => reject(new Error("unreadable")), { once: true });
    video.src = url;
  });
}

/*
 * Make a chosen video smaller, by playing it into a 480p canvas and recording
 * that. It takes as long as the video lasts, and says so. The sound comes from
 * the video itself; the player is kept silent so it is not heard while it
 * works.
 */
function shrinkVideo(video, name) {
  const scale = Math.min(1, VIDEO_HEIGHT / video.videoHeight);
  const canvas = document.createElement("canvas");
  // Even numbers: some encoders refuse odd sizes.
  canvas.width = Math.round((video.videoWidth * scale) / 2) * 2;
  canvas.height = Math.round((video.videoHeight * scale) / 2) * 2;
  const draw = canvas.getContext("2d");

  const tracks = [...canvas.captureStream(25).getVideoTracks()];
  try {
    tracks.push(...video.captureStream().getAudioTracks());
  } catch {
    // No sound track to take; the video is kept without sound.
  }
  const stream = new MediaStream(tracks);

  video.volume = 0;
  const paint = () => {
    if (!filming || video.paused || video.ended) return;
    draw.drawImage(video, 0, 0, canvas.width, canvas.height);
    requestAnimationFrame(paint);
  };

  $("save-video").disabled = true;
  $("record-video").disabled = true;
  recordStream(stream, {
    kind: "shrinking",
    source: video,
    onTick: (seconds) => {
      const total = Math.round(video.duration);
      $("video-status").textContent =
        `Making it smaller: ${lengthInWords(Math.min(seconds, total))} of ` +
        `${lengthInWords(total)}. It takes as long as the video lasts.`;
    },
    onDone: (bytes) => {
      $("record-video").disabled = false;
      URL.revokeObjectURL(video.src);
      showTheVideo(bytes, "video/webm", Math.max(1, Math.round(video.duration)), name);
    },
  });
  video.addEventListener("ended", stopFilming, { once: true });
  video.play().then(paint).catch((error) => {
    console.error(error);
    stopFilming();
    $("video-status").textContent = "That video could not be played. Try another.";
  });
}

$("video-file").addEventListener("change", async () => {
  const file = $("video-file").files?.[0];
  if (!file) return;

  const url = URL.createObjectURL(file);
  let video;
  try {
    video = await videoFrom(url);
  } catch {
    URL.revokeObjectURL(url);
    $("video-status").textContent =
      "That video could not be opened. MP4 and WebM videos work best.";
    return;
  }

  if (!Number.isFinite(video.duration) || video.duration > MOST_SECONDS_OF_MEDIA + 0.5) {
    URL.revokeObjectURL(url);
    $("video-status").textContent =
      "That is longer than two minutes. Two minutes is the most, so the " +
      "people reading this will watch all of it.";
    return;
  }

  // Small enough and 480p or less already: kept exactly as it is.
  if (
    VIDEO_TYPES_KEPT_AS_THEY_ARE.includes(file.type) &&
    file.size <= MOST_BYTES_IN_A_VIDEO &&
    video.videoHeight <= VIDEO_HEIGHT
  ) {
    URL.revokeObjectURL(url);
    const bytes = new Uint8Array(await file.arrayBuffer());
    showTheVideo(bytes, file.type, Math.max(1, Math.round(video.duration)), file.name);
    return;
  }

  shrinkVideo(video, file.name);
});

function putTheVideoDown() {
  stopFilming();
  if ($("add-video").open) $("add-video").close();
  $("video-live").srcObject = null;
  $("record-video").disabled = false;
  videoBytes = null;
  videoFor = null;
}

$("cancel-video").addEventListener("click", putTheVideoDown);
// Closed with Escape: the camera goes off too.
$("add-video").addEventListener("close", () => {
  stopFilming();
  $("video-live").srcObject = null;
});

$("save-video").addEventListener("click", () =>
  whileWorking($("save-video"), "Adding…", async () => {
    const bytes = videoBytes;
    const section = SECTION_FOR_KEY[videoFor];
    if (!bytes || !section) return;

    // In pieces, one at a time, saying how far it has got.
    const count = Math.ceil(bytes.length / MOST_BYTES_IN_A_PIECE);
    const pieces = [];
    for (let i = 0; i < count; i++) {
      $("video-status").textContent = `Saving part ${i + 1} of ${count}…`;
      const part = bytes.slice(i * MOST_BYTES_IN_A_PIECE, (i + 1) * MOST_BYTES_IN_A_PIECE);
      pieces.push(await call("add_media_piece", part, circle.cellId));
    }

    await call(
      "add_media",
      {
        section,
        kind: "Video",
        mime_type: videoType,
        file_name: videoName,
        in_words: $("video-words").value.trim(),
        seconds: videoSeconds,
        pieces,
        size: bytes.length,
      },
      circle.cellId,
    );
    putTheVideoDown();
    announce("Video added.");
    shownMedia = "";
    await showMedia();
  }).catch(problem),
);

// ---------------------------------------------------------------------------
// A successor (migration batch, item 9)
// ---------------------------------------------------------------------------
//
// Ceri's rules, 19 September 2026: anybody in the circle may be named; a
// waiting period everybody can see, in which the holder can say "I'm still
// here"; the holder can change or remove the successor at any time; and a
// checker she names, asked first, with anybody else able to answer if the
// checker cannot.
//
// Who may write each step is checked by every device. When — the waiting
// period, and when others may answer — is decided here, by every copy of
// Hearth. In the demo the waits are minutes, so it can be walked in one
// sitting; in the released app they are fourteen days and seven.

const DAY = 24 * 60 * 60 * 1000;
const WAIT_BEFORE_TAKING_OVER = import.meta.env.DEV ? 2 * 60 * 1000 : 14 * DAY;
const OTHERS_MAY_ANSWER_AFTER = import.meta.env.DEV ? 60 * 1000 : 7 * DAY;

let succession = null; // what get_succession said, for the circle on screen

/** Microseconds from the zome to milliseconds for the clock. */
const msOf = (timestamp) => Number(timestamp) / 1000;

function whenInWords(ms) {
  const when = new Date(ms);
  return import.meta.env.DEV
    ? when.toLocaleTimeString("en-GB")
    : when.toLocaleDateString("en-GB", { day: "numeric", month: "long" });
}

/**
 * Where a taking-over stands, from what the circle says and the clock.
 *
 * The checker's word decides if they have given it. If they have not, the
 * first answer from anybody else — given once others may answer — decides.
 * A "she can carry on" stops it; a "she cannot" lets it go ahead once the
 * waiting period is over.
 */
function decideTakeover(state, now = Date.now()) {
  const claim = state?.claim;
  if (!claim) return null;
  const started = msOf(claim.at);
  const othersFrom = started + OTHERS_MAY_ANSWER_AFTER;
  const readyFrom = started + WAIT_BEFORE_TAKING_OVER;

  const byChecker = claim.checks.filter((c) => c.by === state.checker).pop() ?? null;
  const byOthers = claim.checks.find(
    (c) => c.by !== state.checker && msOf(c.at) >= othersFrom,
  );
  const decisive = byChecker ?? byOthers ?? null;

  const out = {
    successor: claim.by,
    claim: claim.claim,
    started,
    othersFrom,
    readyFrom,
    stopped: claim.still_here,
    checkedFine: Boolean(decisive?.holder_can_carry_on),
    checkedCannot: Boolean(decisive && !decisive.holder_can_carry_on),
    checkerAnswered: Boolean(byChecker),
    othersMayAnswer: !byChecker && now >= othersFrom,
    ready: false,
  };
  out.ready = !out.stopped && out.checkedCannot && now >= readyFrom;
  return out;
}

async function readSuccession() {
  if (!circle) return;
  const state = await orNothingYet(call("get_succession", null, circle.cellId), null);
  if (!state) return;
  succession = state;
  renderSuccession();
}

const nameOf = (key) => members.get(key)?.name?.trim() || "Somebody";

function aButton(text, secondary, onPress) {
  const b = document.createElement("button");
  b.type = "button";
  if (secondary) b.className = "secondary";
  b.textContent = text;
  b.addEventListener("click", () => onPress(b));
  return b;
}

function aLine(text, className) {
  const p = document.createElement("p");
  if (className) p.className = className;
  p.textContent = text;
  return p;
}

function renderSuccession() {
  const body = $("succession-body");
  const banner = $("taking-over-banner");
  /*
   * Held still while somebody is using it. The circle is read again every
   * twenty seconds, and redrawing the lists would put back the old choice
   * underneath somebody half-way through making a new one.
   */
  if (body.contains(document.activeElement) || banner.contains(document.activeElement)) {
    return;
  }
  body.replaceChildren();
  banner.replaceChildren();
  if (!succession) {
    $("succession").hidden = true;
    banner.hidden = true;
    return;
  }

  const mine = asText(me);
  const amHolder = isHolder();
  const holderName = nameOf(holder);
  const takeover = decideTakeover(succession);
  const active = takeover && !takeover.stopped && !takeover.checkedFine;

  // ---- Somebody is taking over: everybody sees it, at the top of the record.
  if (active) {
    const who = nameOf(takeover.successor);
    banner.append(
      aLine(
        amHolder
          ? `${who} has said you can no longer look after this circle, and has started to take it over.`
          : `${who} has said ${holderName} can no longer look after this circle, and has started to take it over.`,
      ),
    );
    if (amHolder) {
      banner.append(
        aLine("If you can still look after it, press this. It stops it at once."),
      );
      const row = document.createElement("div");
      row.className = "actions";
      row.append(
        aButton("I'm still here", false, (b) =>
          whileWorking(b, "Saying so…", async () => {
            await call("still_here", takeover.claim, circle.cellId);
            announce("Done. Nobody can take this circle over from you.");
            await readSuccession();
          }).catch(problem),
        ),
      );
      banner.append(row);
    } else {
      banner.append(
        aLine(
          takeover.checkedCannot
            ? `Somebody has checked, and says ${holderName} cannot carry on. ${who} can take over from ${whenInWords(takeover.readyFrom)}.`
            : `Nothing changes until somebody has checked on ${holderName}, and not before ${whenInWords(takeover.readyFrom)}.`,
          "hint",
        ),
      );
    }
  }
  banner.hidden = !active;

  // ---- The section on the People page.
  const show = [];

  // The holder names who takes over, and who checks.
  if (amHolder) {
    show.push(
      aLine(
        "Somebody to take over if you ever cannot look after this circle, and " +
          "somebody who lives near you or can phone you, to check first. " +
          "Nothing happens unless the person you name starts it, everybody in " +
          "the circle sees if they do, and you can stop it by saying you are " +
          "still here.",
        "hint",
      ),
    );
    const others = [...members.keys()].filter((k) => k !== mine);
    if (others.length === 0) {
      show.push(aLine("There is nobody else in the circle to name yet.", "hint"));
    } else {
      const choose = (id, label, current) => {
        const field = document.createElement("div");
        field.className = "field";
        const l = document.createElement("label");
        l.htmlFor = id;
        l.textContent = label;
        const select = document.createElement("select");
        select.id = id;
        select.append(new Option("Nobody", ""));
        for (const k of others) select.append(new Option(nameOf(k), k, false, k === current));
        field.append(l, select);
        return field;
      };
      show.push(choose("successor-choice", "Who takes over", succession.successor));
      show.push(choose("checker-choice", "Who checks on you first", succession.checker));
      const row = document.createElement("div");
      row.className = "actions";
      row.append(
        aButton("Save", false, (b) =>
          whileWorking(b, "Saving…", async () => {
            const successor = $("successor-choice").value || null;
            const checker = $("checker-choice").value || null;
            if (successor && successor === checker) {
              announce("The person who checks has to be somebody other than the one who takes over.");
              return;
            }
            await call("name_successor", { successor, checker }, circle.cellId);
            announce(successor ? "Saved." : "Nobody is named to take over now.");
            await readSuccession();
          }).catch(problem),
        ),
      );
      show.push(row);
    }
  }

  // The successor, named and not yet started.
  if (!amHolder && succession.successor === mine && !active) {
    show.push(
      aLine(
        `${holderName} has named you to take over this circle if they can no ` +
          `longer look after it.`,
      ),
    );
    show.push(
      aLine(
        `Only start if that has happened. Everybody in the circle will see it, ` +
          `${holderName} can stop it by saying they are still here, and ` +
          `somebody has to check on them before you can take over.`,
        "hint",
      ),
    );
    const row = document.createElement("div");
    row.className = "actions";
    row.append(
      aButton("Start taking over", true, (b) => {
        if (!confirm(`Tell everybody that ${holderName} can no longer look after this circle?`)) return;
        whileWorking(b, "Starting…", async () => {
          await call("start_taking_over", null, circle.cellId);
          announce("Started. Everybody in the circle can see it.");
          await readSuccession();
        }).catch(problem);
      }),
    );
    show.push(row);
  }

  // The successor, started: where it stands, and the last step when it is time.
  if (active && takeover.successor === mine) {
    if (takeover.ready) {
      show.push(
        aLine(
          `Somebody has checked, ${holderName} has not said they are still ` +
            `here, and the waiting is over. You can take over now: Hearth will ` +
            `make a new circle with you holding it, and move everybody across.`,
        ),
      );
      const row = document.createElement("div");
      row.className = "actions";
      row.append(
        aButton("Take over the circle", false, (b) => {
          if (!confirm("Make a new circle that you hold, and move everybody into it?")) return;
          whileWorking(b, "Taking over…", () => takeOver(holderName)).catch(problem);
        }),
      );
      show.push(row);
    } else {
      show.push(
        aLine(
          takeover.checkedCannot
            ? `Somebody has checked. You can take over from ${whenInWords(takeover.readyFrom)}.`
            : `Waiting for somebody to check on ${holderName}. You can take over ` +
                `no sooner than ${whenInWords(takeover.readyFrom)}.`,
          "hint",
        ),
      );
    }
  }

  // The checker, and — if the checker has not answered — everybody else.
  const amChecker = succession.checker === mine;

  // Chosen to check, and nothing has happened: said, so it is not a surprise
  // on the day.
  if (!amHolder && amChecker && !active) {
    show.push(
      aLine(
        `${holderName} has chosen you to check on them, in person or by phone, ` +
          `if anybody ever says they can no longer look after this circle. ` +
          `There is nothing to do unless that happens.`,
      ),
    );
  }
  if (active && !amHolder && takeover.successor !== mine && !takeover.checkerAnswered) {
    if (amChecker || takeover.othersMayAnswer) {
      show.push(
        aLine(
          amChecker
            ? `You are the person asked to check on ${holderName}. Please see ` +
                `them, or phone them, and then say:`
            : `The person asked to check has not answered. If you can see or ` +
                `phone ${holderName}, please do, and then say:`,
        ),
      );
      const answer = (canCarryOn) => (b) =>
        whileWorking(b, "Saying so…", async () => {
          await call(
            "check_on_holder",
            { claim: takeover.claim, holder_can_carry_on: canCarryOn },
            circle.cellId,
          );
          announce("Thank you. Everybody in the circle can see your answer.");
          await readSuccession();
        }).catch(problem);
      const row = document.createElement("div");
      row.className = "actions";
      row.append(
        aButton(`${holderName} can carry on`, false, answer(true)),
        aButton(`${holderName} cannot carry on`, true, answer(false)),
      );
      show.push(row);
    } else {
      show.push(
        aLine(
          `The person ${holderName} chose to check on them is asked first. If ` +
            `they have not answered by ${whenInWords(takeover.othersFrom)}, you ` +
            `can.`,
          "hint",
        ),
      );
    }
  }

  if (takeover?.stopped) {
    show.push(aLine(`${holderName} has said they are still here, so nobody is taking over.`, "hint"));
  } else if (takeover?.checkedFine) {
    show.push(aLine(`Somebody has checked, and ${holderName} can carry on, so nobody is taking over.`, "hint"));
  }

  body.append(...show);
  $("succession").hidden = show.length === 0;
}

/** The last step: a new circle the successor holds, and everybody moved. */
async function takeOver(holderName) {
  const mineName = members.get(asText(me))?.name?.trim() || "The person named";
  await moveTheCircle(
    null,
    "",
    `${holderName} can no longer look after this circle, so ${mineName} has taken it over.`,
  );
  announce("You hold this circle now. Everybody is being moved across.");
}
