//! Adversarial tests.
//!
//! Every rule this project claims is a rule only if something rejects when it
//! is broken. These tests exist to break them.
//!
//! Prompted by an external red-team review which found that membership had
//! been implemented far more strongly than authorship, plus two holes it
//! missed: link creation and deletes were entirely unvalidated.

use aboutme_integrity::{AboutMe, CircleProperties, Invitation, Suggestion};
use holochain::prelude::*;
use holochain::sweettest::*;
use std::collections::HashMap;
use std::path::PathBuf;

const ZOME: &str = "aboutme";

fn dna_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("dnas/aboutme/workdir/aboutme.dna")
}

/// All seven sections of About Me, so a test that round-trips a record
/// round-trips the whole of one.
fn an_about_me(name: &str) -> AboutMe {
    AboutMe {
        display_name: name.to_string(),
        what_matters_to_me: "Seeing my grandchildren".into(),
        people_who_matter: "My daughter Ruth".into(),
        how_to_communicate_with_me: "Speak to my left side, I'm deaf on the right".into(),
        my_wellness: "I am not myself when I stop reading".into(),
        please_do_and_please_do_not: "Please do not move my chair".into(),
        how_to_support_me: "Give me time to answer".into(),
        also_worth_knowing: "I was a district nurse for thirty years".into(),
        supported_to_write_this_by: "My daughter Ruth".into(),
        codes: Vec::new(),
        // The words as they are typed. The zome locks them on the way in, so
        // nothing a test writes is ever the locked form.
        locked: None,
    }
}

/// A circle whose founder is `founder`, built from the real packed DNA.
async fn circle_dna(founder: &AgentPubKey) -> DnaFile {
    circle_dna_with_seconder(founder, None).await
}

/// The same, for a circle that asks two people to agree before anybody joins.
async fn circle_dna_with_seconder(
    founder: &AgentPubKey,
    seconder: Option<&AgentPubKey>,
) -> DnaFile {
    let properties = CircleProperties {
        founder: Some(founder.to_string()),
        lobby: false,
        // Nobody is named in the circle's identity any more. The flag says
        // two people must agree; who the second is, is written inside the
        // circle and can be written again. The argument is kept so callers
        // still read as "a circle that asks two people".
        seconder: None,
        requires_second_yes: seconder.is_some(),
        waiting_for: None,
    };
    SweetDnaFile::from_bundle_with_overrides(
        &dna_path(),
        DnaModifiersOpt::none().with_properties(properties),
    )
    .await
    .expect("the packed DNA should load; run `hc dna pack` first")
}

/// Install the circle for one agent, optionally presenting an invitation.
///
/// Returns `Err` when the membrane rejects them, which is the point of several
/// of these tests.
async fn join(
    conductor: &SweetConductor,
    app_id: &str,
    agent: &AgentPubKey,
    dna: &DnaFile,
    invitation: Option<&Invitation>,
) -> anyhow::Result<CellId> {
    let bundle = app_bundle_from_dnas(&[("circle".to_string(), dna.clone())], false, None).await;

    let membrane_proof = invitation
        .map(|i| SerializedBytes::try_from(i.clone()).map(MembraneProof::new))
        .transpose()?;

    let roles = HashMap::from([(
        "circle".to_string(),
        RoleSettings::Provisioned {
            membrane_proof,
            modifiers: None,
            init_properties: None,
        },
    )]);

    let app = conductor
        .raw_handle()
        .install_app_bundle(InstallAppPayload {
            source: AppBundleSource::Bytes(bundle.pack()?.into()),
            agent_key: Some(agent.clone()),
            installed_app_id: Some(app_id.to_string()),
            roles_settings: Some(roles),
            network_seed: None,
            ignore_genesis_failure: false,
            restore_from_dht: false,
        })
        .await?;

    conductor
        .raw_handle()
        .enable_app(app_id.to_string())
        .await?;

    let cell_id = app
        .provisioned_cells()
        .next()
        .map(|(_, cell_id)| cell_id)
        .ok_or_else(|| anyhow::anyhow!("no provisioned cell"))?;

    Ok(cell_id)
}

fn zome(cell_id: &CellId) -> SweetZome {
    SweetZome::new(cell_id.clone(), ZOME.into())
}

/// A hash that refers to nothing, for tests about a message rather than about
/// what the message points at. A signal is never looked up — it is a nudge, and
/// the tests below are about who it claims to be from.
fn fake_hash() -> ActionHash {
    ActionHash::from_raw_36(vec![0; 36])
}

/// Alice founds a circle and Bob joins it with a genuine invitation.
async fn a_circle_with_a_member() -> (SweetConductor, CellId, CellId) {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let bob = SweetAgents::one(conductor.keystore()).await;
    let dna = circle_dna(&alice).await;

    let alice_cell = join(&conductor, "alice", &alice, &dna, None)
        .await
        .expect("the founder needs no invitation to her own circle");

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    let bob_cell = join(&conductor, "bob", &bob, &dna, Some(&bundle.invitation))
        .await
        .expect("an invited agent should be admitted");

    // Keys, as the app does on every pass. Everything written in a circle is
    // locked, so without this a member cannot offer a suggestion and the holder
    // cannot write the record — which is a fact about the app worth knowing,
    // not something to work around: see `keep_keys_up_to_date`.
    keys_flowing(&conductor, &alice_cell, &[&bob_cell]).await;

    (conductor, alice_cell, bob_cell)
}

/// Get the circle's keys to everybody, the way the app does when it opens one.
///
/// The member's device publishes its encryption key, the holder makes the
/// circle's key and seals it to whoever is owed one, and the member takes it up.
/// In one conductor this takes one pass each way.
async fn keys_flowing(conductor: &SweetConductor, holder: &CellId, members: &[&CellId]) {
    for cell in members {
        let _: aboutme::KeysHere = conductor.call(&zome(cell), "keep_keys_up_to_date", ()).await;
    }
    keys_until(conductor, holder, |k| k.epoch >= 1 && k.mine >= 1).await;
    for cell in members {
        keys_until(conductor, cell, |k| k.mine >= 1).await;
    }
}

// ---------------------------------------------------------------------------
// The membrane: who may enter
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn uninvited_agent_cannot_join() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let mallory = SweetAgents::one(conductor.keystore()).await;
    let dna = circle_dna(&alice).await;

    join(&conductor, "alice", &alice, &dna, None).await.unwrap();

    let result = join(&conductor, "mallory", &mallory, &dna, None).await;
    assert!(
        result.is_err(),
        "an agent with no invitation must not be able to join"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn an_invitation_cannot_be_passed_on() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let bob = SweetAgents::one(conductor.keystore()).await;
    let dave = SweetAgents::one(conductor.keystore()).await;
    let dna = circle_dna(&alice).await;

    let alice_cell = join(&conductor, "alice", &alice, &dna, None).await.unwrap();

    // Alice invites Bob. Bob hands his invitation to Dave.
    let for_bob: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    let result = join(&conductor, "dave", &dave, &dna, Some(&for_bob.invitation)).await;
    assert!(
        result.is_err(),
        "an invitation is signed over the invitee's own key and must not transfer"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_forge_an_invitation() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let bob = SweetAgents::one(conductor.keystore()).await;
    let mallory = SweetAgents::one(conductor.keystore()).await;
    let dna = circle_dna(&alice).await;

    let alice_cell = join(&conductor, "alice", &alice, &dna, None).await.unwrap();
    let for_bob: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;
    let bob_cell = join(&conductor, "bob", &bob, &dna, Some(&for_bob.invitation))
        .await
        .unwrap();

    // Bob can call invite() — there is no permission check on it — but his
    // signature is not the founder's.
    let forged: aboutme::InvitationBundle = conductor
        .call(
            &zome(&bob_cell),
            "invite",
            aboutme::InviteInput {
                invitee: mallory.to_string(),
                name: String::new(),
            },
        )
        .await;

    let result = join(
        &conductor,
        "mallory",
        &mallory,
        &dna,
        Some(&forged.invitation),
    )
    .await;
    assert!(
        result.is_err(),
        "only the founder's signature admits anyone"
    );
}

// ---------------------------------------------------------------------------
// Authorship: who may speak as the person
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn the_person_can_write_their_own_about_me() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;

    let record: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;

    assert!(record.action().entry_hash().is_some());
}

/// The fault that produced four separate bugs on four screens.
///
/// Every list is fetched from the network, which is right for other people
/// and wrong for me: for the first seconds after I write something the
/// network has not heard of it. The screens said so — a "Who are you?" form
/// on the page just filled in, "Nothing has been written yet" above "Read
/// this over".
///
/// These read immediately after writing, with no wait and no polling,
/// because that is the moment the person is looking at the screen.
#[tokio::test(flavor = "multi_thread")]
async fn what_i_just_wrote_is_there_the_instant_i_look() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    // Bob introduces himself and must be in the list at once.
    let _: Record = conductor
        .call(
            &zome(&bob_cell),
            "introduce_myself",
            aboutme_integrity::Member {
                name: "Gareth".to_string(),
                relationship: "her son".to_string(),
            },
        )
        .await;

    let members: Vec<Record> = conductor.call(&zome(&bob_cell), "get_members", ()).await;
    assert!(
        !members.is_empty(),
        "a person who has just said who they are must not be asked again"
    );

    // Alice writes her record and must find it at once.
    let created: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;
    let original = created.action_address().clone();

    let originals: Vec<ActionHash> = conductor
        .call(&zome(&alice_cell), "get_circle_about_me", ())
        .await;
    assert!(
        originals.contains(&original),
        "\"Nothing has been written yet\" about words just typed in"
    );

    // And a correction must be what she sees, not the version before it.
    let mut corrected = an_about_me("Alice Bell");
    corrected.what_matters_to_me = "Seeing my grandchildren on Sundays".into();
    let updated: Record = conductor
        .call(
            &zome(&alice_cell),
            "update_about_me",
            aboutme::UpdateAboutMeInput {
                original_action_hash: original.clone(),
                previous_action_hash: original.clone(),
                about_me: corrected,
            },
        )
        .await;

    let current: aboutme::CurrentAboutMe = conductor
        .call(&zome(&alice_cell), "get_current_about_me", original)
        .await;
    assert_eq!(
        current.record.as_ref().map(|r| r.action_address().clone()),
        Some(updated.action_address().clone()),
        "a correction must be the version she is looking at"
    );

    // Bob offers something and must see that it was written down.
    let suggestion: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    let offered: Vec<aboutme::SuggestionWithOutcome> = conductor
        .call(&zome(&bob_cell), "get_suggestions", ())
        .await;
    assert!(
        offered
            .iter()
            .any(|s| s.suggestion.action_address() == suggestion.action_address()),
        "somebody who offers something must be able to see that it was taken"
    );

    // And Alice's decision must stick, or she will decide it twice.
    let _: Record = conductor
        .call(
            &zome(&alice_cell),
            "decide_on_suggestion",
            aboutme::DecideInput {
                suggestion: suggestion.action_address().clone(),
                accepted: true,
            },
        )
        .await;

    let decided: Vec<aboutme::SuggestionWithOutcome> = conductor
        .call(&zome(&alice_cell), "get_suggestions", ())
        .await;
    assert!(
        decided
            .iter()
            .find(|s| s.suggestion.action_address() == suggestion.action_address())
            .is_some_and(|s| s.outcome.is_some()),
        "a suggestion just decided must not still look undecided"
    );
}

/// Correcting your own record is not a disagreement with yourself.
///
/// This shipped, and it was found by looking at the screen rather than by any
/// test: writing a record and then editing it once announced "This was changed
/// in 2 places while devices were apart." The count was of every version in
/// the chain, which is two after any ordinary edit. What matters is how many
/// loose ends the chain has, and a chain edited in order has exactly one.
#[tokio::test(flavor = "multi_thread")]
async fn editing_your_own_record_in_order_is_not_a_disagreement() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;

    let created: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;
    let original = created.action_address().clone();

    // Two corrections, one after the other, each on top of the last — which is
    // what a person sitting at one machine actually does.
    let mut head = original.clone();
    for matters in [
        "Seeing my grandchildren",
        "Seeing my grandchildren on Sundays",
    ] {
        let mut edited = an_about_me("Alice Bell");
        edited.what_matters_to_me = matters.into();

        let updated: Record = conductor
            .call(
                &zome(&alice_cell),
                "update_about_me",
                aboutme::UpdateAboutMeInput {
                    original_action_hash: original.clone(),
                    previous_action_hash: head.clone(),
                    about_me: edited,
                },
            )
            .await;
        head = updated.action_address().clone();
    }

    let current: aboutme::CurrentAboutMe = conductor
        .call(&zome(&alice_cell), "get_current_about_me", original)
        .await;

    assert_eq!(
        current.divergent_versions, 1,
        "three versions in a row are one account of a person, not three competing ones"
    );
}

/// And the case the warning exists for, which must still be caught.
#[tokio::test(flavor = "multi_thread")]
async fn two_edits_from_the_same_starting_point_do_disagree() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;

    let created: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;
    let original = created.action_address().clone();

    // Both updates name the original as what they replace: neither knew about
    // the other. This is what two devices apart produce.
    for matters in ["At home if possible", "With her sister"] {
        let mut edited = an_about_me("Alice Bell");
        edited.what_matters_to_me = matters.into();

        let _: Record = conductor
            .call(
                &zome(&alice_cell),
                "update_about_me",
                aboutme::UpdateAboutMeInput {
                    original_action_hash: original.clone(),
                    previous_action_hash: original.clone(),
                    about_me: edited,
                },
            )
            .await;
    }

    let current: aboutme::CurrentAboutMe = conductor
        .call(&zome(&alice_cell), "get_current_about_me", original)
        .await;

    assert_eq!(
        current.divergent_versions, 2,
        "two versions written from the same starting point are a real fork, and          somebody should be told rather than shown one of them silently"
    );
}

// ---------------------------------------------------------------------------
// A second yes: circles that ask two people to agree
// ---------------------------------------------------------------------------
//
// The safeguarding case is not somebody breaking in. It is the holder being
// talked into letting a person in — a plausible caller, a new "friend", a
// relative nobody trusts. The holder is the one under that pressure, so a rule
// the holder can waive alone is not a safeguard at all.
//
// Compare RIX Multi Me's "Buddy", who can veto a share. This is the other way
// round and stronger: nothing happens unless the second person actively
// agrees. A veto has to arrive in time to stop something already moving; a
// signature that was never given stops nothing, because nothing started.

/// Everything a circle with a second yes needs: the founder, the person who
/// must also agree, somebody to invite, and a lobby for the seconder to sign
/// in — which is how the real app does it, and why the seconder never has to
/// be a member of the circle at all.
async fn a_circle_that_asks_two_people() -> (
    SweetConductor,
    CellId, // the founder's cell
    CellId, // the seconder's lobby cell
    DnaFile,
    AgentPubKey, // somebody waiting to be invited
    ActionHash,  // the appointment asking Ruth to agree
) {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let ruth = SweetAgents::one(conductor.keystore()).await;
    let bob = SweetAgents::one(conductor.keystore()).await;

    let dna = circle_dna_with_seconder(&alice, Some(&ruth)).await;
    let alice_cell = join(&conductor, "alice", &alice, &dna, None)
        .await
        .expect("the founder needs no invitation to her own circle");

    // Asked, now that circles carry the rule rather than the person. She
    // does not have to be in the circle to be asked — an appointment names
    // a key, and Alice has hers.
    let appointment: Record = conductor
        .call(&zome(&alice_cell), "appoint", ruth.to_string())
        .await;
    let appointment = appointment.action_address().clone();

    let lobby = lobby_dna().await;
    let ruth_lobby = join(&conductor, "ruth-lobby", &ruth, &lobby, None)
        .await
        .expect("anyone may enter the lobby");

    (conductor, alice_cell, ruth_lobby, dna, bob, appointment)
}

/// One signature is not enough where the circle asks for two.
#[tokio::test(flavor = "multi_thread")]
async fn half_an_invitation_opens_nothing() {
    let (conductor, alice_cell, _ruth_lobby, dna, bob, _appointment) =
        a_circle_that_asks_two_people().await;

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    assert!(
        bundle.invitation.seconded.is_none(),
        "inviting somebody leaves the holder with only her own signature on it"
    );
    assert!(
        bundle.seconder.is_some(),
        "and the invitation says who else must agree, because the joiner needs \
         that to compute the same circle at all"
    );

    let refused = join(&conductor, "bob", &bob, &dna, Some(&bundle.invitation)).await;
    assert!(
        refused.is_err(),
        "the holder's signature alone must not open a circle that asks for two"
    );
}

