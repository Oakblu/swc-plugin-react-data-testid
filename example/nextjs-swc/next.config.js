/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  experimental: {
    swcPlugins: [
      ['swc-plugin-react-data-testid', {}]
    ]
  }
}

module.exports = nextConfig
