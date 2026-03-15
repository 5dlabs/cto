import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Morgan Talking Avatar",
  description: "LiveKit and LemonSlice proof of concept for Morgan's talking avatar.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="antialiased">{children}</body>
    </html>
  );
}