/// Two signatures do.
#[tokio::test(flavor = "multi_thread")]
async fn two_people_agreeing_lets_somebody_in() {
    let (conductor, alice_cell, ruth_lobby, dna, bob, appointment) =
        a_circle_that_asks_two_people().await;

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    // Ruth signs from the lobby, never having joined the circle: she is
    // agreeing to who gets in without being able to read a word of it.
    let seconded: Signature = conductor
        .call(
            &zome(&ruth_lobby),
            "second_an_invitation",
            aboutme::SecondInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    let invitation = Invitation {
        signature: bundle.invitation.signature.clone(),
        seconded: Some(seconded),
        // Named, or the door has no second agreement to check and would let
        // Bob in on Alice's signature alone — which would make this test pass
        // without proving anything about Ruth at all.
        appointment: Some(appointment),
        name: String::new(),
    };

    assert!(
        join(&conductor, "bob", &bob, &dna, Some(&invitation))
            .await
            .is_ok(),
        "with both agreements the door opens"
    );
}

/// The person who must agree does not have to agree to themselves.
///
/// Without this the feature eats its own tail: the one person who has to
/// approve every arrival cannot arrive, because approving their own way in
/// would mean doing it from outside a circle they are not in yet. They come in
/// on the holder's invitation alone, and from then on nobody else does.
#[tokio::test(flavor = "multi_thread")]
async fn the_second_yes_needs_no_second_yes_of_their_own() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let ruth = SweetAgents::one(conductor.keystore()).await;

    let dna = circle_dna_with_seconder(&alice, Some(&ruth)).await;
    let alice_cell = join(&conductor, "alice", &alice, &dna, None)
        .await
        .expect("the founder needs no invitation to her own circle");

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: ruth.to_string(),
                name: String::new(),
            },
        )
        .await;

    assert!(
        bundle.invitation.seconded.is_none(),
        "an ordinary invitation, with only the holder's signature on it"
    );

    assert!(
        join(&conductor, "ruth", &ruth, &dna, Some(&bundle.invitation))
            .await
            .is_ok(),
        "the person the circle asks must be able to get into it"
    );
}

/// The holder cannot be both people. That is the entire point.
#[tokio::test(flavor = "multi_thread")]
async fn the_holder_cannot_give_the_second_yes_herself() {
    let (conductor, alice_cell, _ruth_lobby, dna, bob, appointment) =
        a_circle_that_asks_two_people().await;

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    // Alice signs a second time, from her own cell, trying to be both
    // signatures. This is the attack the feature exists for: the holder under
    // pressure, waiving her own safeguard.
    let forged: Signature = conductor
        .call(
            &zome(&alice_cell),
            "second_an_invitation",
            aboutme::SecondInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    let invitation = Invitation {
        signature: bundle.invitation.signature.clone(),
        seconded: Some(forged),
        // Naming the appointment is what gives the door something to check
        // her forged signature against. Without it there is no second
        // agreement being claimed at all, and nothing to catch.
        appointment: Some(appointment),
        name: String::new(),
    };

    assert!(
        join(&conductor, "bob", &bob, &dna, Some(&invitation))
            .await
            .is_err(),
        "a safeguard the holder can waive alone is not a safeguard"
    );
}

/// **Why the appointment has to travel with the invitation.**
///
/// The door checks the second agreement only against the appointment an
/// invitation names. Take the name off and there is nothing to check it
/// against, so the same forged invitation the test above refuses walks in.
///
/// That is the rule as written, not a fault in it: an invitation naming no
/// appointment is the holder admitting somebody on her own, which is allowed
/// and meant to be seen. The fault was in the app, whose waiting room packed
/// invitations into a line of text and left the appointment out — so every
/// invitation collected from a door arrived looking like that, including the
/// honest ones two people had agreed to. This pins down what that cost.
#[tokio::test(flavor = "multi_thread")]
async fn an_invitation_that_loses_its_appointment_is_not_checked_for_a_second_yes() {
    let (conductor, alice_cell, _ruth_lobby, dna, bob, _appointment) =
        a_circle_that_asks_two_people().await;

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;
    assert!(
        bundle.invitation.appointment.is_some(),
        "the invitation leaves the zome naming the appointment"
    );

    let forged: Signature = conductor
        .call(
            &zome(&alice_cell),
            "second_an_invitation",
            aboutme::SecondInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    // What the waiting room used to hand over: both signatures, no appointment.
    let stripped = Invitation {
        signature: bundle.invitation.signature.clone(),
        seconded: Some(forged),
        appointment: None,
        name: String::new(),
    };

    assert!(
        join(&conductor, "bob", &bob, &dna, Some(&stripped))
            .await
            .is_ok(),
        "with the appointment gone the second signature is never looked at, \
         so the holder signing twice gets somebody in"
    );
}

/// A circle that asks nobody carries on working exactly as it did.
#[tokio::test(flavor = "multi_thread")]
async fn a_circle_with_no_second_yes_is_unchanged() {
    let (conductor, _alice_cell, bob_cell) = a_circle_with_a_member().await;

    let seconder: Option<AgentPubKey> = conductor
        .call(&zome(&bob_cell), "who_seconds_here", ())
        .await;

    assert!(
        seconder.is_none(),
        "most circles ask one person, and must not start asking two by accident"
    );
}

/// The central red-team finding.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_write_the_persons_about_me() {
    let (conductor, _, bob_cell) = a_circle_with_a_member().await;

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;

    assert!(
        result.is_err(),
        "membership must not confer the right to author somebody else's account of themselves"
    );
}

/// The hole the review missed: link creation was unvalidated, so a member could
/// point the person's own update chain at an entry of their own. Readers follow
/// that chain, so the person's record would show the impostor's content without
/// the person's entry ever being touched.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_hijack_the_update_chain() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let alice_record: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;
    let alice_original = alice_record.action_address().clone();

    // Bob cannot even create an About Me to point at any more, so the hijack
    // fails at the first step. Belt and braces: try to update Alice's record
    // directly as well.
    let forged: Result<Record, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "update_about_me",
            aboutme::UpdateAboutMeInput {
                original_action_hash: alice_original.clone(),
                previous_action_hash: alice_original,
                about_me: an_about_me("Alice Bell"),
            },
        )
        .await;

    assert!(
        forged.is_err(),
        "a member must not be able to extend the person's update chain"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_delete_the_persons_record() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let alice_record: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;

    let result: Result<ActionHash, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "delete_about_me",
            alice_record.action_address().clone(),
        )
        .await;

    assert!(result.is_err(), "only the author of a record may delete it");
}

// ---------------------------------------------------------------------------
// Acknowledgements
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn a_member_can_acknowledge_the_record() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let alice_record: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;

    let _ack: Record = conductor
        .call(
            &zome(&bob_cell),
            "acknowledge",
            aboutme::AcknowledgeInput {
                about_me: alice_record.action_address().clone(),
                role: "district nurse".to_string(),
            },
        )
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn nobody_can_acknowledge_their_own_record() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;

    let alice_record: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&alice_cell),
            "acknowledge",
            aboutme::AcknowledgeInput {
                about_me: alice_record.action_address().clone(),
                role: "myself".to_string(),
            },
        )
        .await;

    assert!(
        result.is_err(),
        "a self-acknowledgement would make the evidence worthless"
    );
}

// ---------------------------------------------------------------------------
// Configuration must fail closed
// ---------------------------------------------------------------------------

/// The most dangerous thing that could go wrong quietly: a circle shipped with
/// no founder configured, standing wide open.
///
/// Absence of configuration must never mean absence of a membrane.
#[tokio::test(flavor = "multi_thread")]
async fn a_circle_with_no_founder_admits_nobody() {
    let conductor = SweetConductor::standard().await;
    let mallory = SweetAgents::one(conductor.keystore()).await;

    let dna = SweetDnaFile::from_bundle_with_overrides(
        &dna_path(),
        DnaModifiersOpt::none().with_properties(CircleProperties {
            founder: None,
            seconder: None,
            requires_second_yes: false,
            waiting_for: None,
            lobby: false,
        }),
    )
    .await
    .unwrap();

    let result = join(&conductor, "mallory", &mallory, &dna, None).await;
    assert!(
        result.is_err(),
        "a circle with no founder must be closed to everybody, not open to everybody"
    );
}

/// A founder that is present but not a valid key is a misconfiguration, and
/// must be treated as one rather than falling through to an open circle.
#[tokio::test(flavor = "multi_thread")]
async fn a_circle_with_a_malformed_founder_admits_nobody() {
    let conductor = SweetConductor::standard().await;
    let mallory = SweetAgents::one(conductor.keystore()).await;

    let dna = SweetDnaFile::from_bundle_with_overrides(
        &dna_path(),
        DnaModifiersOpt::none().with_properties(CircleProperties {
            founder: Some("not-an-agent-key".to_string()),
            seconder: None,
            requires_second_yes: false,
            waiting_for: None,
            lobby: false,
        }),
    )
    .await
    .unwrap();

    let result = join(&conductor, "mallory", &mallory, &dna, None).await;
    assert!(
        result.is_err(),
        "a malformed founder must close the circle, not open it"
    );
}

// ---------------------------------------------------------------------------
// Entry content
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn an_about_me_needs_a_display_name() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;

    let mut blank = an_about_me("");
    blank.display_name = "   ".to_string();

    let result: Result<Record, _> = conductor
        .call_fallible(&zome(&alice_cell), "create_about_me", blank)
        .await;

    assert!(
        result.is_err(),
        "a record nobody is named in is not a record of anybody"
    );
}

// ---------------------------------------------------------------------------
// Circles are separate networks, not separate labels
// ---------------------------------------------------------------------------

/// A circle is a cloned cell whose properties name its holder. Because
/// properties form part of the DNA hash, two holders' circles are different
/// networks — which is what makes "one circle per person" a fact about the
/// maths rather than an access rule someone could get wrong.
#[tokio::test(flavor = "multi_thread")]
async fn two_holders_circles_are_different_networks() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let hers: ClonedCell = conductor
        .call(
            &zome(&alice_cell),
            "create_circle",
            aboutme::CreateCircleInput {
                founder: alice_cell.agent_pubkey().to_string(),
                name: "Alice".to_string(),
                network_seed: "shared-seed".to_string(),
                requires_second_yes: false,
            },
        )
        .await;

    // Same seed, same everything except who holds it.
    let his: ClonedCell = conductor
        .call(
            &zome(&bob_cell),
            "create_circle",
            aboutme::CreateCircleInput {
                founder: bob_cell.agent_pubkey().to_string(),
                name: "Bob".to_string(),
                network_seed: "shared-seed".to_string(),
                requires_second_yes: false,
            },
        )
        .await;

    assert_ne!(
        hers.cell_id.dna_hash(),
        his.cell_id.dna_hash(),
        "circles with different holders must be different networks, even with the same seed"
    );
    assert_ne!(
        hers.cell_id.dna_hash(),
        alice_cell.dna_hash(),
        "a circle must be its own network, not the cell it was cloned from"
    );
}

/// You cannot conjure a circle in somebody else's name.
///
/// Creating a circle means joining it, and joining runs the membrane check. An
/// agent who names someone else as holder, and has no invitation from them, is
/// refused at genesis.
///
/// **This is what settles the proxy question.** The holder is whoever will
/// administer the circle — a daughter, a case manager — not necessarily the
/// person the record describes. Who the record is *about* is content;
/// who holds it is the membrane.
#[tokio::test(flavor = "multi_thread")]
async fn nobody_can_create_a_circle_in_another_persons_name() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;
    let someone_else = SweetAgents::one(conductor.keystore()).await;

    let result: Result<ClonedCell, _> = conductor
        .call_fallible(
            &zome(&alice_cell),
            "create_circle",
            aboutme::CreateCircleInput {
                founder: someone_else.to_string(),
                name: "Not mine to make".to_string(),
                network_seed: "seed".to_string(),
                requires_second_yes: false,
            },
        )
        .await;

    assert!(
        result.is_err(),
        "naming someone else as holder must not let you into the circle you just made"
    );
}

/// The same person can hold more than one circle, kept apart by the seed.
#[tokio::test(flavor = "multi_thread")]
async fn one_person_can_have_separate_circles() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;
    let alice = alice_cell.agent_pubkey().clone();

    let mut hashes = Vec::new();
    for seed in ["care", "respite"] {
        let cell: ClonedCell = conductor
            .call(
                &zome(&alice_cell),
                "create_circle",
                aboutme::CreateCircleInput {
                    founder: alice.to_string(),
                    name: seed.to_string(),
                    network_seed: seed.to_string(),
                    requires_second_yes: false,
                },
            )
            .await;
        hashes.push(cell.cell_id.dna_hash().clone());
    }

    assert_ne!(
        hashes[0], hashes[1],
        "a different seed must give a different network"
    );
}

// ---------------------------------------------------------------------------
// The lobby
// ---------------------------------------------------------------------------

