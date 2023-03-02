/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  images: { unoptimized: true }, //https://nextjs.org/docs/messages/export-image-api
  basePath: "/open-data-viz",
};

module.exports = nextConfig;
