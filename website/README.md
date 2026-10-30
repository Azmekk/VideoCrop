# VideoCrop website

A single static page (`public/index.html`) served by a Cloudflare Worker using
[static assets](https://developers.cloudflare.com/workers/static-assets/). There is no build step
and no Worker script: `wrangler.toml` points the Worker at `public/`.

- `public/assets/downloads.js` reads the latest GitHub release and fills in the download button,
  version and size. It picks the x64 or ARM64 installer for the visitor.
- `public/_headers` sets cache and security headers.

## Run locally

```bash
cd website
npm install
npx wrangler dev        # http://localhost:8787
```

## Host on Cloudflare Workers

1. **Log in once:** run `npx wrangler login`. A browser window opens to authorise wrangler with your Cloudflare account.
2. **Deploy:** run `npx wrangler deploy` (or `npm run deploy`). The first deploy creates a Worker called `videocrop` and prints its `https://videocrop.<your-subdomain>.workers.dev` URL.
3. **Add your domain.** Use one of these:
   - **In `wrangler.toml`:** uncomment the `routes` line, adjust the hostname if needed, and run `npx wrangler deploy` again. Wrangler creates the DNS record and TLS certificate. The `vexbyte.com` zone must be on the same Cloudflare account.
   - **In the dashboard:** go to **Workers & Pages → videocrop → Settings → Domains & Routes → Add → Custom domain**, then enter `videocrop.vexbyte.com`.
4. **Hostname:** `index.html` assumes `https://videocrop.vexbyte.com/` for its `og:url`, `og:image` and `canonical` tags. If you use a different hostname, update those three lines.

To update the site later, edit `public/` and run `npx wrangler deploy` again. New app releases need no
site change, because the download button always points at the latest GitHub release.
