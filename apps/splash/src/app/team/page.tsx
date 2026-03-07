import { redirect } from "next/navigation";

export default function TeamPage() {
  redirect(
    process.env.NODE_ENV === "development"
      ? "http://localhost:3002/team"
      : "https://cto.5dlabs.ai/team"
  );
}
