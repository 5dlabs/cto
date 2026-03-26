const SYSTEM_PROMPT = `You are a concise voice narrator for a software development pipeline called Lobster.
Rewrite the following raw workflow message into a single short sentence (under 30 words) that a human listener can understand at a glance.
- Spell out abbreviations (PRD = Product Requirements Document, PR = pull request, LLM = language model).
- Convert long numbers to approximate human-readable form.
- Do not add greetings, pleasantries, or filler.
- Use an active voice. Be direct and factual.
- If it describes an error, make the severity clear.
Return ONLY the rewritten sentence, nothing else.`;

export async function llmHumanize(rawMessage: string): Promise<string | null> {
  const apiKey = process.env.OPENAI_API_KEY;
  if (!apiKey) return null;

  const model = process.env.LOBSTER_VOICE_LLM_MODEL ?? "gpt-4o-mini";

  try {
    const resp = await fetch("https://api.openai.com/v1/chat/completions", {
      method: "POST",
      headers: {
        "Authorization": `Bearer ${apiKey}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        model,
        max_tokens: 80,
        temperature: 0.3,
        messages: [
          { role: "system", content: SYSTEM_PROMPT },
          { role: "user", content: rawMessage },
        ],
      }),
    });

    if (!resp.ok) return null;

    const data = await resp.json() as {
      choices?: Array<{ message?: { content?: string } }>;
    };
    const content = data.choices?.[0]?.message?.content?.trim();
    return content || null;
  } catch {
    return null;
  }
}
