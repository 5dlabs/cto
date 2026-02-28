import { ImageResponse } from "next/og";

export const runtime = "edge";
export const alt = "CTO by 5D Labs - AI Engineering Collective";
export const size = {
  width: 1200,
  height: 630,
};
export const contentType = "image/png";

export default function OpenGraphImage() {
  return new ImageResponse(
    (
      <div
        style={{
          height: "100%",
          width: "100%",
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          alignItems: "center",
          background:
            "radial-gradient(circle at 50% 38%, rgba(34,211,238,0.22) 0%, rgba(3,7,18,1) 52%), linear-gradient(180deg, #030712 0%, #020617 100%)",
          color: "#E2E8F0",
          padding: "56px",
          textAlign: "center",
        }}
      >
        <div
          style={{
            fontSize: 64,
            fontWeight: 800,
            letterSpacing: "-0.04em",
            color: "#F8FAFC",
            marginBottom: "14px",
          }}
        >
          CTO by 5D Labs
        </div>
        <div
          style={{
            fontSize: 30,
            fontWeight: 500,
            lineHeight: 1.3,
            color: "#93C5FD",
          }}
        >
          Multi-agent AI engineering collective
        </div>
        <div
          style={{
            fontSize: 24,
            fontWeight: 500,
            lineHeight: 1.35,
            color: "#A5B4FC",
            marginTop: "14px",
          }}
        >
          Build, ship, and run on bare metal
        </div>
      </div>
    ),
    {
      ...size,
    }
  );
}