/// A lobby whose only job is to exist so the app can be installed and clone
/// real circles from it.
async fn lobby_dna() -> DnaFile {
    SweetDnaFile::from_bundle_with_overrides(
        &dna_path(),
        DnaModifiersOpt::none().with_properties(CircleProperties {
            founder: None,
            lobby: true,
            seconder: None,
            requires_second_yes: false,
            // A plain lobby, not a door to anywhere. Nobody may write in it,
            // and that includes knocking.
            waiting_for: None,
        }),
    )
    .await
    .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn anyone_may_enter_the_lobby() {
    let conductor = SweetConductor::standard().await;
    let anyone = SweetAgents::one(conductor.keystore()).await;

    join(&conductor, "anyone", &anyone, &lobby_dna().await, None)
        .await
        .expect("the lobby exists to be installable by anyone");
}

/// Everybody who installs the app shares the lobby, so it must hold nothing.
/// Joining it is unrestricted precisely because there is nothing to reach.
#[tokio::test(flavor = "multi_thread")]
async fn nobody_may_write_in_the_lobby() {
    let conductor = SweetConductor::standard().await;
    let anyone = SweetAgents::one(conductor.keystore()).await;
    let cell = join(&conductor, "anyone", &anyone, &lobby_dna().await, None)
        .await
        .unwrap();

    let result: Result<Record, _> = conductor
        .call_fallible(&zome(&cell), "create_about_me", an_about_me("Anybody"))
        .await;

    assert!(
        result.is_err(),
        "a shared lobby that accepts writes is a shared database of strangers"
    );
}

/// The lobby's actual purpose: cloning a real circle out of it.
#[tokio::test(flavor = "multi_thread")]
async fn a_circle_can_be_cloned_from_the_lobby() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let lobby = join(&conductor, "alice", &alice, &lobby_dna().await, None)
        .await
        .unwrap();

    let circle: ClonedCell = conductor
        .call(
            &zome(&lobby),
            "create_circle",
            aboutme::CreateCircleInput {
                founder: alice.to_string(),
                name: "Alice".to_string(),
                network_seed: "seed".to_string(),
                requires_second_yes: false,
            },
        )
        .await;

    assert_ne!(
        circle.cell_id.dna_hash(),
        lobby.dna_hash(),
        "a circle must be its own network, not the lobby it came from"
    );

    // And unlike the lobby, the circle accepts the person's own record.
    let _: Record = conductor
        .call(
            &zome(&circle.cell_id),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;
}

// ---------------------------------------------------------------------------
// Signals: telling the holder without polling
// ---------------------------------------------------------------------------

/// The thing families currently have no way of knowing: whether anybody read
/// it. No polling, no server, no notification service — the professional's
/// She sends an invitation and then has no way of knowing whether it worked.
///
/// Everything arrived correctly and the screen said nothing, so the only way
/// to find out whether a nephew had joined was to ring him up. The list of
/// people is the record of who is here; this signal only saves her going to
/// look at it.
#[tokio::test(flavor = "multi_thread")]
async fn the_holder_is_told_when_her_invitation_is_taken_up() {
    let (conductor, _, bob_cell) = a_circle_with_a_member().await;

    let mut alice_hears = conductor.subscribe_to_app_signals("alice".to_string());

    let _: Record = conductor
        .call(
            &zome(&bob_cell),
            "introduce_myself",
            aboutme_integrity::Member {
                name: "Dave Smythe".to_string(),
                relationship: "her nephew".to_string(),
            },
        )
        .await;

    let signal = tokio::time::timeout(std::time::Duration::from_secs(60), alice_hears.recv())
        .await
        .expect("the person who sent the invitation should be told")
        .expect("the signal channel should stay open");

    match signal {
        Signal::App { signal, .. } => {
            let decoded: aboutme::Signal = signal
                .into_inner()
                .decode()
                .expect("the signal should be one of ours");
            let aboutme::Signal::Introduced {
                by,
                name,
                relationship,
                joined,
            } = decoded
            else {
                panic!("expected an Introduced signal, got {decoded:?}");
            };
            assert_eq!(by, *bob_cell.agent_pubkey());
            assert_eq!(name, "Dave Smythe");
            assert_eq!(relationship, "her nephew", "in his own words, unchecked");
            assert!(
                joined,
                "the first time somebody speaks up, they have joined"
            );
        }
        other => panic!("expected an app signal, got {other:?}"),
    }
}

/// And correcting yourself is not joining twice.
#[tokio::test(flavor = "multi_thread")]
async fn correcting_your_own_introduction_is_not_another_arrival() {
    let (conductor, _, bob_cell) = a_circle_with_a_member().await;

    let _: Record = conductor
        .call(
            &zome(&bob_cell),
            "introduce_myself",
            aboutme_integrity::Member {
                name: "Dave Smyth".to_string(),
                relationship: "her nephew".to_string(),
            },
        )
        .await;

    // Subscribed only now, so the first introduction cannot be mistaken for
    // the one under test.
    let mut alice_hears = conductor.subscribe_to_app_signals("alice".to_string());

    let _: Record = conductor
        .call(
            &zome(&bob_cell),
            "introduce_myself",
            aboutme_integrity::Member {
                name: "Dave Smythe".to_string(),
                relationship: "her nephew".to_string(),
            },
        )
        .await;

    let signal = tokio::time::timeout(std::time::Duration::from_secs(60), alice_hears.recv())
        .await
        .expect("she should still be told")
        .expect("the signal channel should stay open");

    match signal {
        Signal::App { signal, .. } => {
            let decoded: aboutme::Signal = signal
                .into_inner()
                .decode()
                .expect("the signal should be one of ours");
            let aboutme::Signal::Introduced { joined, name, .. } = decoded else {
                panic!("expected an Introduced signal, got {decoded:?}");
            };
            assert_eq!(name, "Dave Smythe");
            assert!(
                !joined,
                "somebody fixing the spelling of their own name has not arrived again"
            );
        }
        other => panic!("expected an app signal, got {other:?}"),
    }
}

/// device tells the holder's device directly.
#[tokio::test(flavor = "multi_thread")]
async fn the_holder_is_told_when_someone_reads_the_record() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let mut alice_hears = conductor.subscribe_to_app_signals("alice".to_string());

    let record: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;

    let _: Record = conductor
        .call(
            &zome(&bob_cell),
            "acknowledge",
            aboutme::AcknowledgeInput {
                about_me: record.action_address().clone(),
                role: "district nurse".to_string(),
            },
        )
        .await;

    let signal = tokio::time::timeout(std::time::Duration::from_secs(60), alice_hears.recv())
        .await
        .expect("the holder should be told within a reasonable time")
        .expect("the signal channel should stay open");

    match signal {
        Signal::App { signal, .. } => {
            let decoded: aboutme::Signal = signal
                .into_inner()
                .decode()
                .expect("the signal should be one of ours");
            let aboutme::Signal::Acknowledged { by, role, .. } = decoded else {
                panic!("expected an Acknowledged signal, got {decoded:?}");
            };
            assert_eq!(by, *bob_cell.agent_pubkey(), "the reader should be named");
            assert_eq!(role, "district nurse", "the claimed role travels with it");
        }
        other => panic!("expected an app signal, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Suggestions: everyone contributes, one voice remains
// ---------------------------------------------------------------------------

fn a_suggestion() -> aboutme_integrity::Suggestion {
    aboutme_integrity::Suggestion {
        field: aboutme_integrity::AboutMeField::WhatMattersToMe,
        text: "Her allotment. She talked about it all summer.".to_string(),
        because: "I am her son.".to_string(),
        locked: None,
    }
}

/// The whole point. A member who may not write the record may still offer
/// something for it.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_can_offer_a_suggestion() {
    let (conductor, _, bob_cell) = a_circle_with_a_member().await;

    let record: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    assert!(record.action().entry_hash().is_some());
}

/// And the holder sees it.
#[tokio::test(flavor = "multi_thread")]
async fn the_holder_sees_what_was_offered() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let _: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    let offered: Vec<aboutme::SuggestionWithOutcome> = conductor
        .call(&zome(&alice_cell), "get_suggestions", ())
        .await;

    assert_eq!(offered.len(), 1, "the holder should see the suggestion");
    assert!(
        offered[0].outcome.is_none(),
        "an undecided suggestion has no outcome yet"
    );
}

/// A suggestion is an offer, not an edit. Only the holder decides.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_decide_on_a_suggestion() {
    let (conductor, _, bob_cell) = a_circle_with_a_member().await;

    let offered: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "decide_on_suggestion",
            aboutme::DecideInput {
                suggestion: offered.action_address().clone(),
                accepted: true,
            },
        )
        .await;

    assert!(
        result.is_err(),
        "offering something must not be the same as putting it in the record"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_holder_can_decide_and_the_decision_is_kept() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let offered: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    let _: Record = conductor
        .call(
            &zome(&alice_cell),
            "decide_on_suggestion",
            aboutme::DecideInput {
                suggestion: offered.action_address().clone(),
                // Set aside rather than accepted, because that is the case
                // that must not disappear.
                accepted: false,
            },
        )
        .await;

    let after: Vec<aboutme::SuggestionWithOutcome> = conductor
        .call(&zome(&alice_cell), "get_suggestions", ())
        .await;

    assert_eq!(after.len(), 1, "a suggestion set aside is still there");
    assert!(
        after[0].outcome.is_some(),
        "and what became of it is recorded, not silently dropped"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_change_somebody_elses_suggestion() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let offered: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    let mut altered = a_suggestion();
    altered.text = "Something Bob never said.".to_string();

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&alice_cell),
            "update_suggestion",
            (offered.action_address().clone(), altered),
        )
        .await;

    assert!(
        result.is_err(),
        "nobody may put words in another member's mouth"
    );
}

// ---------------------------------------------------------------------------
// The whole journey
// ---------------------------------------------------------------------------

/// Everything a real circle does, in order.
///
/// The individual rules are tested above. This exists because the parts can
/// all be right while the journey is broken — which is exactly what happened
/// by hand: the record was written, its reference reached the joiner, and the
/// content did not follow. Nothing above would have caught that.
#[tokio::test(flavor = "multi_thread")]
async fn the_whole_journey() {
    let (conductor, alice, bob) = a_circle_with_a_member().await;

    // 1. The person's record.
    let written: Record = conductor
        .call(
            &zome(&alice),
            "create_about_me",
            an_about_me("Margaret Smythe"),
        )
        .await;
    let original = written.action_address().clone();

    // 2. It has to actually reach the other member. Gossip carries links and
    //    entries separately, so the reference can arrive without the content —
    //    poll for both, the way a person pressing "Check again" would.
    let mut arrived = None;
    for _ in 0..60 {
        let originals: Vec<ActionHash> =
            conductor.call(&zome(&bob), "get_circle_about_me", ()).await;

        if let Some(found) = originals.first() {
            let current: aboutme::CurrentAboutMe = conductor
                .call(&zome(&bob), "get_current_about_me", found.clone())
                .await;
            if current.record.is_some() {
                arrived = current.record;
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    assert!(
        arrived.is_some(),
        "the record must reach the other member — the reference and the content"
    );

    // 3. Bob says who he is. Nobody verifies it.
    let _: Record = conductor
        .call(
            &zome(&bob),
            "introduce_myself",
            aboutme_integrity::Member {
                name: "Gareth".to_string(),
                relationship: "her son".to_string(),
            },
        )
        .await;

    // 4. He remembers something she enjoyed. He cannot write it himself.
    let offered: Record = conductor.call(&zome(&bob), "suggest", a_suggestion()).await;

    // 5. The holder sees it and decides.
    let waiting: Vec<aboutme::SuggestionWithOutcome> =
        conductor.call(&zome(&alice), "get_suggestions", ()).await;
    assert_eq!(waiting.len(), 1, "the holder should see what was offered");

    let _: Record = conductor
        .call(
            &zome(&alice),
            "decide_on_suggestion",
            aboutme::DecideInput {
                suggestion: offered.action_address().clone(),
                accepted: true,
            },
        )
        .await;

    // 6. And Bob marks that he has read it: the whole professional workflow.
    let _: Record = conductor
        .call(
            &zome(&bob),
            "acknowledge",
            aboutme::AcknowledgeInput {
                about_me: original.clone(),
                role: "her son".to_string(),
            },
        )
        .await;

    let readers: Vec<aboutme::WhoRead> = conductor
        .call(&zome(&alice), "get_acknowledgements", original)
        .await;
    assert_eq!(readers.len(), 1, "the holder should know it was read");
    assert_eq!(
        readers[0].role, "her son",
        "and what he said he was, which is locked in the circle and opened here"
    );
    assert!(!readers[0].locked_out);
}

// ---------------------------------------------------------------------------
// Signals: a nudge that cannot lie about who it is from
// ---------------------------------------------------------------------------
//
// A signal is not evidence. The acknowledgement written to the chain is the
// evidence, and a signal only saves somebody going to look.
//
// But the interface acts on one directly — it puts a sentence on the screen
// naming a person — so it has to be true. Every signal says who it is from, and
// that field is filled in by whoever sent it. Until it was checked against who
// actually called, any member of a circle could make the holder's screen say
// that a district nurse had read the record when nobody had.

/// Claiming to be somebody else gets you nothing.
#[tokio::test(flavor = "multi_thread")]
async fn a_signal_that_names_the_wrong_sender_is_dropped() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let mut bob_hears = conductor.subscribe_to_app_signals("bob".to_string());

    // Bob announces that Alice has offered something. She has not.
    let _: () = conductor
        .call(
            &zome(&bob_cell),
            "recv_remote_signal",
            aboutme::Signal::Suggested {
                suggestion: fake_hash(),
                by: alice_cell.agent_pubkey().clone(),
                text: "Words Alice never wrote.".to_string(),
            },
        )
        .await;

    let heard = tokio::time::timeout(std::time::Duration::from_secs(5), bob_hears.recv()).await;

    assert!(
        heard.is_err(),
        "a signal claiming to be from somebody else must reach no screen"
    );
}

/// And the honest case still works, so the check above is not simply a wall.
#[tokio::test(flavor = "multi_thread")]
async fn a_signal_from_the_person_it_names_is_delivered() {
    let (conductor, _alice_cell, bob_cell) = a_circle_with_a_member().await;

    let mut bob_hears = conductor.subscribe_to_app_signals("bob".to_string());

    let _: () = conductor
        .call(
            &zome(&bob_cell),
            "recv_remote_signal",
            aboutme::Signal::Suggested {
                suggestion: fake_hash(),
                by: bob_cell.agent_pubkey().clone(),
                text: "Her allotment.".to_string(),
            },
        )
        .await;

    let signal = tokio::time::timeout(std::time::Duration::from_secs(60), bob_hears.recv())
        .await
        .expect("a signal that names its real sender should arrive")
        .expect("the signal channel should stay open");

    match signal {
        Signal::App { signal, .. } => {
            let decoded: aboutme::Signal = signal
                .into_inner()
                .decode()
                .expect("the signal should be one of ours");
            let aboutme::Signal::Suggested { by, .. } = decoded else {
                panic!("expected a Suggested signal, got {decoded:?}");
            };
            assert_eq!(by, *bob_cell.agent_pubkey());
        }
        other => panic!("expected an app signal, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Everybody must agree which one is the current one
// ---------------------------------------------------------------------------
//
// Lists here are assembled from links, and the order links arrive in is not
// promised to be the same on any two machines. Anywhere a reader takes the last
// of something as the one that counts, that order has to be imposed rather than
// inherited — see `order_versions` in the integrity crate, which says the same
// thing about versions of a record.

/// Correcting how you describe yourself, and having the correction be the one
/// that shows.
#[tokio::test(flavor = "multi_thread")]
async fn the_latest_introduction_is_the_one_that_counts() {
    let (conductor, _alice_cell, bob_cell) = a_circle_with_a_member().await;

    for name in ["Dave Smyth", "Dave Smythe"] {
        let _: Record = conductor
            .call(
                &zome(&bob_cell),
                "introduce_myself",
                aboutme_integrity::Member {
                    name: name.to_string(),
                    relationship: "her nephew".to_string(),
                },
            )
            .await;
    }

    let members: Vec<Record> = conductor.call(&zome(&bob_cell), "get_members", ()).await;

    // A reader folds these into a map keyed by author, so the last one wins.
    // Both of Bob's are here; the corrected spelling has to be the later.
    let mine: Vec<String> = members
        .iter()
        .filter(|r| r.action().author() == bob_cell.agent_pubkey())
        .filter_map(|r| {
            r.entry()
                .to_app_option::<aboutme_integrity::Member>()
                .ok()
                .flatten()
        })
        .map(|m| m.name)
        .collect();

    assert_eq!(
        mine,
        vec!["Dave Smyth".to_string(), "Dave Smythe".to_string()],
        "introductions must come back oldest first, so a correction is last"
    );
}

/// Changing your mind about a suggestion, and having the change be the one that
/// shows.
#[tokio::test(flavor = "multi_thread")]
async fn the_latest_decision_is_the_one_that_counts() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let offered: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    // Set aside, and then thought better of.
    for accepted in [false, true] {
        let _: Record = conductor
            .call(
                &zome(&alice_cell),
                "decide_on_suggestion",
                aboutme::DecideInput {
                    suggestion: offered.action_address().clone(),
                    accepted,
                },
            )
            .await;
    }

    let after: Vec<aboutme::SuggestionWithOutcome> = conductor
        .call(&zome(&alice_cell), "get_suggestions", ())
        .await;

    assert_eq!(after.len(), 1);
    let outcome = after[0]
        .outcome
        .as_ref()
        .expect("a decided suggestion has an outcome")
        .entry()
        .to_app_option::<aboutme_integrity::SuggestionOutcome>()
        .ok()
        .flatten()
        .expect("and it is an outcome");

    assert!(
        outcome.accepted,
        "the decision shown must be the last one made, not the first"
    );
}

// ---------------------------------------------------------------------------
// Configuration that fails closed, in the case that was added last
// ---------------------------------------------------------------------------

/// A circle naming a second person it cannot read admits nobody at all —
/// including the holder.
///
/// The equivalent for a malformed founder has a test above. This one was added
/// afterwards and did not get one, which is exactly how the two would drift
/// apart. Getting it wrong must close the door rather than open it: absence of
/// configuration must never mean absence of a membrane, and neither must a typo
/// in it.
#[tokio::test(flavor = "multi_thread")]
async fn a_circle_with_an_unreadable_second_person_admits_nobody() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;

    let dna = SweetDnaFile::from_bundle_with_overrides(
        &dna_path(),
        DnaModifiersOpt::none().with_properties(CircleProperties {
            founder: Some(alice.to_string()),
            lobby: false,
            // The older way of naming a second person, kept working so that a
            // circle made that way still asks for two agreements. Unreadable
            // still closes the circle rather than being quietly ignored, which
            // is what this test is about.
            seconder: Some("not an identifier at all".to_string()),
            requires_second_yes: false,
            waiting_for: None,
        }),
    )
    .await
    .expect("the packed DNA should load");

    let result = join(&conductor, "alice", &alice, &dna, None).await;

    assert!(
        result.is_err(),
        "a circle that names a second person it cannot read must close, \
         not quietly forget the second person"
    );
}

// ---------------------------------------------------------------------------
// An acknowledgement must be about a record
// ---------------------------------------------------------------------------

/// Reading something is only meaningful about an About Me.
///
/// The hash an acknowledgement points at comes from whoever is acknowledging,
/// so it is the one part of this they choose. Without the check, an
/// acknowledgement could be attached to anything at all and counted as evidence
/// that somebody had read the person's record.
#[tokio::test(flavor = "multi_thread")]
async fn an_acknowledgement_cannot_point_at_something_that_is_not_a_record() {
    let (conductor, _alice_cell, bob_cell) = a_circle_with_a_member().await;

    // A real entry Bob wrote, which is simply not an About Me.
    let offered: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "acknowledge",
            aboutme::AcknowledgeInput {
                about_me: offered.action_address().clone(),
                role: "district nurse".to_string(),
            },
        )
        .await;

    assert!(
        result.is_err(),
        "an acknowledgement must reference an About Me and nothing else"
    );
}

// ---------------------------------------------------------------------------
// The second person is told who they are agreeing to
// ---------------------------------------------------------------------------

/// A key identifies nobody, so the holder says who she thinks it is.
///
/// Without this the person being asked to agree sees a line of base64 and
/// nothing else, and the only thing they can actually agree to is that they
/// were asked — which safeguards nobody. It is her claim and nothing checks
/// it; what it gives the second person is something they can answer.
#[tokio::test(flavor = "multi_thread")]
async fn an_invitation_carries_who_the_holder_says_it_is_for() {
    let (conductor, alice_cell, _bob_cell) = a_circle_with_a_member().await;

    let ronnie = SweetAgents::one(conductor.keystore()).await;

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: ronnie.to_string(),
                name: "  Ronnie Smythe  ".to_string(),
            },
        )
        .await;

    assert_eq!(
        bundle.invitee_name, "Ronnie Smythe",
        "the name travels with the invitation, trimmed"
    );
    assert_eq!(
        bundle.invitee,
        ronnie.to_string(),
        "and it is attached to the key it was given for"
    );
}

