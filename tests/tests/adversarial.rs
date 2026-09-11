//! Adversarial tests.
//!
//! Every rule this project claims is a rule only if something rejects when it
//! is broken. These tests exist to break them.
//!
//! Prompted by an external red-team review which found that membership had
//! been implemented far more strongly than authorship, plus two holes it
//! missed: link creation and deletes were entirely unvalidated.

use aboutme_integrity::{AboutMe, CircleProperties, Invitation};
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

    (conductor, alice_cell, bob_cell)
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
        .call(&zome(&ruth_lobby), "second_an_invitation", bob.to_string())
        .await;

    let invitation = Invitation {
        signature: bundle.invitation.signature.clone(),
        seconded: Some(seconded),
        // Named, or the door has no second agreement to check and would let
        // Bob in on Alice's signature alone — which would make this test pass
        // without proving anything about Ruth at all.
        appointment: Some(appointment),
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
        .call(&zome(&alice_cell), "second_an_invitation", bob.to_string())
        .await;

    let invitation = Invitation {
        signature: bundle.invitation.signature.clone(),
        seconded: Some(forged),
        // Naming the appointment is what gives the door something to check
        // her forged signature against. Without it there is no second
        // agreement being claimed at all, and nothing to catch.
        appointment: Some(appointment),
    };

    assert!(
        join(&conductor, "bob", &bob, &dna, Some(&invitation))
            .await
            .is_err(),
        "a safeguard the holder can waive alone is not a safeguard"
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

    let readers: Vec<Record> = conductor
        .call(&zome(&alice), "get_acknowledgements", original)
        .await;
    assert_eq!(readers.len(), 1, "the holder should know it was read");
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
        .call(&zome(&ruth_cell), "second_an_invitation", dave.to_string())
        .await;
    let dave_invitation = aboutme_integrity::Invitation {
        signature: for_dave.invitation.signature.clone(),
        seconded: Some(seconded),
        appointment: None,
    };
    let dave_cell = join(&conductor, "dave", &dave, &dna, Some(&dave_invitation))
        .await
        .expect("two agreements admit an ordinary member");

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

fn a_knock() -> aboutme_integrity::Knock {
    aboutme_integrity::Knock {
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
