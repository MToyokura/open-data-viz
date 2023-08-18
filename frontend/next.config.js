/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  images: { unoptimized: true }, //https://nextjs.org/docs/messages/export-image-api

  // ローカルで実行する場合は .env.local で LOCATION=local を設定する
  basePath: process.env.LOCATION === "local" ? "" : "/open-data-viz",
};

module.exports = nextConfig;