/// An invitation with no name still works.
///
/// The name is a help, not a requirement. Refusing to make an invitation
/// without one would turn a courtesy into a gate, and there is nothing to
/// enforce because nothing is checked anyway.
#[tokio::test(flavor = "multi_thread")]
async fn an_invitation_without_a_name_is_still_an_invitation() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let bob = SweetAgents::one(conductor.keystore()).await;
    let dna = circle_dna(&alice).await;

    let alice_cell = join(&conductor, "alice", &alice, &dna, None).await.unwrap();

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    assert_eq!(bundle.invitee_name, "");

    join(&conductor, "bob", &bob, &dna, Some(&bundle.invitation))
        .await
        .expect("a nameless invitation still admits the person it names");
}

// ---------------------------------------------------------------------------
// Two agreements that reach each other inside the circle
// ---------------------------------------------------------------------------
//
// The two agreements used to be carried between devices by hand. They now
// travel as entries, which means two new rules that only hold if something
// rejects when they are broken — and one of them, "only the person this circle
// asks may agree", is the whole of the safeguard.

/// A circle that asks two people, with both of them actually in it.
///
/// The by-hand route did not need the second person inside; this one does,
/// because they read the proposal from their own copy of the circle.
async fn a_circle_with_both_people_in_it() -> (
    SweetConductor,
    CellId,      // the holder
    CellId,      // the second person
    CellId,      // an ordinary member, in the circle and asked nothing
    AgentPubKey, // somebody waiting to be let in
) {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let ruth = SweetAgents::one(conductor.keystore()).await;
    let dave = SweetAgents::one(conductor.keystore()).await;
    let ronnie = SweetAgents::one(conductor.keystore()).await;

    let dna = circle_dna_with_seconder(&alice, Some(&ruth)).await;

    let alice_cell = join(&conductor, "alice", &alice, &dna, None)
        .await
        .expect("the founder needs no invitation to her own circle");

    // The seconder needs no second agreement to their own admission.
    let for_ruth: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: ruth.to_string(),
                name: "Ruth".to_string(),
            },
        )
        .await;
    let ruth_cell = join(&conductor, "ruth", &ruth, &dna, Some(&for_ruth.invitation))
        .await
        .expect("the second person comes in on the holder's invitation alone");

    // And is asked, once she is here. Who agrees is written in the circle now,
    // not baked into its identity, so this is the moment it happens — and it
    // is a moment that can happen again if she is ever unable to answer.
    let _: Record = conductor
        .call(&zome(&alice_cell), "appoint", ruth.to_string())
        .await;

    // An ordinary member, admitted the long way, who is asked nothing.
    let for_dave: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: dave.to_string(),
                name: "Dave".to_string(),
            },
        )
        .await;
    let seconded: Signature = conductor
        .call(
            &zome(&ruth_cell),
            "second_an_invitation",
            aboutme::SecondInput {
                invitee: dave.to_string(),
                name: "Dave".to_string(),
            },
        )
        .await;
    let dave_invitation = aboutme_integrity::Invitation {
        signature: for_dave.invitation.signature.clone(),
        seconded: Some(seconded),
        appointment: None,
        name: "Dave".to_string(),
    };
    let dave_cell = join(&conductor, "dave", &dave, &dna, Some(&dave_invitation))
        .await
        .expect("two agreements admit an ordinary member");

    keys_flowing(&conductor, &alice_cell, &[&ruth_cell, &dave_cell]).await;

    (conductor, alice_cell, ruth_cell, dave_cell, ronnie)
}

/// The whole point: nobody carries anything, and the result opens the door.
#[tokio::test(flavor = "multi_thread")]
async fn two_agreements_given_in_the_circle_make_a_working_invitation() {
    let (conductor, alice_cell, ruth_cell, _dave_cell, ronnie) =
        a_circle_with_both_people_in_it().await;

    // Alice puts Ronnie forward. Nothing is sent to anybody.
    let _: Record = conductor
        .call(
            &zome(&alice_cell),
            "propose_member",
            aboutme::ProposeInput {
                invitee: ronnie.to_string(),
                name: "Ronnie Smythe, Margaret's cousin".to_string(),
            },
        )
        .await;

    // Ruth reads it from her own copy of the circle.
    let waiting: Vec<aboutme::PendingMember> = conductor
        .call(&zome(&ruth_cell), "get_pending_members", ())
        .await;
    let ronnies = waiting
        .iter()
        .find(|p| p.invitee == ronnie.to_string())
        .expect("the second person sees who has been put forward");
    assert_eq!(ronnies.name, "Ronnie Smythe, Margaret's cousin");
    assert!(!ronnies.agreed, "nobody has agreed yet");
    assert!(
        ronnies.invitation.is_none(),
        "and there is no invitation until somebody has"
    );

    let _: Record = conductor
        .call(&zome(&ruth_cell), "endorse", ronnies.proposed.clone())
        .await;

    // The finished invitation appears on the holder's side, assembled from two
    // agreements neither of them copied anywhere.
    let now: Vec<aboutme::PendingMember> = conductor
        .call(&zome(&alice_cell), "get_pending_members", ())
        .await;
    let finished = now
        .iter()
        .find(|p| p.invitee == ronnie.to_string())
        .expect("still listed");
    assert!(finished.agreed, "the second agreement is recorded");
    let invitation = finished
        .invitation
        .as_ref()
        .expect("both agreements make an invitation");

    // And it actually works, which is the only claim worth making.
    let dna =
        circle_dna_with_seconder(alice_cell.agent_pubkey(), Some(ruth_cell.agent_pubkey())).await;
    join(
        &conductor,
        "ronnie",
        &ronnie,
        &dna,
        Some(&invitation.invitation),
    )
    .await
    .expect("an invitation assembled from two agreements must open the door");
}

/// A member cannot put somebody forward.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_propose_somebody() {
    let (conductor, _alice_cell, _ruth_cell, dave_cell, ronnie) =
        a_circle_with_both_people_in_it().await;

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&dave_cell),
            "propose_member",
            aboutme::ProposeInput {
                invitee: ronnie.to_string(),
                name: "A friend of mine".to_string(),
            },
        )
        .await;

    assert!(
        result.is_err(),
        "only the person whose circle it is may put somebody forward"
    );
}

/// **The rule the whole safeguard rests on.**
///
/// If any member could agree, the second yes would be a second yes from
/// whoever happened to be about — which is not a safeguard, it is a queue.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_give_the_second_agreement() {
    let (conductor, alice_cell, _ruth_cell, dave_cell, ronnie) =
        a_circle_with_both_people_in_it().await;

    let proposal: Record = conductor
        .call(
            &zome(&alice_cell),
            "propose_member",
            aboutme::ProposeInput {
                invitee: ronnie.to_string(),
                name: "Ronnie".to_string(),
            },
        )
        .await;

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&dave_cell),
            "endorse",
            proposal.action_address().clone(),
        )
        .await;

    assert!(
        result.is_err(),
        "only the person this circle asks may give the second agreement"
    );
}

/// And the holder cannot agree with herself.
///
/// The same rule as the by-hand route enforced, kept when the route changed.
/// A safeguard the person under pressure can satisfy alone is not one.
#[tokio::test(flavor = "multi_thread")]
async fn the_holder_cannot_give_the_second_agreement_herself() {
    let (conductor, alice_cell, _ruth_cell, _dave_cell, ronnie) =
        a_circle_with_both_people_in_it().await;

    let proposal: Record = conductor
        .call(
            &zome(&alice_cell),
            "propose_member",
            aboutme::ProposeInput {
                invitee: ronnie.to_string(),
                name: "Ronnie".to_string(),
            },
        )
        .await;

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&alice_cell),
            "endorse",
            proposal.action_address().clone(),
        )
        .await;

    assert!(
        result.is_err(),
        "the holder must not be able to agree with herself"
    );
}

// ---------------------------------------------------------------------------
// Who is asked to agree, and what happens when they cannot
// ---------------------------------------------------------------------------
//
// The second person used to be part of the circle's identity, which made them
// permanent. Now it is an entry the holder writes, which is what lets her
// appoint somebody else the day the first person is past helping. These are
// the rules that keeps honest.

/// Only the holder decides who is asked to agree.
///
/// Without this any member could appoint themselves and then agree to their
/// own arrivals, which is not a safeguard, it is a formality.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_appoint_anybody() {
    let (conductor, _alice_cell, _ruth_cell, dave_cell, ronnie) =
        a_circle_with_both_people_in_it().await;

    let result: Result<Record, _> = conductor
        .call_fallible(&zome(&dave_cell), "appoint", ronnie.to_string())
        .await;

    assert!(
        result.is_err(),
        "only the person whose circle this is may ask somebody to agree"
    );
}

/// And she cannot appoint herself.
///
/// A safeguard the person under pressure can satisfy alone is not one. The
/// whole case this exists for is somebody leaning on her.
#[tokio::test(flavor = "multi_thread")]
async fn the_holder_cannot_appoint_herself() {
    let (conductor, alice_cell, _ruth_cell, _dave_cell, _ronnie) =
        a_circle_with_both_people_in_it().await;

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&alice_cell),
            "appoint",
            alice_cell.agent_pubkey().to_string(),
        )
        .await;

    assert!(
        result.is_err(),
        "the person who agrees to who joins has to be somebody else"
    );
}

/// **The way out of the situation everybody eventually meets.**
///
/// The second person dies, or loses the device their keys were on. Before this
/// the circle could never admit anybody again and the only escape was a new
/// circle with every member joining afresh. Now she appoints somebody else,
/// and the person she appoints can agree.
#[tokio::test(flavor = "multi_thread")]
async fn the_holder_can_appoint_somebody_else_and_the_circle_carries_on() {
    let (conductor, alice_cell, _ruth_cell, dave_cell, ronnie) =
        a_circle_with_both_people_in_it().await;

    // Ruth is past helping. Dave is asked instead.
    let _: Record = conductor
        .call(
            &zome(&alice_cell),
            "appoint",
            dave_cell.agent_pubkey().to_string(),
        )
        .await;

    let asked: Option<AgentPubKey> = conductor
        .call(&zome(&alice_cell), "who_seconds_here", ())
        .await;
    assert_eq!(
        asked.as_ref(),
        Some(dave_cell.agent_pubkey()),
        "the newest appointment is the one in force"
    );

    // And it works end to end: Alice puts Ronnie forward, Dave agrees, and the
    // invitation that comes out opens the door.
    let _: Record = conductor
        .call(
            &zome(&alice_cell),
            "propose_member",
            aboutme::ProposeInput {
                invitee: ronnie.to_string(),
                name: "Ronnie".to_string(),
            },
        )
        .await;

    let waiting: Vec<aboutme::PendingMember> = conductor
        .call(&zome(&dave_cell), "get_pending_members", ())
        .await;
    let theirs = waiting
        .iter()
        .find(|p| p.invitee == ronnie.to_string())
        .expect("the newly appointed person sees who has been put forward");

    let _: Record = conductor
        .call(&zome(&dave_cell), "endorse", theirs.proposed.clone())
        .await;

    let now: Vec<aboutme::PendingMember> = conductor
        .call(&zome(&alice_cell), "get_pending_members", ())
        .await;
    let invitation = now
        .iter()
        .find(|p| p.invitee == ronnie.to_string())
        .and_then(|p| p.invitation.as_ref())
        .expect("the replacement's agreement makes a whole invitation");

    let dna = circle_dna_with_seconder(alice_cell.agent_pubkey(), Some(&ronnie)).await;
    join(
        &conductor,
        "ronnie",
        &ronnie,
        &dna,
        Some(&invitation.invitation),
    )
    .await
    .expect("a circle whose second person was replaced still admits people");
}

/// Somebody who has been replaced cannot give new agreements.
///
/// The point of being able to replace them is that replacing them means
/// something. Note what this does *not* claim: an agreement Ruth already gave
/// stays good, and an invitation built from it still opens the door. That is
/// deliberate — an agreement is a thing somebody did at a moment, and moments
/// do not become undone. What changes is that she cannot give another.
#[tokio::test(flavor = "multi_thread")]
async fn a_replaced_second_person_can_no_longer_agree_to_anybody_new() {
    let (conductor, alice_cell, ruth_cell, dave_cell, ronnie) =
        a_circle_with_both_people_in_it().await;

    // Dave is asked instead of Ruth.
    let _: Record = conductor
        .call(
            &zome(&alice_cell),
            "appoint",
            dave_cell.agent_pubkey().to_string(),
        )
        .await;

    let asked: Option<AgentPubKey> = conductor
        .call(&zome(&alice_cell), "who_seconds_here", ())
        .await;
    assert_eq!(
        asked.as_ref(),
        Some(dave_cell.agent_pubkey()),
        "the replacement is the one asked now"
    );

    // Somebody new is put forward, and Ruth tries to agree to them.
    let proposal: Record = conductor
        .call(
            &zome(&alice_cell),
            "propose_member",
            aboutme::ProposeInput {
                invitee: ronnie.to_string(),
                name: "Ronnie".to_string(),
            },
        )
        .await;

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&ruth_cell),
            "endorse",
            proposal.action_address().clone(),
        )
        .await;

    assert!(
        result.is_err(),
        "somebody who has been replaced must not be able to agree to anybody new"
    );
}

// ---------------------------------------------------------------------------
// The waiting room
// ---------------------------------------------------------------------------

/// A waiting room for one circle. Anybody may enter it.
async fn a_waiting_room(holder: &AgentPubKey) -> DnaFile {
    SweetDnaFile::from_bundle_with_overrides(
        &dna_path(),
        DnaModifiersOpt::none()
            .with_network_seed("a-door".to_string())
            .with_properties(CircleProperties {
                founder: None,
                lobby: false,
                seconder: None,
                requires_second_yes: false,
                waiting_for: Some(holder.to_string()),
            }),
    )
    .await
    .expect("the packed DNA should load")
}

/// An invitation as one line of text, the way the room carries it.
fn as_text(invitation: &Invitation) -> String {
    let bytes = SerializedBytes::try_from(invitation.clone())
        .expect("an invitation packs into bytes")
        .bytes()
        .to_vec();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn from_text(text: &str) -> Invitation {
    let bytes: Vec<u8> = (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("hex"))
        .collect();
    Invitation::try_from(SerializedBytes::from(UnsafeBytes::from(bytes)))
        .expect("and unpacks again")
}

/// What somebody says when they knock.
///
/// The words, not the entry. A knock on the DHT is two sealed boxes; the zome
/// makes them, because sealing needs the holder’s key and the keystore, and
/// neither belongs in a test fixture.
fn a_knock() -> aboutme_integrity::WhoIsKnocking {
    aboutme_integrity::WhoIsKnocking {
        name: "Ronnie Smythe".to_string(),
        relationship: "her cousin".to_string(),
    }
}

/// Anybody may knock, without an invitation and without being known.
///
/// This is the whole point of the room. A door where you must already be known
/// in order to ask is the closed door it exists to replace.
#[tokio::test(flavor = "multi_thread")]
async fn anybody_may_knock_at_a_waiting_room() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let ronnie = SweetAgents::one(conductor.keystore()).await;

    let room = a_waiting_room(&alice).await;
    let ronnie_cell = join(&conductor, "ronnie-room", &ronnie, &room, None)
        .await
        .expect("a waiting room is open, so there is nothing to present");

    let _: Record = conductor
        .call(&zome(&ronnie_cell), "knock", a_knock())
        .await;

    let at_the_door: Vec<aboutme::Knocking> =
        conductor.call(&zome(&ronnie_cell), "get_knocks", ()).await;

    assert_eq!(at_the_door.len(), 1);
    assert_eq!(at_the_door[0].name, "Ronnie Smythe");
    assert_eq!(
        at_the_door[0].who,
        ronnie.to_string(),
        "whoever knocked brought their own key by arriving"
    );
    assert!(!at_the_door[0].answered);
}

