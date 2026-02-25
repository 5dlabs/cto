interface Env {
  ATTIO_API_KEY: string;
}

const ATTIO_BASE = "https://api.attio.com/v2";

const LIST_SLUGS: Record<string, string> = {
  waitlist: "waitlist",
  investor: "investors",
  consulting: "consulting_leads",
};

async function attioFetch(path: string, env: Env, options: RequestInit = {}) {
  return fetch(`${ATTIO_BASE}${path}`, {
    ...options,
    headers: {
      Authorization: `Bearer ${env.ATTIO_API_KEY}`,
      "Content-Type": "application/json",
      ...options.headers,
    },
  });
}

async function assertPerson(email: string, name: string | undefined, env: Env) {
  const values: Record<string, unknown> = {
    email_addresses: [{ email_address: email }],
  };

  if (name) {
    const parts = name.trim().split(/\s+/);
    values.name = [
      {
        first_name: parts[0],
        last_name: parts.slice(1).join(" ") || undefined,
        full_name: name,
      },
    ];
  }

  const res = await attioFetch("/objects/people/records?matching_attribute=email_addresses", env, {
    method: "PUT",
    body: JSON.stringify({ data: { values } }),
  });

  if (!res.ok) {
    const err = await res.json();
    throw new Error(`Attio assert person failed: ${JSON.stringify(err)}`);
  }

  return res.json();
}

async function addToList(listSlug: string, personRecordId: string, env: Env) {
  const res = await attioFetch(`/lists/${listSlug}/entries`, env, {
    method: "POST",
    body: JSON.stringify({
      data: {
        parent_object: "people",
        parent_record_id: personRecordId,
        entry_values: {},
      },
    }),
  });

  if (!res.ok) {
    const err = await res.json() as Record<string, unknown>;
    if (err?.code === "already_exists") return { alreadyExists: true };
    throw new Error(`Attio add to list failed: ${JSON.stringify(err)}`);
  }

  return res.json();
}

export const onRequestPost: PagesFunction<Env> = async (context) => {
  const corsHeaders = {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "POST, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type",
  };

  try {
    const body = (await context.request.json()) as {
      email?: string;
      name?: string;
      source?: string;
    };

    if (!body.email || !body.email.includes("@")) {
      return new Response(JSON.stringify({ error: "Valid email required" }), {
        status: 400,
        headers: { "Content-Type": "application/json", ...corsHeaders },
      });
    }

    const source = body.source && LIST_SLUGS[body.source] ? body.source : "waitlist";
    const listSlug = LIST_SLUGS[source];

    const personResult = await assertPerson(body.email, body.name, context.env);
    const personId = personResult.data.id.record_id;

    const listResult = await addToList(listSlug, personId, context.env);
    const alreadyOnList = listResult?.alreadyExists === true;

    return new Response(
      JSON.stringify({
        success: true,
        alreadyOnList,
        message: alreadyOnList ? "You're already on the list!" : "You've been added!",
      }),
      {
        status: 200,
        headers: { "Content-Type": "application/json", ...corsHeaders },
      }
    );
  } catch (err) {
    console.error("Waitlist error:", err);
    return new Response(
      JSON.stringify({ error: "Something went wrong. Please try again." }),
      {
        status: 500,
        headers: { "Content-Type": "application/json", ...corsHeaders },
      }
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
