/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  images: { unoptimized: true }, //https://nextjs.org/docs/messages/export-image-api
  basePath: process.env.NODE_ENV === "production" ? "/open-data-viz" : "",
};

module.exports = nextConfig;