/// Knocking means nothing in the plain lobby every installation shares.
///
/// Without this rule, "somebody wants to join Margaret Smythe's circle" would
/// be written into a network that every person who installs this app is in.
#[tokio::test(flavor = "multi_thread")]
async fn nobody_may_knock_in_the_ordinary_lobby() {
    let conductor = SweetConductor::standard().await;
    let ronnie = SweetAgents::one(conductor.keystore()).await;

    let lobby = lobby_dna().await;
    let ronnie_cell = join(&conductor, "ronnie-lobby", &ronnie, &lobby, None)
        .await
        .expect("anyone may enter the lobby");

    let result: Result<Record, _> = conductor
        .call_fallible(&zome(&ronnie_cell), "knock", a_knock())
        .await;

    assert!(
        result.is_err(),
        "a knock in the shared lobby would be seen by everybody who installs this app"
    );
}

/// Only the holder of the circle answers knocks at its door.
///
/// A forged answer could admit nobody — the circle's own door still checks the
/// signature — but it would let a stranger hand somebody a thing that looks
/// like a welcome and silently is not.
#[tokio::test(flavor = "multi_thread")]
async fn only_the_holder_may_answer_a_knock() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let ronnie = SweetAgents::one(conductor.keystore()).await;
    let mallory = SweetAgents::one(conductor.keystore()).await;

    let room = a_waiting_room(&alice).await;
    let ronnie_cell = join(&conductor, "ronnie-room", &ronnie, &room, None)
        .await
        .unwrap();
    let mallory_cell = join(&conductor, "mallory-room", &mallory, &room, None)
        .await
        .expect("the room is open to her too, which is the point of testing this");

    let knocked: Record = conductor
        .call(&zome(&ronnie_cell), "knock", a_knock())
        .await;

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&mallory_cell),
            "admit",
            aboutme::AdmitInput {
                knock: knocked.action_address().clone(),
                invitation: "anything at all".to_string(),
            },
        )
        .await;

    assert!(
        result.is_err(),
        "somebody who does not hold the circle must not be able to answer for it"
    );
}

/// A room naming a holder it cannot read admits nobody.
///
/// The same rule as a circle with a mistyped founder: a room nobody can be
/// admitted from is visibly broken, and one that admits on a typo is not.
#[tokio::test(flavor = "multi_thread")]
async fn a_waiting_room_naming_nobody_readable_is_closed() {
    let conductor = SweetConductor::standard().await;
    let ronnie = SweetAgents::one(conductor.keystore()).await;

    let room = SweetDnaFile::from_bundle_with_overrides(
        &dna_path(),
        DnaModifiersOpt::none().with_properties(CircleProperties {
            founder: None,
            lobby: false,
            seconder: None,
            requires_second_yes: false,
            waiting_for: Some("not an identifier at all".to_string()),
        }),
    )
    .await
    .expect("the packed DNA should load");

    let result = join(&conductor, "ronnie-room", &ronnie, &room, None).await;

    assert!(
        result.is_err(),
        "a waiting room that names nobody readable must close, not open"
    );
}

/// **The whole journey in, without anybody sending anybody an identifier.**
///
/// Ronnie knocks. Alice answers by leaving an invitation in the open room.
/// Ronnie collects it and it opens the circle. Nothing was copied between them
/// except the address of the room, which is public and never changes.
#[tokio::test(flavor = "multi_thread")]
async fn a_knock_answered_lets_somebody_into_the_circle() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let ronnie = SweetAgents::one(conductor.keystore()).await;

    // Alice's circle, and the door to it.
    let dna = circle_dna(&alice).await;
    let alice_cell = join(&conductor, "alice", &alice, &dna, None).await.unwrap();

    let room = a_waiting_room(&alice).await;
    let alice_room = join(&conductor, "alice-room", &alice, &room, None)
        .await
        .unwrap();
    let ronnie_room = join(&conductor, "ronnie-room", &ronnie, &room, None)
        .await
        .unwrap();

    // He asks. She has never seen his identifier and never asked for it.
    let _: Record = conductor
        .call(&zome(&ronnie_room), "knock", a_knock())
        .await;

    let at_the_door: Vec<aboutme::Knocking> =
        conductor.call(&zome(&alice_room), "get_knocks", ()).await;
    let asking = at_the_door.first().expect("she sees him at the door");

    // She makes his invitation from the key his knock brought with it.
    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: asking.who.clone(),
                name: asking.name.clone(),
            },
        )
        .await;

    // Left in the open room. Safe there because it is signed over his key and
    // admits nobody else.
    // The room carries the invitation as opaque text, exactly as the app does
    // — it packs it into one line before sending. Hex here rather than the
    // app's base64, because what is being tested is that the room hands back
    // what it was given, not the encoding.
    let carried = as_text(&bundle.invitation);
    let _: Record = conductor
        .call(
            &zome(&alice_room),
            "admit",
            aboutme::AdmitInput {
                knock: asking.knock.clone(),
                invitation: carried.clone(),
            },
        )
        .await;

    // He collects it.
    let waiting_for_him: Option<String> = conductor
        .call(&zome(&ronnie_room), "my_admission", ())
        .await;
    let collected = waiting_for_him.expect("the answer is waiting where he can reach it");
    assert_eq!(collected, carried);

    let invitation = from_text(&collected);

    join(&conductor, "ronnie", &ronnie, &dna, Some(&invitation))
        .await
        .expect("an invitation collected from the door must open the circle");
}

// ---------------------------------------------------------------------------
// Both sides must build the same circle
// ---------------------------------------------------------------------------
//
// A circle is its DNA hash, and everything that goes into that hash has to be
// computed identically by the person who made it and the person joining. Get
// it wrong and there is no error anywhere: two circles with the same name,
// both working perfectly, invisible to each other.
//
// That is not hypothetical. It happened, and it cost ten minutes of staring at
// two screens that both said everything was fine — because the invitation
// carried *who* had been asked to agree but not *whether* anybody had to be,
// and nobody had been asked yet.

/// The invitation carries the rule, not just the person.
#[tokio::test(flavor = "multi_thread")]
async fn a_joiner_lands_in_the_same_circle_when_nobody_is_appointed_yet() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let bob = SweetAgents::one(conductor.keystore()).await;

    // The lobby every installation has, which real circles are cloned from.
    let lobby = lobby_dna().await;
    let alice_lobby = join(&conductor, "alice-lobby", &alice, &lobby, None)
        .await
        .expect("anyone may enter the lobby");

    // A circle that asks two people to agree, with nobody asked yet — which is
    // every such circle for its first few minutes.
    let hers: ClonedCell = conductor
        .call(
            &zome(&alice_lobby),
            "create_circle",
            aboutme::CreateCircleInput {
                founder: alice.to_string(),
                name: "Margaret".to_string(),
                network_seed: "same-circle".to_string(),
                requires_second_yes: true,
            },
        )
        .await;

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&hers.cell_id),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: "Dave".to_string(),
            },
        )
        .await;

    assert!(
        bundle.requires_second_yes,
        "the invitation has to say the circle asks two people, or the joiner \
         builds one that asks nobody"
    );
    assert!(
        bundle.seconder.is_none(),
        "and nobody has been asked yet, which is exactly the case that broke"
    );

    let bob_lobby = join(&conductor, "bob-lobby", &bob, &lobby, None)
        .await
        .expect("anyone may enter the lobby");

    let his: ClonedCell = conductor
        .call(
            &zome(&bob_lobby),
            "join_circle",
            aboutme::JoinCircleInput {
                founder: bundle.founder.clone(),
                name: "Margaret".to_string(),
                network_seed: bundle.network_seed.clone(),
                invitation: bundle.invitation.clone(),
                requires_second_yes: bundle.requires_second_yes,
                seconder: bundle.seconder.clone(),
            },
        )
        .await;

    // The whole assertion. Anything else being equal is not enough: if these
    // differ they are two networks, and nothing will ever tell either of them.
    assert_eq!(
        hers.cell_id.dna_hash(),
        his.cell_id.dna_hash(),
        "the holder and the joiner must compute the same circle"
    );
}

/// And the same where the circle asks nobody, so the flag is not just ignored.
#[tokio::test(flavor = "multi_thread")]
async fn a_joiner_lands_in_the_same_circle_when_it_asks_nobody() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let bob = SweetAgents::one(conductor.keystore()).await;

    let lobby = lobby_dna().await;
    let alice_lobby = join(&conductor, "alice-lobby", &alice, &lobby, None)
        .await
        .unwrap();

    let hers: ClonedCell = conductor
        .call(
            &zome(&alice_lobby),
            "create_circle",
            aboutme::CreateCircleInput {
                founder: alice.to_string(),
                name: "Margaret".to_string(),
                network_seed: "asks-nobody".to_string(),
                requires_second_yes: false,
            },
        )
        .await;

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&hers.cell_id),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    assert!(!bundle.requires_second_yes);

    let bob_lobby = join(&conductor, "bob-lobby", &bob, &lobby, None)
        .await
        .unwrap();

    let his: ClonedCell = conductor
        .call(
            &zome(&bob_lobby),
            "join_circle",
            aboutme::JoinCircleInput {
                founder: bundle.founder.clone(),
                name: "Margaret".to_string(),
                network_seed: bundle.network_seed.clone(),
                invitation: bundle.invitation.clone(),
                requires_second_yes: bundle.requires_second_yes,
                seconder: bundle.seconder.clone(),
            },
        )
        .await;

    assert_eq!(
        hers.cell_id.dna_hash(),
        his.cell_id.dna_hash(),
        "a circle that asks nobody must also be the same circle on both sides"
    );
}

/// Two circles that differ only in the rule are different circles.
///
/// The other half of the same fact. If these collided, the flag would not be
/// in the identity at all and none of the above would mean anything.
#[tokio::test(flavor = "multi_thread")]
async fn the_rule_is_part_of_what_makes_a_circle() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;

    let lobby = lobby_dna().await;
    let alice_lobby = join(&conductor, "alice-lobby", &alice, &lobby, None)
        .await
        .unwrap();

    let asking: ClonedCell = conductor
        .call(
            &zome(&alice_lobby),
            "create_circle",
            aboutme::CreateCircleInput {
                founder: alice.to_string(),
                name: "Asks two".to_string(),
                network_seed: "identical".to_string(),
                requires_second_yes: true,
            },
        )
        .await;

    let not_asking: ClonedCell = conductor
        .call(
            &zome(&alice_lobby),
            "create_circle",
            aboutme::CreateCircleInput {
                founder: alice.to_string(),
                name: "Asks nobody".to_string(),
                network_seed: "identical".to_string(),
                requires_second_yes: false,
            },
        )
        .await;

    assert_ne!(
        asking.cell_id.dna_hash(),
        not_asking.cell_id.dna_hash(),
        "same holder, same seed, different rule — and so a different circle"
    );
}

// ---------------------------------------------------------------------------
// Being asked is a question, not an instruction
// ---------------------------------------------------------------------------
//
// The holder writes an appointment naming somebody. Nothing about that can
// make them agree to an arrival — refusing is always available by simply never
// endorsing anybody. What these cover is that refusing can be *said*, where
// the holder sees it, and that nobody else can say it on their behalf.

/// What one cell says about who agrees here, right now.
async fn who_agrees(conductor: &SweetConductor, cell: &CellId) -> Option<aboutme::WhoAgrees> {
    conductor.call(&zome(cell), "who_agrees_here", ()).await
}

/// Wait for an answer to reach a cell that did not write it.
///
/// Polled rather than assumed. These are two agents on one conductor, but they
/// are still two agents: the answer is written on one chain and read from the
/// other over the network, and expecting that to be instant is how a test
/// comes to pass on the machine that wrote it and nowhere else.
async fn wait_for_answer(
    conductor: &SweetConductor,
    cell: &CellId,
    want: Option<bool>,
) -> Option<bool> {
    for _ in 0..60 {
        let seen = who_agrees(conductor, cell).await.and_then(|w| w.willing);
        if seen == want {
            return seen;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    who_agrees(conductor, cell).await.and_then(|w| w.willing)
}

/// Nobody has answered, and that is not the same as having said no.
#[tokio::test(flavor = "multi_thread")]
async fn asked_and_not_yet_answered_is_its_own_state() {
    let (conductor, alice_cell, _ruth_cell, _dave_cell, _ronnie) =
        a_circle_with_both_people_in_it().await;

    let asked = who_agrees(&conductor, &alice_cell)
        .await
        .expect("somebody has been asked");

    assert_eq!(
        asked.willing, None,
        "an unanswered asking must not read as a yes or as a no — the holder \
         acts differently on each of the three"
    );
}

/// The answer reaches the holder, which is the whole reason it is written down.
#[tokio::test(flavor = "multi_thread")]
async fn saying_no_is_something_the_holder_can_see() {
    let (conductor, alice_cell, ruth_cell, _dave_cell, _ronnie) =
        a_circle_with_both_people_in_it().await;

    let appointment = who_agrees(&conductor, &ruth_cell)
        .await
        .expect("Ruth was asked")
        .appointment;

    let _: Record = conductor
        .call(
            &zome(&ruth_cell),
            "answer_appointment",
            aboutme::AnswerInput {
                appointment,
                willing: false,
            },
        )
        .await;

    assert_eq!(
        wait_for_answer(&conductor, &alice_cell, Some(false)).await,
        Some(false),
        "a no the holder cannot see is the same to her as no answer at all, \
         and telling those two apart is what this exists for"
    );
}

/// You may change your mind, and the newest answer is the one that counts.
#[tokio::test(flavor = "multi_thread")]
async fn the_newest_answer_is_the_answer() {
    let (conductor, alice_cell, ruth_cell, _dave_cell, _ronnie) =
        a_circle_with_both_people_in_it().await;

    let appointment = who_agrees(&conductor, &ruth_cell)
        .await
        .expect("Ruth was asked")
        .appointment;

    for willing in [false, true] {
        let _: Record = conductor
            .call(
                &zome(&ruth_cell),
                "answer_appointment",
                aboutme::AnswerInput {
                    appointment: appointment.clone(),
                    willing,
                },
            )
            .await;
    }

    assert_eq!(
        who_agrees(&conductor, &ruth_cell).await.unwrap().willing,
        Some(true),
        "the second answer replaces the first on her own screen without \
         waiting for the network, which is the rule everywhere else here"
    );

    assert_eq!(
        wait_for_answer(&conductor, &alice_cell, Some(true)).await,
        Some(true),
        "and it replaces it for the holder too; nothing is erased, but the \
         newest answer is the one she is looking at"
    );
}

/// Somebody else's willingness is not yours to declare.
///
/// Without this an ordinary member could write "yes, she is willing" and the
/// holder's screen would say the safeguard was in place when the person
/// holding it had never heard of it.
#[tokio::test(flavor = "multi_thread")]
async fn only_the_person_asked_may_answer() {
    let (conductor, alice_cell, ruth_cell, dave_cell, _ronnie) =
        a_circle_with_both_people_in_it().await;

    let appointment = who_agrees(&conductor, &ruth_cell)
        .await
        .expect("Ruth was asked")
        .appointment;

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&dave_cell),
            "answer_appointment",
            aboutme::AnswerInput {
                appointment,
                willing: true,
            },
        )
        .await;

    assert!(
        result.is_err(),
        "only the person an appointment names may answer it"
    );

    assert_eq!(
        who_agrees(&conductor, &alice_cell).await.unwrap().willing,
        None,
        "and the holder is still waiting, rather than looking at an answer \
         Ruth never gave"
    );
}

