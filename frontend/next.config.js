//const isProduction = process.env.NODE_ENV === 'production'

/** @type {import('next').NextConfig} */
const nextConfig = {
  images: {
    unoptimized: true,
  },
  //basePath: '',
  output: "export",
  distDir: 'out',
  reactStrictMode: true,
}

module.exports = nextConfig;