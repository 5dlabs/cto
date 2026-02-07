interface Env {
  DB: D1Database;
}

export const onRequestPost: PagesFunction<Env> = async (context) => {
  const { request, env } = context;
  
  // CORS headers
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

    // Insert into D1
    const result = await env.DB.prepare(
      "INSERT INTO waitlist (email) VALUES (?) ON CONFLICT(email) DO NOTHING RETURNING id"
    ).bind(email).first();

    if (result) {
      return new Response(
        JSON.stringify({ success: true, message: "Added to waitlist" }),
        { status: 201, headers }
      );
    } else {
      // Email already exists
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