/// A room is public, so what is said in it is sealed.
///
/// Once the room is the only way into a circle, every arrival is announced at
/// its door — and anybody who has ever been given the address can read that
/// door. "Ronnie Smythe, her cousin" in the open is a fact about who visits
/// somebody, which is itself sensitive: a psychiatrist, a substance misuse
/// worker, a domestic abuse advocate.
///
/// So the words are boxed to the holder. What cannot be hidden is the key that
/// wrote the knock — it is the action's author, and it is the whole reason
/// nobody had to collect it by hand.
#[tokio::test(flavor = "multi_thread")]
async fn a_knock_says_nothing_to_the_rest_of_the_room() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let ronnie = SweetAgents::one(conductor.keystore()).await;
    let mallory = SweetAgents::one(conductor.keystore()).await;

    let room = a_waiting_room(&alice).await;
    let alice_room = join(&conductor, "alice-room", &alice, &room, None)
        .await
        .expect("the holder stands at her own door");
    let ronnie_cell = join(&conductor, "ronnie-room", &ronnie, &room, None)
        .await
        .unwrap();
    let mallory_cell = join(&conductor, "mallory-room", &mallory, &room, None)
        .await
        .expect("the room is open to her too, which is the point of testing this");

    let _: Record = conductor
        .call(&zome(&ronnie_cell), "knock", a_knock())
        .await;

    // Somebody else in the same room. She can see that a knock happened and
    // whose key wrote it, and that is all she can see.
    let mut hers: Vec<aboutme::Knocking> = Vec::new();
    for _ in 0..60 {
        hers = conductor.call(&zome(&mallory_cell), "get_knocks", ()).await;
        if !hers.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    assert_eq!(hers.len(), 1, "the knock itself is not hidden, and cannot be");
    assert_eq!(
        hers[0].who,
        ronnie.to_string(),
        "nor is the key that wrote it — that is the action's author"
    );
    assert_eq!(
        hers[0].name, "",
        "but the words are sealed to the holder, and she is not the holder"
    );
    assert_eq!(hers[0].relationship, "");

    // The holder, who is the one person the words were sealed for.
    let mut theirs: Vec<aboutme::Knocking> = Vec::new();
    for _ in 0..60 {
        theirs = conductor.call(&zome(&alice_room), "get_knocks", ()).await;
        if theirs.first().is_some_and(|k| !k.name.is_empty()) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    assert_eq!(
        theirs.first().map(|k| k.name.as_str()),
        Some("Ronnie Smythe"),
        "the person deciding has to be able to read what she is deciding about"
    );
    assert_eq!(
        theirs[0].relationship, "her cousin",
        "and how they say they are connected, which is often the useful part"
    );
}

/// You can read your own knock back, which is not as obvious as it sounds.
///
/// Boxing is between two keys and opened with the recipient's secret, so a
/// knock sealed only to the holder would be unreadable to the person who wrote
/// it. That is not academic: somebody let in after a restart arrived nameless,
/// in a circle that then asked them who they were when they had already said.
#[tokio::test(flavor = "multi_thread")]
async fn the_person_knocking_can_read_their_own_knock() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let ronnie = SweetAgents::one(conductor.keystore()).await;

    let room = a_waiting_room(&alice).await;
    let ronnie_cell = join(&conductor, "ronnie-room", &ronnie, &room, None)
        .await
        .unwrap();

    let _: Record = conductor
        .call(&zome(&ronnie_cell), "knock", a_knock())
        .await;

    let mine: Vec<aboutme::Knocking> =
        conductor.call(&zome(&ronnie_cell), "get_knocks", ()).await;

    assert_eq!(
        mine.first().map(|k| k.name.as_str()),
        Some("Ronnie Smythe"),
        "his own app must be able to tell him back what he said"
    );
}

/// An empty name is refused, and it is the app that refuses it.
///
/// It used to be a validation rule checked by every peer. It cannot be now the
/// words are sealed — a peer that cannot read a thing cannot have an opinion
/// about it. The check moved rather than disappearing, and this is where it
/// went.
#[tokio::test(flavor = "multi_thread")]
async fn knocking_without_saying_who_you_are_is_refused() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let ronnie = SweetAgents::one(conductor.keystore()).await;

    let room = a_waiting_room(&alice).await;
    let ronnie_cell = join(&conductor, "ronnie-room", &ronnie, &room, None)
        .await
        .unwrap();

    let result: Result<Record, _> = conductor
        .call_fallible(
            &zome(&ronnie_cell),
            "knock",
            aboutme_integrity::WhoIsKnocking {
                name: "   ".to_string(),
                relationship: "her cousin".to_string(),
            },
        )
        .await;

    assert!(
        result.is_err(),
        "a knock with nobody's name on it tells the holder nothing she can act on"
    );
}

// ---------------------------------------------------------------------------
// How much anybody may write (migration batch, item 3)
// ---------------------------------------------------------------------------
//
// Everything written in a circle is copied to every member's device. The app
// keeps to these limits already; these tests are about a copy that does not,
// so every call below goes straight to the zome with nothing in between.

fn words(n: usize) -> String {
    vec!["word"; n].join(" ")
}

/// Five hundred words a section, and not one more.
#[tokio::test(flavor = "multi_thread")]
async fn a_section_holds_five_hundred_words_and_no_more() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;

    let mut fits = an_about_me("Alice Bell");
    fits.what_matters_to_me = words(500);
    let fitted: Result<Record, _> = conductor
        .call_fallible(&zome(&alice_cell), "create_about_me", fits)
        .await;
    assert!(fitted.is_ok(), "five hundred words is the limit, not over it");

    let mut over = an_about_me("Alice Bell");
    over.my_wellness = words(501);
    let refused: Result<Record, _> = conductor
        .call_fallible(&zome(&alice_cell), "create_about_me", over)
        .await;
    assert!(
        refused.is_err(),
        "one section of one person's record is copied to every device in the circle"
    );
}

/// A wall of text with no spaces in it is still too long.
#[tokio::test(flavor = "multi_thread")]
async fn text_without_spaces_cannot_dodge_the_limit() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;

    let mut wall = an_about_me("Alice Bell");
    wall.also_worth_knowing = "x".repeat(8_001);
    let refused: Result<Record, _> = conductor
        .call_fallible(&zome(&alice_cell), "create_about_me", wall)
        .await;
    assert!(
        refused.is_err(),
        "one word eight thousand characters long is not a way round five hundred words"
    );
}

/// A suggestion is held to the same limit as the section it is about.
#[tokio::test(flavor = "multi_thread")]
async fn a_suggestion_holds_five_hundred_words_and_no_more() {
    let (conductor, _, bob_cell) = a_circle_with_a_member().await;

    let refused: Result<Record, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "suggest",
            aboutme_integrity::Suggestion {
                field: aboutme_integrity::AboutMeField::WhatMattersToMe,
                text: words(501),
                because: String::new(),
                locked: None,
            },
        )
        .await;
    assert!(
        refused.is_err(),
        "any member may suggest, so any member could otherwise fill everybody's disk"
    );
}

/// A name is a name.
#[tokio::test(flavor = "multi_thread")]
async fn a_name_is_not_a_place_to_write_an_essay() {
    let (conductor, _, bob_cell) = a_circle_with_a_member().await;

    let refused: Result<Record, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "introduce_myself",
            aboutme_integrity::Member {
                name: "B".repeat(201),
                relationship: "her nephew".to_string(),
            },
        )
        .await;
    assert!(refused.is_err(), "a name can be up to two hundred characters");
}

/// Ten knocks by one person at one door, and no more.
///
/// Every knock sits on the holder's device. Somebody who knocks a few times
/// because nobody answered is fine; somebody who knocks a thousand times is
/// filling her disk and burying the people she is waiting for.
#[tokio::test(flavor = "multi_thread")]
async fn one_person_cannot_bury_a_door_in_knocks() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let ronnie = SweetAgents::one(conductor.keystore()).await;

    let room = a_waiting_room(&alice).await;
    let ronnie_cell = join(&conductor, "ronnie-room", &ronnie, &room, None)
        .await
        .expect("a waiting room is open");

    for n in 1..=10 {
        let knocked: Result<Record, _> = conductor
            .call_fallible(&zome(&ronnie_cell), "knock", a_knock())
            .await;
        assert!(knocked.is_ok(), "knock {n} of ten should be allowed");
    }

    let eleventh: Result<Record, _> = conductor
        .call_fallible(&zome(&ronnie_cell), "knock", a_knock())
        .await;
    assert!(
        eleventh.is_err(),
        "an eleventh knock by the same person at the same door is refused"
    );
}

// ---------------------------------------------------------------------------
// The name on an invitation is signed (migration batch, item 2)
// ---------------------------------------------------------------------------

/// Change the name on the way, and the invitation stops working.
///
/// The second person is asked to agree to "Ronnie, her cousin", not to a key.
/// Whoever carries an invitation between the two of them could once rename
/// the person on it without breaking anything. Now both signatures are over
/// the key and the name together.
#[tokio::test(flavor = "multi_thread")]
async fn renaming_somebody_on_an_invitation_breaks_it() {
    let conductor = SweetConductor::standard().await;
    let alice = SweetAgents::one(conductor.keystore()).await;
    let bob = SweetAgents::one(conductor.keystore()).await;
    let dna = circle_dna(&alice).await;

    let alice_cell = join(&conductor, "alice", &alice, &dna, None)
        .await
        .expect("the founder needs no invitation to her own circle");

    let bundle: aboutme::InvitationBundle = conductor
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: "Bob, her nephew".to_string(),
            },
        )
        .await;
    assert_eq!(bundle.invitation.name, "Bob, her nephew");

    let mut renamed = bundle.invitation.clone();
    renamed.name = "The district nurse".to_string();

    assert!(
        join(&conductor, "bob-renamed", &bob, &dna, Some(&renamed))
            .await
            .is_err(),
        "the name the holder signed is part of what she signed"
    );
    // The invitation as she made it opening the door is covered by every
    // other test that joins somebody by name.
}

// ---------------------------------------------------------------------------
// Removing somebody, the ordinary way (migration batch, item 4)
// ---------------------------------------------------------------------------

/// The holder removes somebody, lets them back, and the newest decision is
/// the one every app reads.
#[tokio::test(flavor = "multi_thread")]
async fn the_holder_can_remove_somebody_and_let_them_back() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;
    let bob = bob_cell.agent_pubkey().to_string();

    let _: Record = conductor
        .call(
            &zome(&alice_cell),
            "decide_departure",
            aboutme::DepartureInput {
                who: bob.clone(),
                removed: true,
            },
        )
        .await;
    let standing: Vec<aboutme::Standing> =
        conductor.call(&zome(&alice_cell), "get_departures", ()).await;
    assert_eq!(standing.len(), 1);
    assert!(standing[0].removed, "Bob has been removed");

    let _: Record = conductor
        .call(
            &zome(&alice_cell),
            "decide_departure",
            aboutme::DepartureInput {
                who: bob,
                removed: false,
            },
        )
        .await;
    let standing: Vec<aboutme::Standing> =
        conductor.call(&zome(&alice_cell), "get_departures", ()).await;
    assert_eq!(standing.len(), 1, "one line per person, not one per decision");
    assert!(!standing[0].removed, "the newest decision is the one that counts");
}

/// Only the holder removes anybody.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_remove_anybody() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    let refused: Result<Record, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "decide_departure",
            aboutme::DepartureInput {
                who: alice_cell.agent_pubkey().to_string(),
                removed: true,
            },
        )
        .await;
    assert!(
        refused.is_err(),
        "a member removing the holder, or anybody, is not a decision they can make"
    );
}

/// The holder cannot remove herself: that would leave a circle nobody may
/// write in, and it is what a successor is for.
#[tokio::test(flavor = "multi_thread")]
async fn the_holder_cannot_remove_herself() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;

    let refused: Result<Record, _> = conductor
        .call_fallible(
            &zome(&alice_cell),
            "decide_departure",
            aboutme::DepartureInput {
                who: alice_cell.agent_pubkey().to_string(),
                removed: true,
            },
        )
        .await;
    assert!(refused.is_err());
}

// ---------------------------------------------------------------------------
// Photos, sound and video (migration batch, item 7)
// ---------------------------------------------------------------------------

use aboutme_integrity::{AboutMeField, MediaKind};

fn a_photo(pieces: Vec<EntryHash>, size: u64) -> aboutme::AddMediaInput {
    aboutme::AddMediaInput {
        section: AboutMeField::PeopleWhoMatter,
        kind: MediaKind::Photo,
        mime_type: "image/jpeg".to_string(),
        file_name: "ruth-and-me.jpg".to_string(),
        in_words: "Me with my daughter Ruth at the allotment".to_string(),
        seconds: 0,
        pieces,
        size,
    }
}

/// The holder adds a photo, and it reads back exactly as it went in.
#[tokio::test(flavor = "multi_thread")]
async fn the_holder_can_add_a_photo_and_read_it_back() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;
    let picture: Vec<u8> = (0..50_000u32).map(|i| (i % 251) as u8).collect();

    let piece: EntryHash = conductor
        .call(&zome(&alice_cell), "add_media_piece", aboutme::Bytes(picture.clone()))
        .await;
    let _: Record = conductor
        .call(&zome(&alice_cell), "add_media", a_photo(vec![piece.clone()], 50_000))
        .await;

    let here: Vec<aboutme::MediaHere> =
        conductor.call(&zome(&alice_cell), "get_media", ()).await;
    assert_eq!(here.len(), 1);
    assert_eq!(here[0].media.pieces, vec![piece.clone()]);

    let back: aboutme::Bytes = conductor
        .call(&zome(&alice_cell), "get_media_piece", piece.clone())
        .await;
    assert_eq!(back.0, picture, "the photo comes back byte for byte");

    // The words beside it are locked too, and read back as they were written.
    assert_eq!(
        here[0].media.in_words,
        "Me with my daughter Ruth at the allotment",
        "the caption is locked in the circle and opened for a member"
    );

    let _: () = conductor
        .call(&zome(&alice_cell), "remove_media", here[0].item.clone())
        .await;
    let after: Vec<aboutme::MediaHere> =
        conductor.call(&zome(&alice_cell), "get_media", ()).await;
    assert!(after.is_empty(), "a removed photo does not come back on her own screen");
}

/// A photo added after somebody is removed cannot be opened on their device.
///
/// The same rule as the record, applied to the thing people mind most about: a
/// photograph of somebody in their own home. On two devices, so that his
/// keystore is his — see `a_circle_on_two_devices`.
#[tokio::test(flavor = "multi_thread")]
async fn a_removed_member_cannot_open_a_photo_added_afterwards() {
    let (conductors, alice_cell, bob_cell) = a_circle_on_two_devices().await;
    let hers = conductors.get(0).unwrap();
    let his = conductors.get(1).unwrap();
    let bob = bob_cell.agent_pubkey().clone();

    keys_flowing_on_two(&conductors, &alice_cell, &bob_cell).await;

    let _: Record = hers
        .call(
            &zome(&alice_cell),
            "decide_departure",
            aboutme::DepartureInput {
                who: bob.to_string(),
                removed: true,
            },
        )
        .await;
    keys_until_on(&conductors, 0, &alice_cell, |k| {
        k.epoch == 2 && k.mine == 2
    })
    .await;

    let picture: Vec<u8> = (0..20_000u32).map(|i| (i % 251) as u8).collect();
    let piece: EntryHash = hers
        .call(
            &zome(&alice_cell),
            "add_media_piece",
            aboutme::Bytes(picture),
        )
        .await;
    let _: Record = hers
        .call(
            &zome(&alice_cell),
            "add_media",
            a_photo(vec![piece.clone()], 20_000),
        )
        .await;

    // It reaches his device like everything else does; it is the opening of it
    // that fails. Waiting for it to arrive first is what makes that the claim.
    let mut arrived = false;
    for _ in 0..60 {
        let here: Vec<aboutme::MediaHere> = his.call(&zome(&bob_cell), "get_media", ()).await;
        if here.iter().any(|m| m.media.pieces.contains(&piece)) {
            arrived = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    assert!(
        arrived,
        "the photo reaches his device, as everything here does"
    );

    let asked: Result<aboutme::Bytes, _> = his
        .call_fallible(&zome(&bob_cell), "get_media_piece", piece)
        .await;
    assert!(
        asked.is_err(),
        "a removed member must not be able to open a photo added after he went"
    );
}

/// Media is part of the person's account of themselves: only the holder adds it.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_add_media() {
    let (conductor, _, bob_cell) = a_circle_with_a_member().await;
    let refused: Result<EntryHash, _> = conductor
        .call_fallible(&zome(&bob_cell), "add_media_piece", aboutme::Bytes(vec![1; 1000]))
        .await;
    assert!(refused.is_err(), "a member cannot put files on everybody's device");
}

/// A piece is at most three megabytes.
#[tokio::test(flavor = "multi_thread")]
async fn a_piece_over_three_megabytes_is_refused() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;
    let refused: Result<EntryHash, _> = conductor
        .call_fallible(
            &zome(&alice_cell),
            "add_media_piece",
            aboutme::Bytes(vec![7; 3_000_001]),
        )
        .await;
    assert!(refused.is_err());
}

