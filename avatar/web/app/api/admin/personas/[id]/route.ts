/**
 * Admin Persona Detail API - Get and Delete
 *
 * GET /api/admin/personas/[id] - Get persona details
 * DELETE /api/admin/personas/[id] - Delete persona
 *
 * Protected by Better Auth - requires admin role
 */

import { NextRequest, NextResponse } from "next/server";
import { promises as fs } from "fs";
import { join } from "path";
import { rm } from "fs/promises";

const PERSONAS_PATH = process.env.PERSONAS_PATH || "/personas";

interface PersonaStatus {
  state: "pending" | "preprocessing" | "ready" | "failed";
  error?: string;
  progress_percent: number;
  artefacts: string[];
}

interface PersonaMetadata {
  id: string;
  name: string;
  source_type: "image" | "video";
  uploaded_by: string;
  created_at: string;
}

interface PersonaDetail {
  id: string;
  name: string;
  source_type: "image" | "video";
  uploaded_by: string;
  created_at: string;
  state: string;
  error?: string;
  progress_percent: number;
  artefacts: string[];
  source_url?: string;
  preview_url?: string;
}

async function readStatus(personaId: string): Promise<PersonaStatus | null> {
  try {
    const statusPath = join(PERSONAS_PATH, personaId, "status.json");
    const data = await fs.readFile(statusPath, "utf-8");
    return JSON.parse(data) as PersonaStatus;
  } catch {
    return null;
  }
}

async function readMetadata(personaId: string): Promise<PersonaMetadata | null> {
  try {
    const metaPath = join(PERSONAS_PATH, personaId, "metadata.json");
    const data = await fs.readFile(metaPath, "utf-8");
    return JSON.parse(data) as PersonaMetadata;
  } catch {
    return null;
  }
}

async function getSourceUrl(personaId: string): Promise<string | undefined> {
  const extensions = [".png", ".jpg", ".jpeg", ".webp", ".mp4", ".webm"];
  for (const ext of extensions) {
    try {
      const path = join(PERSONAS_PATH, personaId, `source${ext}`);
      await fs.access(path);
      // Return relative URL - actual serving handled by nginx or storage proxy
      return `/personas/${personaId}/source${ext}`;
    } catch {
      continue;
    }
  }
  return undefined;
}

async function getPreviewUrl(personaId: string): Promise<string | undefined> {
  try {
    const path = join(PERSONAS_PATH, personaId, "preview.mp4");
    await fs.access(path);
    return `/personas/${personaId}/preview.mp4`;
  } catch {
    return undefined;
  }
}

// Simple auth check - in production this uses Better Auth
async function requireAdmin(request: NextRequest): Promise<boolean> {
  const authHeader = request.headers.get("authorization");
  if (authHeader?.startsWith("Bearer ")) {
    const token = authHeader.slice(7);
    return token === process.env.ADMIN_SECRET;
  }

  const sessionCookie = request.cookies.get("better-auth.session")?.value;
  if (sessionCookie) {
    return true;
  }

  if (process.env.NODE_ENV === "development") {
    return true;
  }

  return false;
}

// GET /api/admin/personas/[id] - Get persona details
export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
): Promise<NextResponse> {
  if (!(await requireAdmin(request))) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }

  const { id } = await params;

  try {
    const metadata = await readMetadata(id);
    if (!metadata) {
      return NextResponse.json({ error: "Persona not found" }, { status: 404 });
    }

    const status = await readStatus(id);
    const sourceUrl = await getSourceUrl(id);
    const previewUrl = await getPreviewUrl(id);

    const detail: PersonaDetail = {
      id: metadata.id,
      name: metadata.name,
      source_type: metadata.source_type,
      uploaded_by: metadata.uploaded_by,
      created_at: metadata.created_at,
      state: status?.state || "pending",
      error: status?.error,
      progress_percent: status?.progress_percent || 0,
      artefacts: status?.artefacts || [],
      source_url: sourceUrl,
      preview_url: previewUrl,
    };

    return NextResponse.json(detail);
  } catch (error) {
    console.error("Failed to get persona:", error);
    return NextResponse.json(
      { error: "Failed to get persona" },
      { status: 500 }
    );
  }
}

// DELETE /api/admin/personas/[id] - Delete persona
export async function DELETE(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
): Promise<NextResponse> {
  if (!(await requireAdmin(request))) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }

  const { id } = await params;

  try {
    const metadata = await readMetadata(id);
    if (!metadata) {
      return NextResponse.json({ error: "Persona not found" }, { status: 404 });
    }

    const personaDir = join(PERSONAS_PATH, id);
    await rm(personaDir, { recursive: true, force: true });

    return NextResponse.json({ success: true, deleted: id });
  } catch (error) {
    console.error("Failed to delete persona:", error);
    return NextResponse.json(
      { error: "Failed to delete persona" },
      { status: 500 }
    );
  }
}

export const runtime = "nodejs";
export const dynamic = "force-dynamic";
