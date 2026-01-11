import { auth } from "@/lib/auth";
import { toNextJsHandler } from "better-auth/next-js";
import { NextRequest, NextResponse } from "next/server";

const { GET: baseGet, POST: basePost } = toNextJsHandler(auth);

// Check if OAuth credentials are configured
const isOAuthConfigured = (): boolean => {
  return !!process.env.GITHUB_CLIENT_ID && !!process.env.GITHUB_CLIENT_SECRET;
};

// Wrap handlers with error logging and credential validation
export async function GET(request: NextRequest) {
  try {
    // Check for OAuth callback requests
    if (request.url.includes("/callback/github") && !isOAuthConfigured()) {
      console.error("[Auth GET] GitHub OAuth callback received but credentials are not configured");
      return NextResponse.json(
        { 
          error: "OAuth not configured", 
          message: "GitHub OAuth credentials are not set. Please configure GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET environment variables." 
        },
        { status: 500 }
      );
    }
    return await baseGet(request);
  } catch (error) {
    console.error("[Auth GET] Error:", {
      url: request.url,
      error: error instanceof Error ? error.message : String(error),
      stack: error instanceof Error ? error.stack : undefined,
    });
    return NextResponse.json(
      { error: "Authentication error", message: error instanceof Error ? error.message : "Unknown error" },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    // Check for social sign-in requests
    if (request.url.includes("/sign-in/social")) {
      if (!isOAuthConfigured()) {
        console.error("[Auth POST] Social sign-in requested but GitHub OAuth credentials are not configured", {
          hasClientId: !!process.env.GITHUB_CLIENT_ID,
          hasClientSecret: !!process.env.GITHUB_CLIENT_SECRET,
          hasBetterAuthSecret: !!process.env.BETTER_AUTH_SECRET,
        });
        return NextResponse.json(
          { 
            error: "OAuth not configured", 
            message: "GitHub OAuth is not configured. Please ensure GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET environment variables are set.",
            code: "OAUTH_NOT_CONFIGURED"
          },
          { status: 500 }
        );
      }
    }
    return await basePost(request);
  } catch (error) {
    console.error("[Auth POST] Error:", {
      url: request.url,
      error: error instanceof Error ? error.message : String(error),
      stack: error instanceof Error ? error.stack : undefined,
    });
    return NextResponse.json(
      { error: "Authentication error", message: error instanceof Error ? error.message : "Unknown error" },
      { status: 500 }
    );
  }
}