/// Each kind has its own limits: types, length, pieces.
#[tokio::test(flavor = "multi_thread")]
async fn media_is_held_to_the_limits_of_its_kind() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;
    let piece: EntryHash = conductor
        .call(&zome(&alice_cell), "add_media_piece", aboutme::Bytes(vec![3; 1000]))
        .await;

    // A photo that says it is a video file.
    let mut wrong_type = a_photo(vec![piece.clone()], 1000);
    wrong_type.mime_type = "video/webm".to_string();
    let refused: Result<Record, _> = conductor
        .call_fallible(&zome(&alice_cell), "add_media", wrong_type)
        .await;
    assert!(refused.is_err(), "a photo is an image file");

    // A recording longer than two minutes.
    let mut too_long = a_photo(vec![piece.clone()], 1000);
    too_long.kind = MediaKind::Sound;
    too_long.mime_type = "audio/webm".to_string();
    too_long.seconds = 121;
    let refused: Result<Record, _> = conductor
        .call_fallible(&zome(&alice_cell), "add_media", too_long)
        .await;
    assert!(refused.is_err(), "two minutes at most");

    // A video in more pieces than thirty megabytes needs.
    let mut too_many = a_photo(vec![piece; 11], 11_000);
    too_many.kind = MediaKind::Video;
    too_many.mime_type = "video/webm".to_string();
    too_many.seconds = 60;
    let refused: Result<Record, _> = conductor
        .call_fallible(&zome(&alice_cell), "add_media", too_many)
        .await;
    assert!(refused.is_err(), "ten pieces at most");
}

// ---------------------------------------------------------------------------
// A successor (migration batch, item 9)
// ---------------------------------------------------------------------------

fn naming(successor: Option<&AgentPubKey>) -> aboutme::NameSuccessorInput {
    aboutme::NameSuccessorInput {
        successor: successor.map(|k| k.to_string()),
        checker: None,
    }
}

/// The holder names a successor; the successor starts; the holder says she is
/// still here — and every step is where the circle can see it.
#[tokio::test(flavor = "multi_thread")]
async fn a_successor_can_start_and_the_holder_can_stop_it() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;
    let bob = bob_cell.agent_pubkey().clone();

    let _: Record = conductor
        .call(&zome(&alice_cell), "name_successor", naming(Some(&bob)))
        .await;
    let claim: Record = conductor
        .call(&zome(&bob_cell), "start_taking_over", ())
        .await;

    let _: Record = conductor
        .call(&zome(&alice_cell), "still_here", claim.action_address().clone())
        .await;
    let state: aboutme::SuccessionState =
        conductor.call(&zome(&alice_cell), "get_succession", ()).await;
    let seen = state.claim.expect("the claim is where she can see it");
    assert_eq!(seen.by, bob.to_string());
    assert!(seen.still_here, "and so is her answer");
}

/// Only the holder names a successor.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_name_themselves_successor() {
    let (conductor, _, bob_cell) = a_circle_with_a_member().await;
    let refused: Result<Record, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "name_successor",
            naming(Some(bob_cell.agent_pubkey())),
        )
        .await;
    assert!(refused.is_err());
}

/// The holder cannot be her own successor.
#[tokio::test(flavor = "multi_thread")]
async fn the_holder_cannot_succeed_herself() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;
    let refused: Result<Record, _> = conductor
        .call_fallible(
            &zome(&alice_cell),
            "name_successor",
            naming(Some(alice_cell.agent_pubkey())),
        )
        .await;
    assert!(refused.is_err());
}

/// Nobody can start taking over who was not named.
#[tokio::test(flavor = "multi_thread")]
async fn only_the_person_named_can_start_taking_over() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;
    // Named: nobody.
    let _: Record = conductor
        .call(&zome(&alice_cell), "name_successor", naming(None))
        .await;
    let refused: Result<Record, _> = conductor
        .call_fallible(&zome(&bob_cell), "start_taking_over", ())
        .await;
    assert!(refused.is_err(), "Bob was never named");
}

/// The successor cannot also be the one who checks on her, and she cannot
/// check on herself: the point is a second person.
#[tokio::test(flavor = "multi_thread")]
async fn nobody_checks_on_the_holder_but_a_second_person() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;
    let bob = bob_cell.agent_pubkey().clone();
    let _: Record = conductor
        .call(&zome(&alice_cell), "name_successor", naming(Some(&bob)))
        .await;
    let claim: Record = conductor
        .call(&zome(&bob_cell), "start_taking_over", ())
        .await;

    let by_successor: Result<Record, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "check_on_holder",
            aboutme::CheckInput {
                claim: claim.action_address().clone(),
                holder_can_carry_on: false,
            },
        )
        .await;
    assert!(by_successor.is_err(), "the successor cannot vouch for his own claim");

    let by_holder: Result<Record, _> = conductor
        .call_fallible(
            &zome(&alice_cell),
            "check_on_holder",
            aboutme::CheckInput {
                claim: claim.action_address().clone(),
                holder_can_carry_on: true,
            },
        )
        .await;
    assert!(by_holder.is_err(), "she answers with I'm still here");
}

/// A check by a third person is seen — by everybody, including whoever gave it.
///
/// The first check ever given, in the demo, vanished: reading answers back
/// took a check for a "still here" from the wrong person and dropped it. This
/// is the test that would have caught it.
#[tokio::test(flavor = "multi_thread")]
async fn a_check_on_the_holder_is_seen() {
    let (conductor, alice_cell, ruth_cell, dave_cell, _) =
        a_circle_with_both_people_in_it().await;
    let dave = dave_cell.agent_pubkey().clone();
    let ruth = ruth_cell.agent_pubkey().clone();

    let _: Record = conductor
        .call(
            &zome(&alice_cell),
            "name_successor",
            aboutme::NameSuccessorInput {
                successor: Some(dave.to_string()),
                checker: Some(ruth.to_string()),
            },
        )
        .await;
    let claim: Record = conductor
        .call(&zome(&dave_cell), "start_taking_over", ())
        .await;
    let _: Record = conductor
        .call(
            &zome(&ruth_cell),
            "check_on_holder",
            aboutme::CheckInput {
                claim: claim.action_address().clone(),
                holder_can_carry_on: false,
            },
        )
        .await;

    let state: aboutme::SuccessionState =
        conductor.call(&zome(&ruth_cell), "get_succession", ()).await;
    let seen = state.claim.expect("the claim stands");
    assert_eq!(seen.checks.len(), 1, "the check is there");
    assert!(!seen.checks[0].holder_can_carry_on);
    assert!(!seen.still_here, "and it was not mistaken for 'still here'");
}

// ---------------------------------------------------------------------------
// Keys (migration batch, item 6)
// ---------------------------------------------------------------------------
//
// The one thing encryption adds over the everyday removal: what the circle
// writes after somebody is removed reaches their device locked with a key they
// were never given. These tests are about who ends up holding which key,
// because that is the whole of it.

/// A circle on two devices, each with a keystore of its own.
///
/// Every other test in this file runs both people inside one conductor, which is
/// right for rules: they are checked by code, and the code is the same. It is
/// useless for keys. One conductor has one keystore, and a key is stored there
/// under a name — so a key the holder made is reachable from any agent in that
/// conductor, and "he was never given this key" cannot be shown at all. Two
/// conductors are two keystores, which is what being given a key means.
///
/// Returns the batch (the calls have to go to the conductor that owns the cell),
/// the holder's cell, and the member's.
async fn a_circle_on_two_devices() -> (SweetConductorBatch, CellId, CellId) {
    let conductors = SweetConductorBatch::standard(2).await;
    let alice = SweetAgents::one(conductors.get(0).unwrap().keystore()).await;
    let bob = SweetAgents::one(conductors.get(1).unwrap().keystore()).await;
    let dna = circle_dna(&alice).await;

    let alice_cell = join(conductors.get(0).unwrap(), "alice", &alice, &dna, None)
        .await
        .expect("the founder needs no invitation to her own circle");

    let bundle: aboutme::InvitationBundle = conductors
        .get(0)
        .unwrap()
        .call(
            &zome(&alice_cell),
            "invite",
            aboutme::InviteInput {
                invitee: bob.to_string(),
                name: String::new(),
            },
        )
        .await;

    let bob_cell = join(
        conductors.get(1).unwrap(),
        "bob",
        &bob,
        &dna,
        Some(&bundle.invitation),
    )
    .await
    .expect("an invited agent should be admitted");

    // Two conductors do not find each other on their own here.
    conductors.exchange_peer_info().await;

    (conductors, alice_cell, bob_cell)
}

/// Get the circle's key to the member, across two conductors.
///
/// The order matters and is the app's own: the member's device publishes its
/// encryption key first, because until it has, the holder has nothing to seal
/// to; then the holder makes the circle's key and seals it; then the member
/// takes it up. Waiting on the member first can never finish, which is exactly
/// what four of these tests did before this existed.
async fn keys_flowing_on_two(
    conductors: &SweetConductorBatch,
    holder: &CellId,
    member: &CellId,
) -> aboutme::KeysHere {
    both_until(conductors, holder, member, |k| k.mine >= 1).await
}

