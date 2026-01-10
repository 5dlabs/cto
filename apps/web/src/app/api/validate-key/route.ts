import { NextRequest, NextResponse } from "next/server";
import { auth } from "@/lib/auth";
import { headers } from "next/headers";

interface ValidateKeyRequest {
  provider: string;
  api_key: string;
}

interface ValidateKeyResponse {
  valid: boolean;
  provider: string;
  message: string;
  models?: string[];
}

export async function POST(request: NextRequest) {
  const session = await auth.api.getSession({
    headers: await headers(),
  });

  if (!session) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }

  const body: ValidateKeyRequest = await request.json();

  if (!body.provider || !body.api_key) {
    return NextResponse.json(
      { error: "Provider and API key are required" },
      { status: 400 }
    );
  }

  try {
    const result = await validateApiKey(body.provider, body.api_key);
    return NextResponse.json(result);
  } catch (error) {
    console.error("Key validation error:", error);
    return NextResponse.json(
      {
        valid: false,
        provider: body.provider,
        message: "Failed to validate API key",
      },
      { status: 500 }
    );
  }
}

async function validateApiKey(
  provider: string,
  apiKey: string
): Promise<ValidateKeyResponse> {
  switch (provider.toLowerCase()) {
    case "anthropic": {
      // Validate Anthropic key by making a simple API call
      const response = await fetch("https://api.anthropic.com/v1/messages", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "x-api-key": apiKey,
          "anthropic-version": "2023-06-01",
        },
        body: JSON.stringify({
          model: "claude-3-haiku-20240307",
          max_tokens: 1,
          messages: [{ role: "user", content: "Hi" }],
        }),
      });

      if (response.ok) {
        return {
          valid: true,
          provider: "anthropic",
          message: "Anthropic API key is valid",
          models: [
            "claude-sonnet-4-20250514",
            "claude-3-5-sonnet-20241022",
            "claude-3-opus-20240229",
            "claude-3-haiku-20240307",
          ],
        };
      }

      const error = await response.json().catch(() => ({}));
      if (response.status === 401) {
        return {
          valid: false,
          provider: "anthropic",
          message: "Invalid API key",
        };
      }

      return {
        valid: false,
        provider: "anthropic",
        message: error.error?.message || "Failed to validate key",
      };
    }

    case "openai": {
      // Validate OpenAI key
      const response = await fetch("https://api.openai.com/v1/models", {
        headers: {
          Authorization: `Bearer ${apiKey}`,
        },
      });

      if (response.ok) {
        return {
          valid: true,
          provider: "openai",
          message: "OpenAI API key is valid",
          models: ["gpt-4o", "gpt-4-turbo", "gpt-3.5-turbo"],
        };
      }

      return {
        valid: false,
        provider: "openai",
        message: "Invalid API key",
      };
    }

    case "google":
    case "gemini": {
      // Validate Google/Gemini key
      const response = await fetch(
        `https://generativelanguage.googleapis.com/v1/models?key=${apiKey}`
      );

      if (response.ok) {
        return {
          valid: true,
          provider: "google",
          message: "Google API key is valid",
          models: ["gemini-2.0-flash", "gemini-1.5-pro", "gemini-1.5-flash"],
        };
      }

      return {
        valid: false,
        provider: "google",
        message: "Invalid API key",
      };
    }

    default:
      return {
        valid: false,
        provider,
        message: `Unsupported provider: ${provider}`,
      };
  }
}
