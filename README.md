![afbeelding](https://user-images.githubusercontent.com/33700526/207815865-9b471652-5723-4d35-8847-dce0fb9701eb.png)

# GAVL SVG to PNG Cloudflare Worker

Internal SVG-to-PNG rasterizer for GAVL, forked from
[`GewoonJaap/svg-to-png-cf-worker`](https://github.com/GewoonJaap/svg-to-png-cf-worker).

The deployed Worker has no public route or `workers.dev` URL. Auction Central
calls it through a Cloudflare service binding.

# Contract

- `POST /render`
- `Content-Type: image/svg+xml`
- Body: raw SVG, up to 10 MB and 4096×4096 / 16.7 million pixels
- Response: `image/png`

Remote URL fetching is intentionally unsupported. The caller must resolve and
inline trusted image assets before rasterization.

Liberation Sans Bold is bundled so native SVG text is converted to paths by
`usvg` in the fontless Workers runtime. Its SIL Open Font License is included
in `fonts/LICENSE_LIBERATION`.

# Development

Install Rust, `wasm32-unknown-unknown`, `worker-build`, and Wrangler, then run
`wrangler dev` or `wrangler deploy`.