/// Refresh both devices until the member's own state is what is wanted.
///
/// Both, in turn, because that is how this works in life: the member's device
/// publishes its encryption key, and the holder's device has to *see* that
/// before it can seal anything to them — which on two machines is a moment
/// later, not instantly. Polling the member alone waits for something nobody
/// is doing, which is how two of these tests failed with
/// `KeysHere { epoch: 1, mine: 0 }`: the circle had a key and he had not been
/// given it, because nothing asked her device to look again.
async fn both_until(
    conductors: &SweetConductorBatch,
    holder: &CellId,
    member: &CellId,
    enough: impl Fn(&aboutme::KeysHere) -> bool,
) -> aboutme::KeysHere {
    let mut last: Option<aboutme::KeysHere> = None;
    for _ in 0..60 {
        let _: aboutme::KeysHere = conductors
            .get(0)
            .unwrap()
            .call(&zome(holder), "keep_keys_up_to_date", ())
            .await;
        let his: aboutme::KeysHere = conductors
            .get(1)
            .unwrap()
            .call(&zome(member), "keep_keys_up_to_date", ())
            .await;
        if enough(&his) {
            return his;
        }
        last = Some(his);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    panic!("the member's keys never reached what the test waited for: {last:?}");
}

/// The same as `keys_until`, for a cell in a batch of conductors.
async fn keys_until_on(
    conductors: &SweetConductorBatch,
    which: usize,
    cell: &CellId,
    enough: impl Fn(&aboutme::KeysHere) -> bool,
) -> aboutme::KeysHere {
    let conductor = conductors.get(which).expect("that conductor exists");
    let mut last: Option<aboutme::KeysHere> = None;
    for _ in 0..60 {
        let here: aboutme::KeysHere = conductor.call(&zome(cell), "keep_keys_up_to_date", ()).await;
        if enough(&here) {
            return here;
        }
        last = Some(here);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    panic!("keys never reached the state the test waited for: {last:?}");
}

/// Ask a cell about keys until it says what the test is waiting for, or give up.
///
/// Keys travel as entries, so one member's device learns about another's a
/// moment later, as with everything else here.
async fn keys_until(
    conductor: &SweetConductor,
    cell: &CellId,
    enough: impl Fn(&aboutme::KeysHere) -> bool,
) -> aboutme::KeysHere {
    let mut last: Option<aboutme::KeysHere> = None;
    for _ in 0..20 {
        let here: aboutme::KeysHere = conductor.call(&zome(cell), "keep_keys_up_to_date", ()).await;
        if enough(&here) {
            return here;
        }
        last = Some(here);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    panic!("keys never reached the state the test waited for: {last:?}");
}

/// The holder makes the first key, and every member ends up able to use it.
#[tokio::test(flavor = "multi_thread")]
async fn everybody_in_the_circle_can_use_the_key() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    // Bob's device publishes its encryption key; without it the holder has
    // nothing to seal to.
    let _: aboutme::KeysHere = conductor
        .call(&zome(&bob_cell), "keep_keys_up_to_date", ())
        .await;

    let hers = keys_until(&conductor, &alice_cell, |k| k.epoch == 1 && k.mine == 1).await;
    assert!(
        hers.waiting_for.is_empty(),
        "the holder should not still be waiting for anybody's key: {:?}",
        hers.waiting_for
    );

    let his = keys_until(&conductor, &bob_cell, |k| k.mine == 1).await;
    assert_eq!(his.epoch, 1, "one key, and Bob can use it");
}

/// Somebody removed is not given the key the circle uses from then on.
///
/// On two devices, because that is the only way this can be shown. See
/// `a_circle_on_two_devices`.
#[tokio::test(flavor = "multi_thread")]
async fn a_removed_member_is_not_given_the_next_key() {
    let (conductors, alice_cell, bob_cell) = a_circle_on_two_devices().await;
    let bob = bob_cell.agent_pubkey().clone();

    keys_flowing_on_two(&conductors, &alice_cell, &bob_cell).await;

    let _: Record = conductors
        .get(0)
        .unwrap()
        .call(
            &zome(&alice_cell),
            "decide_departure",
            aboutme::DepartureInput {
                who: bob.to_string(),
                removed: true,
            },
        )
        .await;

    let hers = keys_until_on(&conductors, 0, &alice_cell, |k| {
        k.epoch == 2 && k.mine == 2
    })
    .await;
    assert_eq!(hers.epoch, 2, "removing somebody starts a new key");

    // Bob's device sees that the circle has moved on, and cannot follow: no
    // amount of asking gives him key 2, because nothing in the circle carries
    // it to him.
    // Both devices refresh, so this is not waiting on something nobody does: she
    // hands out keys and he takes up whatever is his. Key 2 is not.
    let his = both_until(&conductors, &alice_cell, &bob_cell, |k| k.epoch == 2).await;
    assert_eq!(
        his.mine, 1,
        "a removed member keeps what he had and is given nothing after"
    );
    let held: Vec<u32> = conductors
        .get(1)
        .unwrap()
        .call(&zome(&bob_cell), "keys_i_can_use", ())
        .await;
    assert_eq!(
        held,
        vec![1],
        "key 1 only, which he was given while he was in"
    );
}

/// Somebody owed the circle's history is given every past key, not only the
/// one in use.
///
/// Ceri's decision of 20 September 2026: the record's history is part of the
/// record, so a new district nurse can read how it used to read. Shown here by
/// removing somebody and letting them back, which is the same code path a
/// genuinely new member takes — the holder seals every epoch to anybody who is
/// owed one. On two devices, because what is being tested is which keys reach
/// his keystore.
#[tokio::test(flavor = "multi_thread")]
async fn somebody_owed_the_history_is_given_every_past_key() {
    let (conductors, alice_cell, bob_cell) = a_circle_on_two_devices().await;
    let hers = conductors.get(0).unwrap();
    let bob = bob_cell.agent_pubkey().clone();

    keys_flowing_on_two(&conductors, &alice_cell, &bob_cell).await;

    // He goes, which starts key 2 without him; then he is let back in, and is
    // owed both keys.
    for removed in [true, false] {
        let _: Record = hers
            .call(
                &zome(&alice_cell),
                "decide_departure",
                aboutme::DepartureInput {
                    who: bob.to_string(),
                    removed,
                },
            )
            .await;
    }

    let his = both_until(&conductors, &alice_cell, &bob_cell, |k| k.mine == 2).await;
    assert_eq!(
        his.mine, 2,
        "back in the circle, and able to read what it says now"
    );

    // And key 1 as well, which is what "the history" means: both keys were
    // sealed to him, not only the one the circle is using now.
    let held: Vec<u32> = conductors
        .get(1)
        .unwrap()
        .call(&zome(&bob_cell), "keys_i_can_use", ())
        .await;
    assert_eq!(held, vec![1, 2], "the history, and the present");

    let sealed_again: u32 = hers.call(&zome(&alice_cell), "hand_out_keys", ()).await;
    assert_eq!(
        sealed_again, 0,
        "and nothing is handed out twice, however often the circle is opened"
    );
}

/// Only the person who holds the circle hands out its keys.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_cannot_hand_out_keys() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;
    let _: aboutme::KeysHere = conductor
        .call(&zome(&bob_cell), "keep_keys_up_to_date", ())
        .await;
    keys_until(&conductor, &alice_cell, |k| k.epoch == 1).await;

    let refused: Result<u32, _> = conductor.call_fallible(&zome(&bob_cell), "new_key", ()).await;
    assert!(
        refused.is_err(),
        "a member starting a new key would lock the holder out of her own circle"
    );

    let sealed: u32 = conductor.call(&zome(&bob_cell), "hand_out_keys", ()).await;
    assert_eq!(sealed, 0, "and he seals nothing to anybody");
}

/// The record goes into the circle locked, and nothing of it in the open.
///
/// This is the test that would catch the worst mistake available here: words
/// written where anybody receiving the circle could read them, with every
/// screen still looking exactly right.
#[tokio::test(flavor = "multi_thread")]
async fn the_record_is_written_locked() {
    let (conductor, alice_cell, _) = a_circle_with_a_member().await;

    let created: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;

    let written = AboutMe::try_from(
        created
            .entry()
            .as_option()
            .cloned()
            .expect("the record has an entry"),
    )
    .expect("and it is an About Me");

    assert!(
        written.locked.is_some(),
        "the record must be locked with the circle's key"
    );
    assert!(
        written.display_name.is_empty()
            && written.what_matters_to_me.is_empty()
            && written.supported_to_write_this_by.is_empty(),
        "and nothing of it may be left in the open: {written:?}"
    );

    // And it reads back as what was typed, for the person who holds the key.
    let current: aboutme::CurrentAboutMe = conductor
        .call(
            &zome(&alice_cell),
            "get_current_about_me",
            created.action_address().clone(),
        )
        .await;
    let words = current.about_me.expect("she can open her own record");
    assert_eq!(words.display_name, "Alice Bell");
    assert_eq!(words.what_matters_to_me, "Seeing my grandchildren");
    assert!(!current.locked_out);
}

/// A suggestion goes into the circle locked, and reads back as it was offered.
///
/// A suggestion carries what somebody noticed about a person — often the most
/// candid words in the circle — so it is locked like the record. Which section
/// it is about stays in the open, because that is a heading.
#[tokio::test(flavor = "multi_thread")]
async fn a_suggestion_is_written_locked() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;

    // Keys first: Bob cannot lock anything until he has one.
    let _: aboutme::KeysHere = conductor
        .call(&zome(&bob_cell), "keep_keys_up_to_date", ())
        .await;
    keys_until(&conductor, &alice_cell, |k| k.epoch == 1).await;
    keys_until(&conductor, &bob_cell, |k| k.mine == 1).await;

    let offered: Record = conductor
        .call(&zome(&bob_cell), "suggest", a_suggestion())
        .await;

    let written = Suggestion::try_from(
        offered
            .entry()
            .as_option()
            .cloned()
            .expect("the suggestion has an entry"),
    )
    .expect("and it is a suggestion");
    assert!(written.locked.is_some(), "it must be locked");
    assert!(
        written.text.is_empty() && written.because.is_empty(),
        "and nothing of what he wrote may be left in the open: {written:?}"
    );
    assert_eq!(
        written.field,
        aboutme_integrity::AboutMeField::WhatMattersToMe,
        "the section it is about is a heading, and stays readable"
    );

    // And the holder reads what he actually offered.
    let hash = offered.action_address().clone();
    let mut seen = None;
    for _ in 0..20 {
        let list: Vec<aboutme::SuggestionWithOutcome> =
            conductor.call(&zome(&alice_cell), "get_suggestions", ()).await;
        if let Some(found) = list
            .into_iter()
            .find(|s| s.suggestion.action_address() == &hash)
        {
            seen = found.words;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    let words = seen.expect("the holder can open what was offered to her");
    assert_eq!(words.text, "Her allotment. She talked about it all summer.");
    assert_eq!(words.because, "I am her son.");
}

/// Somebody removed cannot read what the circle writes afterwards.
///
/// The whole of what encryption adds over the everyday removal. Everything else
/// in this file is about an app behaving itself; this one holds whatever the app
/// on the other device does. On two devices, so that "his keystore" means
/// something — see `a_circle_on_two_devices`.
#[tokio::test(flavor = "multi_thread")]
async fn a_removed_member_cannot_open_what_is_written_next() {
    let (conductors, alice_cell, bob_cell) = a_circle_on_two_devices().await;
    let hers = conductors.get(0).unwrap();
    let his = conductors.get(1).unwrap();
    let bob = bob_cell.agent_pubkey().clone();

    keys_flowing_on_two(&conductors, &alice_cell, &bob_cell).await;

    let created: Record = hers
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;
    let original = created.action_address().clone();

    // While he is in the circle, Bob reads it. Nothing takes that back.
    let mut read_it = false;
    for _ in 0..60 {
        let current: aboutme::CurrentAboutMe = his
            .call(&zome(&bob_cell), "get_current_about_me", original.clone())
            .await;
        if current.about_me.is_some() {
            read_it = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    assert!(read_it, "a member reads the record he is there to read");

    // Then he is removed, which starts a new key, and she writes again.
    let _: Record = hers
        .call(
            &zome(&alice_cell),
            "decide_departure",
            aboutme::DepartureInput {
                who: bob.to_string(),
                removed: true,
            },
        )
        .await;
    keys_until_on(&conductors, 0, &alice_cell, |k| {
        k.epoch == 2 && k.mine == 2
    })
    .await;

    let mut after = an_about_me("Alice Bell");
    after.my_wellness = "Written after he was removed".into();
    let updated: Record = hers
        .call(
            &zome(&alice_cell),
            "update_about_me",
            aboutme::UpdateAboutMeInput {
                original_action_hash: original.clone(),
                previous_action_hash: original.clone(),
                about_me: after,
            },
        )
        .await;

    // His device receives it — replication does not choose person by person —
    // and cannot open it. Not "is not shown it": cannot open it.
    let mut arrived = None;
    for _ in 0..60 {
        let current: aboutme::CurrentAboutMe = his
            .call(&zome(&bob_cell), "get_current_about_me", original.clone())
            .await;
        if current.record.as_ref().map(|r| r.action_address()) == Some(updated.action_address()) {
            arrived = Some(current);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    let current = arrived.expect("the new version reaches his device, as everything does");
    assert!(
        current.about_me.is_none(),
        "a removed member must not be able to open what was written after he went"
    );
    assert!(
        current.locked_out,
        "and his app should say so, rather than showing an empty record"
    );
}

// ---------------------------------------------------------------------------
// Passes: the outer ring
// ---------------------------------------------------------------------------
//
// A pass lets somebody passing through — a ward nurse, a paramedic — read
// chosen sections without joining. It is a capability grant in the circle's
// door, so nearly every one of these is a test that something is *refused*:
// the sections not chosen, a pass that was stopped or has run out, a guessed
// secret, anybody but the holder making one. See docs/outer-ring.md.

/// Alice's circle, with her record written, and her door open on this device.
/// Nia, a nurse who is in nothing, has entered the same door.
async fn a_record_and_a_door() -> (SweetConductor, CellId, CellId, CellId, CellId) {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;
    let _: Record = conductor
        .call(
            &zome(&alice_cell),
            "create_about_me",
            an_about_me("Alice Bell"),
        )
        .await;

    let alice = alice_cell.agent_pubkey().clone();
    let nia = SweetAgents::one(conductor.keystore()).await;
    let door = a_waiting_room(&alice).await;
    let alice_door = join(&conductor, "alice-door", &alice, &door, None)
        .await
        .expect("the holder may open her own door");
    let nia_door = join(&conductor, "nia-door", &nia, &door, None)
        .await
        .expect("anybody may stand at a door");

    (conductor, alice_cell, bob_cell, alice_door, nia_door)
}

fn a_pass_for(circle: &CellId, until: Option<Timestamp>) -> aboutme::MakePassInput {
    aboutme::MakePassInput {
        circle: circle.dna_hash().to_string(),
        sections: vec![
            aboutme_integrity::AboutMeField::HowToCommunicateWithMe,
            aboutme_integrity::AboutMeField::PleaseDoAndPleaseDoNot,
        ],
        for_whom: "Ward 7".to_string(),
        until: until.map(|t| t.as_micros()),
    }
}

fn seconds_from_now(seconds: u64) -> Timestamp {
    (Timestamp::now() + std::time::Duration::from_secs(seconds)).expect("a time in range")
}

async fn ask(
    conductor: &SweetConductor,
    nia_door: &CellId,
    holder: &AgentPubKey,
    secret: CapSecret,
) -> holochain::conductor::api::error::ConductorApiResult<aboutme::PassedWords> {
    conductor
        .call_fallible(
            &zome(nia_door),
            "ask_with_a_pass",
            aboutme::AskWithPassInput {
                holder: holder.to_string(),
                secret,
            },
        )
        .await
}

/// The whole point: the sections chosen, in the holder's words, and nothing
/// else — and the holder's own screen is told it happened.
#[tokio::test(flavor = "multi_thread")]
async fn a_pass_reads_what_it_was_made_for_and_nothing_else() {
    let (conductor, alice_cell, _, alice_door, nia_door) = a_record_and_a_door().await;
    let mut alice_hears = conductor.subscribe_to_app_signals("alice-door".to_string());

    let pass: aboutme::PassMade = conductor
        .call(
            &zome(&alice_door),
            "make_a_pass",
            a_pass_for(&alice_cell, Some(seconds_from_now(3600))),
        )
        .await;

    let words = ask(&conductor, &nia_door, alice_cell.agent_pubkey(), pass.secret)
        .await
        .expect("a live pass, presented by whoever holds it, is answered");

    assert_eq!(words.name, "Alice Bell");
    assert_eq!(words.sections.len(), 2, "only the sections the pass was made for");
    assert_eq!(
        words.sections[0].words,
        "Speak to my left side, I'm deaf on the right"
    );
    assert_eq!(words.sections[1].words, "Please do not move my chair");
    let everything = format!("{words:?}");
    assert!(
        !everything.contains("grandchildren") && !everything.contains("district nurse"),
        "nothing from a section that was not chosen may travel"
    );

    let signal = tokio::time::timeout(std::time::Duration::from_secs(30), alice_hears.recv())
        .await
        .expect("the holder's screen should hear that the pass was used")
        .expect("the signal channel should stay open");
    let Signal::App { signal, .. } = signal else {
        panic!("expected an app signal");
    };
    let decoded: aboutme::Signal = signal.into_inner().decode().expect("one of ours");
    let aboutme::Signal::PassUsed { for_whom, .. } = decoded else {
        panic!("expected PassUsed, got {decoded:?}");
    };
    assert_eq!(for_whom, "Ward 7");
}

/// "Stop it" means stopped, by Holochain itself, before any of our code runs.
#[tokio::test(flavor = "multi_thread")]
async fn a_stopped_pass_opens_nothing() {
    let (conductor, alice_cell, _, alice_door, nia_door) = a_record_and_a_door().await;

    let pass: aboutme::PassMade = conductor
        .call(&zome(&alice_door), "make_a_pass", a_pass_for(&alice_cell, None))
        .await;
    ask(&conductor, &nia_door, alice_cell.agent_pubkey(), pass.secret)
        .await
        .expect("it works before it is stopped");

    let _: ActionHash = conductor
        .call(&zome(&alice_door), "stop_a_pass", pass.grant.clone())
        .await;

    assert!(
        ask(&conductor, &nia_door, alice_cell.agent_pubkey(), pass.secret)
            .await
            .is_err(),
        "a stopped pass must read nothing"
    );

    let listed: Vec<aboutme::PassHere> = conductor
        .call(&zome(&alice_door), "passes_here", ())
        .await;
    assert!(listed.is_empty(), "and it is no longer listed as given");
}

/// A pass with a time on it stops at that time, without anybody doing anything.
#[tokio::test(flavor = "multi_thread")]
async fn a_pass_that_has_run_out_opens_nothing() {
    let (conductor, alice_cell, _, alice_door, nia_door) = a_record_and_a_door().await;

    let pass: aboutme::PassMade = conductor
        .call(
            &zome(&alice_door),
            "make_a_pass",
            a_pass_for(&alice_cell, Some(seconds_from_now(2))),
        )
        .await;
    tokio::time::sleep(std::time::Duration::from_secs(4)).await;

    assert!(
        ask(&conductor, &nia_door, alice_cell.agent_pubkey(), pass.secret)
            .await
            .is_err(),
        "a pass past its time must read nothing"
    );

    let listed: Vec<aboutme::PassHere> = conductor
        .call(&zome(&alice_door), "passes_here", ())
        .await;
    assert!(listed[0].run_out, "and the holder's list says it has run out");
}

/// Without the secret there is no pass, whoever you are.
#[tokio::test(flavor = "multi_thread")]
async fn a_guessed_pass_opens_nothing() {
    let (conductor, alice_cell, _, alice_door, nia_door) = a_record_and_a_door().await;

    let _: aboutme::PassMade = conductor
        .call(&zome(&alice_door), "make_a_pass", a_pass_for(&alice_cell, None))
        .await;

    assert!(
        ask(&conductor, &nia_door, alice_cell.agent_pubkey(), [7u8; 64].into())
            .await
            .is_err(),
        "a secret nobody was given must read nothing"
    );
}

/// Only the person whose record it is gives it out.
///
/// Bob is in Alice's circle and can read everything in it. That does not make
/// it his to hand to a stranger.
#[tokio::test(flavor = "multi_thread")]
async fn only_the_holder_can_make_a_pass() {
    let (conductor, alice_cell, bob_cell, _, _) = a_record_and_a_door().await;

    let door = a_waiting_room(alice_cell.agent_pubkey()).await;
    let bob_at_her_door = join(&conductor, "bob-door", bob_cell.agent_pubkey(), &door, None)
        .await
        .expect("anybody may stand at a door");
    let from_bob: Result<aboutme::PassMade, _> = conductor
        .call_fallible(
            &zome(&bob_at_her_door),
            "make_a_pass",
            a_pass_for(&alice_cell, None),
        )
        .await;
    assert!(from_bob.is_err(), "a member cannot make a pass at her door");

    // Nor inside the circle itself: a pass is made at the door, which is the
    // only place the person it is given to can reach.
    let inside: Result<aboutme::PassMade, _> = conductor
        .call_fallible(&zome(&alice_cell), "make_a_pass", a_pass_for(&alice_cell, None))
        .await;
    assert!(inside.is_err(), "a pass is made at the door, not in the circle");
}

/// The function a pass reaches for the words must not be a way round the pass.
///
/// It answers only the device's own agent, and only in a circle that agent
/// holds — so Bob, calling it in his copy of Alice's circle, gets nothing.
#[tokio::test(flavor = "multi_thread")]
async fn the_words_behind_a_pass_are_not_a_back_door() {
    let (conductor, _, bob_cell, _, _) = a_record_and_a_door().await;

    let bob_asks: Result<aboutme::PassedWords, _> = conductor
        .call_fallible(
            &zome(&bob_cell),
            "words_for_a_pass",
            vec![aboutme_integrity::AboutMeField::WhatMattersToMe],
        )
        .await;
    assert!(
        bob_asks.is_err(),
        "a member's device cannot hand out a record it does not hold"
    );
}

/// Found in review, 23 September 2026: a device without the circle's newest key
/// used to lock with the newest one it had. After a removal, that older key is
/// exactly the one the removed person still holds. Now nothing is written with
/// an older key: a member waits a moment for the new one, and the removed
/// person's own device cannot write anything its old key opens.
#[tokio::test(flavor = "multi_thread")]
async fn nothing_is_locked_with_a_key_older_than_the_newest() {
    let (conductors, alice_cell, bob_cell) = a_circle_on_two_devices().await;
    keys_flowing_on_two(&conductors, &alice_cell, &bob_cell).await;

    let _: Record = conductors
        .get(0)
        .unwrap()
        .call(
            &zome(&alice_cell),
            "decide_departure",
            aboutme::DepartureInput {
                who: bob_cell.agent_pubkey().to_string(),
                removed: true,
            },
        )
        .await;

    // His device hears that there is a key 2. It is never sealed to him.
    keys_until_on(&conductors, 1, &bob_cell, |k| k.epoch == 2).await;

    let written: Result<Record, _> = conductors
        .get(1)
        .unwrap()
        .call_fallible(&zome(&bob_cell), "suggest", a_suggestion())
        .await;
    assert!(
        written.is_err(),
        "nothing may be locked with key 1 once the circle has key 2"
    );
}

/// Found in audit, 23 September 2026: the rules accept a claim resting on any
/// naming the holder ever made of that person, so an old claim — its checks
/// given, its waiting period long over — counted again the moment she named
/// the same person afresh. Now a new naming starts succession over.
#[tokio::test(flavor = "multi_thread")]
async fn a_new_naming_leaves_an_old_claim_behind() {
    let (conductor, alice_cell, bob_cell) = a_circle_with_a_member().await;
    let bob = bob_cell.agent_pubkey().clone();

    let _: Record = conductor
        .call(&zome(&alice_cell), "name_successor", naming(Some(&bob)))
        .await;
    let _: Record = conductor
        .call(&zome(&bob_cell), "start_taking_over", ())
        .await;

    // She acts again: names the same person, afresh.
    let _: Record = conductor
        .call(&zome(&alice_cell), "name_successor", naming(Some(&bob)))
        .await;

    let state: aboutme::SuccessionState =
        conductor.call(&zome(&alice_cell), "get_succession", ()).await;
    assert!(
        state.claim.is_none(),
        "a claim made under an earlier naming does not carry over"
    );
}
