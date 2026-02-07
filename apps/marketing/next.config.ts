import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  images: {
    unoptimized: true, // Required for static export
  },
  trailingSlash: true,
  // Exclude API routes from static export (handled by Cloudflare Functions)
  skipTrailingSlashRedirect: true,
};

export default nextConfig;
