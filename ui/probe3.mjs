import {
  AdminWebsocket,
  AppWebsocket,
  encodeHashToBase64,
} from "@holochain/client";

const say = (...a) => console.log(...a);

for (const port of process.argv.slice(2).map(Number)) {
  say("\n=================== conductor on admin", port);
  const admin = await AdminWebsocket.connect({
    url: new URL(`ws://127.0.0.1:${port}`),
    wsClientOptions: { origin: "hc-spin" },
  });
  const app = (await admin.listApps({}))[0];
  const ifaces = await admin.listAppInterfaces();
  const appPort =
    ifaces[0]?.port ??
    (await admin.attachAppInterface({ allowed_origins: "probe" })).port;
  const token = await admin.issueAppAuthenticationToken({
    installed_app_id: app.installed_app_id,
    single_use: false,
    expiry_seconds: 300,
  });
  const client = await AppWebsocket.connect({
    url: new URL(`ws://127.0.0.1:${appPort}`),
    token: token.token,
    wsClientOptions: { origin: "hc-spin" },
    defaultTimeout: 60000,
  });

  const info = await client.appInfo();
  say("  agent:", encodeHashToBase64(info.agent_pub_key));

  for (const cells of Object.values(info.cell_info)) {
    for (const cell of cells) {
      const c = cell.value;
      if (!c?.cell_id) continue;
      try {
        await admin.authorizeSigningCredentials(c.cell_id);
      } catch {
        continue;
      }
      const ask = (fn, payload = null) =>
        client.callZome({
          cell_id: c.cell_id,
          zome_name: "aboutme",
          fn_name: fn,
          payload,
        });

      let holder = null;
      try {
        holder = await ask("who_holds_this");
      } catch (e) {
        continue;
      }

      const name = cell.type === "cloned" ? `clone "${c.name}"` : "provisioned";

      if (!holder) {
        try {
          const knocks = await ask("get_knocks");
          if (knocks.length) {
            say(`  -- ${name} (a room)`);
            say(
              "     knocks:",
              JSON.stringify(
                knocks.map((k) => ({
                  name: k.name,
                  who: k.who.slice(0, 10) + "…",
                  answered: k.answered,
                })),
              ),
            );
          }
        } catch {}
        continue;
      }

      say(`  -- ${name} (a circle)`);
      try {
        const originals = await ask("get_circle_about_me");
        say("     about-me records found:", originals.length);
        for (const original of originals) {
          const current = await ask("get_current_about_me", original);
          const entry = current?.record?.entry?.Present?.entry;
          say(
            "       record readable:",
            Boolean(current?.record),
            current?.record
              ? `(${current.record.signed_action.hashed.content.author ? "has author" : ""})`
              : "",
          );
        }
      } catch (e) {
        say("     about me FAILED:", (e.message ?? String(e)).slice(0, 180));
      }
      try {
        const pending = await ask("get_pending_members");
        say(
          "     pending proposals:",
          JSON.stringify(
            pending.map((p) => ({
              name: p.name,
              who: p.invitee.slice(0, 10) + "…",
              agreed: p.agreed,
              ready: Boolean(p.invitation),
            })),
          ),
        );
      } catch (e) {
        say("     pending FAILED:", (e.message ?? String(e)).slice(0, 180));
      }
      try {
        const members = await ask("get_members");
        say("     members:", members.length);
      } catch {}
    }
  }

  client.client.close();
  admin.client.close();
}

process.exit(0);
