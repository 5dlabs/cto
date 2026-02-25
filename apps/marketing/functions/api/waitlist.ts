interface Env {
  DB: D1Database;
  ATTIO_API_KEY: string;
}

const ATTIO_BASE = "https://api.attio.com/v2";

async function syncToAttio(email: string, env: Env) {
  if (!env.ATTIO_API_KEY) return;

  try {
    const personRes = await fetch(
      `${ATTIO_BASE}/objects/people/records?matching_attribute=email_addresses`,
      {
        method: "PUT",
        headers: {
          Authorization: `Bearer ${env.ATTIO_API_KEY}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          data: {
            values: {
              email_addresses: [{ email_address: email }],
            },
          },
        }),
      }
    );

    if (!personRes.ok) return;

    const person = (await personRes.json()) as { data: { id: { record_id: string } } };
    const recordId = person.data.id.record_id;

    await fetch(`${ATTIO_BASE}/lists/waitlist/entries`, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${env.ATTIO_API_KEY}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        data: {
          parent_object: "people",
          parent_record_id: recordId,
          entry_values: {},
        },
      }),
    });
  } catch (err) {
    console.error("Attio sync failed (non-blocking):", err);
  }
}

export const onRequestPost: PagesFunction<Env> = async (context) => {
  const { request, env } = context;

  const headers = {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "POST, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type",
    "Content-Type": "application/json",
  };

  try {
    const body = await request.json() as { email?: string };
    const email = body.email?.trim().toLowerCase();

    if (!email || !email.includes("@")) {
      return new Response(
        JSON.stringify({ success: false, error: "Invalid email" }),
        { status: 400, headers }
      );
    }

    const result = await env.DB.prepare(
      "INSERT INTO waitlist (email) VALUES (?) ON CONFLICT(email) DO NOTHING RETURNING id"
    ).bind(email).first();

    // Sync to Attio CRM in the background (non-blocking)
    context.waitUntil(syncToAttio(email, env));

    if (result) {
      return new Response(
        JSON.stringify({ success: true, message: "Added to waitlist" }),
        { status: 201, headers }
      );
    } else {
      return new Response(
        JSON.stringify({ success: true, message: "Already on waitlist" }),
        { status: 200, headers }
      );
    }
  } catch (error) {
    console.error("Waitlist error:", error);
    return new Response(
      JSON.stringify({ success: false, error: "Server error" }),
      { status: 500, headers }
    );
  }
};

export const onRequestOptions: PagesFunction = async () => {
  return new Response(null, {
    status: 204,
    headers: {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "POST, OPTIONS",
      "Access-Control-Allow-Headers": "Content-Type",
    },
  });
};
