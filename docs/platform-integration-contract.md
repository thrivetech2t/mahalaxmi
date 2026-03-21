# Platform Integration Contract
## Website ↔ Product Platform: Full Interaction Specification

**Last updated:** 2026-03-20
**Branch:** integration
**Scope:** All interactions between mahalaxmi.ai (Next.js) and the product/orchestration platform.
**Forward context:** This document covers the current REST integration, federated orchestration extension points, and the full WebSocket-based communication dashboard (Mahalaxmi instance control plane) which is planned but not yet built.

---

## 1. Architecture Overview

The website is a **BFF (Backend-for-Frontend)** — every platform call originates server-side in Next.js API routes. The browser never touches the platform directly.

```
Browser
  │
  ├── /api/auth/*           ← auth proxy (JWT in httpOnly cookie)
  ├── /api/products/*       ← product catalog proxy
  ├── /api/releases/*       ← binary download proxy
  ├── /api/checkout         ← checkout session proxy
  ├── /api/mahalaxmi/checkout/session/* ← provisioning poll proxy
  ├── /api/mahalaxmi/servers/*           ← server management proxy
  ├── /api/mahalaxmi/billing/*           ← billing portal proxy
  └── /api/mahalaxmi/verification/*      ← verification proxy
         │
         │  (server-side only — PAK keys never leave the server)
         ▼
  MAHALAXMI_AUTH_API_URL        ← identity / auth service
  MAHALAXMI_PLATFORM_API_URL    ← orchestration platform (servers, products, billing)
```

---

## 2. Environment Variables

All backend URLs and credentials are injected at runtime via env vars. Nothing is hardcoded.

| Variable | Purpose |
|---|---|
| `MAHALAXMI_AUTH_API_URL` | Identity service base URL |
| `MAHALAXMI_PLATFORM_API_URL` | Platform/orchestration service base URL |
| `MAHALAXMI_CLOUD_PAK_KEY` | PAK key for headless cloud orchestration product |
| `MAHALAXMI_TERMINAL_PAK_KEY` | PAK key for AI terminal product |
| `MAHALAXMI_DESKTOP_PAK_KEY` | PAK key for desktop pro product |
| `MAHALAXMI_VSCODE_PAK_KEY` | PAK key for VS Code extension product |

PAK keys are **server-side only**. They never appear in browser responses, client bundles, or logs.

---

## 3. Authentication

### 3.1 Token Storage

- Cookie name: `mahalaxmi_token`
- Flags: `HttpOnly`, `Secure` (production), `SameSite=Lax`, `Max-Age=86400` (24 h)
- Contains a JWT with claims: `user`, `email`, `sub`, `exp`
- The browser cannot read this cookie — all auth state is surfaced via `/api/auth/me`

### 3.2 Middleware Route Protection

`src/middleware.js` guards:

- `/dashboard/**` → redirect to `/login?redirect={path}` if no cookie
- `/account/**` → same

### 3.3 Auth API Routes

All routes proxy to `MAHALAXMI_AUTH_API_URL`. Every call adds `clientId: 'mahalaxmi'`.

| Route | Method | Platform call | Notes |
|---|---|---|---|
| `/api/auth/login` | POST | `POST /v1/auth/login` | Sets `mahalaxmi_token` cookie |
| `/api/auth/register` | POST | `POST /v1/auth/register` | Sets token + `post_verify_redirect` cookie (30 min) |
| `/api/auth/me` | GET | `GET /v1/auth/me` | Validates JWT locally first; falls back to JWT payload if backend down |
| `/api/auth/logout` | POST | — | Clears cookie (maxAge=0), no platform call |
| `/api/auth/forgot-password` | POST | `POST /v1/auth/forgot-password` | |
| `/api/auth/reset-password` | POST | `POST /v1/auth/reset-password` | |
| `/api/auth/verify-email` | GET | `GET /v1/auth/verify-email?token=` | |
| `/api/auth/resend-verification` | POST | `POST /v1/auth/resend-verification` | |
| `/api/auth/account` | DELETE | `DELETE /v1/auth/account` | Clears cookie on success |

### 3.4 Auth Header Patterns

Two server-side helper functions in `src/lib/proxyHelpers.js` produce all auth headers:

```js
// User-authenticated calls
jwtHeaders(token)
// → { Authorization: "Bearer {jwt}", "Content-Type": "application/json" }

// PAK + user-authenticated calls (server management, billing)
pakAndJwtHeaders(pakKey, userToken)
// → { Authorization: "Bearer {pakKey}", "X-User-Token": "{jwt}", "Content-Type": "application/json" }
```

---

## 4. Product Catalog

### 4.1 PAK Key → Product Slug Mapping

| Slug | PAK Key |
|---|---|
| `mahalaxmi-ai-terminal-orchestration` | `MAHALAXMI_TERMINAL_PAK_KEY` |
| `mahalaxmi-ai-terminal-orchestration-pro` | `MAHALAXMI_DESKTOP_PAK_KEY` |
| `mahalaxmi-headless-orchestration` | `MAHALAXMI_CLOUD_PAK_KEY` |
| `mahalaxmi-vscode-extension` | `MAHALAXMI_VSCODE_PAK_KEY` |

### 4.2 Product API Routes

| Route | Platform call | Cache |
|---|---|---|
| `GET /api/products` | `GET /api/v1/public/product` (once per slug, parallel) | 30 s ISR |
| `GET /api/products/[slug]` | `GET /api/v1/public/product` | 30 s ISR |

Each call uses the PAK key for that product in `X-Channel-API-Key`.

### 4.3 Offering Fetch (Server-side lib)

`src/lib/productApi.js` exposes:

- `getDesktopProductOffering()` → `POST /api/v1/products/offering` with `MAHALAXMI_DESKTOP_PAK_KEY` (5 min cache)
- `getCloudProductOffering()` → same endpoint with `MAHALAXMI_CLOUD_PAK_KEY` (5 min cache)
- `getPricingTiers()` → derived from desktop offering
- `getProviderCatalog()` → derived from cloud offering

---

## 5. Binary Downloads

All download URLs are proxied. No direct binary URLs are ever given to the browser.

| Route | Platform call | Cache |
|---|---|---|
| `GET /api/releases/latest?platform=&architecture=` | `GET /api/v1/public/releases/latest` (TERMINAL_PAK_KEY) | 5 min |
| `GET /api/releases/download?id=` | `GET /api/v1/public/releases/{id}/download` (TERMINAL_PAK_KEY) | no-store |

The download proxy streams binary content, passing through `Content-Type`, `Content-Disposition`, and `Content-Length`. The platform returns 404 for unavailable platforms → website shows "Coming Soon".

---

## 6. Cloud Checkout & Provisioning

### 6.1 Buy Flow

```
User clicks Buy on /cloud/pricing
  │
  ├── isAuthenticated? (from AuthContext → /api/auth/me)
  │     No → redirect /register?redirect=/cloud/pricing
  │     Yes ↓
  │
  POST /api/checkout
    body: { tier, billing_cycle, success_url, cancel_url }
    cookie: mahalaxmi_token
  │
  Server:
    1. Extract JWT from cookie (401 if missing)
    2. Resolve PAK key from tier prefix (cloud-, desktop-, vscode-)
    3. Extract user email from JWT
    4. POST {PLATFORM}/api/v1/mahalaxmi/checkout/session
         Headers: Authorization: Bearer {pakKey}
                  X-Channel-API-Key: {pakKey}
                  X-User-Email: {email}
                  X-User-Token: Bearer {jwt}
         Body: { tier, billing_cycle, email, success_url, cancel_url,
                 cloud_provider: "hetzner" }
  │
  Response: { checkout_url: "https://checkout.stripe.com/..." }
  │
Browser redirects to Stripe → payment → Stripe redirects to success_url
```

**Fixed URLs (mahalaxmi.ai):**
```
success_url: https://mahalaxmi.ai/checkout/success?session_id={CHECKOUT_SESSION_ID}
cancel_url:  https://mahalaxmi.ai/cloud/pricing
```

### 6.2 Provisioning Poll

After Stripe payment, the user lands on `/checkout/success`. The page polls until provisioning completes:

```
GET /api/mahalaxmi/checkout/session/{sessionId}   (every 3 s, 10 min timeout)
  │
  Server → GET {PLATFORM}/api/v1/mahalaxmi/checkout/session/{sessionId}
    Headers: Authorization: Bearer {jwt}
             X-Channel-API-Key: MAHALAXMI_CLOUD_PAK_KEY
  │
  202 → still provisioning (show progress bar, estimated_ready_seconds)
  200 → terminal state:
        { status: 'active'|'failed', project_id, endpoint, deep_link,
          tier, cloud_provider }
  │
  On 'active': show endpoint + VS Code deep_link → auto-redirect to /dashboard in 5 s
```

---

## 7. Server / Machine Management (Dashboard)

This is the core of the federated orchestration surface. The dashboard at `/dashboard/servers` is the user's control plane.

### 7.1 Server List

```
GET /api/mahalaxmi/servers
  (polled every 5 s by ServersContent component)
  Headers (client → proxy): x-user-id, x-user-email
  │
  Server → GET {PLATFORM}/api/v1/mahalaxmi/servers
    Headers: Authorization: Bearer {jwt}
    Cache: no-store
  │
  Response: Array of server objects:
  {
    id, project_name, status, tier, cloud_provider,
    fqdn, created_at, is_configured, has_keep_warm,
    idle_timeout_minutes, ...
  }
```

### 7.2 Server Detail

```
GET /api/mahalaxmi/servers/{id}
  Server → GET {PLATFORM}/api/v1/mahalaxmi/servers/{id}
    Headers: jwt
```

### 7.3 Server Actions

| Action | Route | Method | Platform call |
|---|---|---|---|
| Configure (set project name) | `/api/mahalaxmi/servers/{id}/configure` | PATCH | `PATCH /api/v1/mahalaxmi/servers/{id}/configure` |
| Stop | `/api/mahalaxmi/servers/{id}/stop` | POST | `POST /api/v1/mahalaxmi/servers/{id}/stop` |
| Restart | `/api/mahalaxmi/servers/{id}/restart` | POST | `POST /api/v1/mahalaxmi/servers/{id}/restart` |
| Update timeout | `/api/mahalaxmi/servers/{id}/timeout` | PATCH | `PATCH /api/v1/mahalaxmi/servers/{id}/timeout` |
| Get VS Code config | `/api/mahalaxmi/servers/{id}/vscode-config` | GET | `GET /api/v1/mahalaxmi/servers/{id}/vscode-config` |
| Delete project | `/api/mahalaxmi/projects/{id}` | DELETE | `DELETE /api/v1/mahalaxmi/projects/{id}` (async 202) |

Stop and Restart use `pakAndJwtHeaders` (PAK key + user JWT). All others use JWT-only headers.

**Configure response** returns the updated server with `fqdn` and `is_configured: true`.
**VS Code config response:** `{ deep_link: "vscode://..." }`

### 7.4 Billing

```
POST /api/mahalaxmi/billing/portal-url
  Server → POST {PLATFORM}/api/v1/mahalaxmi/billing/portal
    Headers: pakAndJwtHeaders(CLOUD_PAK_KEY, userJwt)
  Response: { url: "https://billing.stripe.com/..." }
```

---

## 8. Student Verification

```
POST /api/mahalaxmi/verification/apply   body: { tier_id }
GET  /api/mahalaxmi/verification/status

Both proxy to: {PLATFORM}/api/v1/mahalaxmi/verification/*
Auth: jwt headers

If checkout returns { error: 'verification_required', status: 403 }
  → UI shows "Student verification required" gating message
```

---

## 9. Federated Orchestration — Extension Points

The following describes how the current integration maps to a federated orchestration model, where users manage machines across multiple providers and regions through the dashboard.

### 9.1 Current Model

- One server = one cloud project (Hetzner)
- Dashboard lists all servers for the authenticated user
- Each server has: `fqdn`, `status`, `tier`, `cloud_provider`, `project_name`
- Actions: start/stop/restart, configure name, get VS Code deep link

### 9.2 Federated Model — What Changes on the Platform Side

The platform needs to return federated machines alongside or in place of the current servers list. The website proxy at `/api/mahalaxmi/servers` is the integration point — no website changes are required if the platform returns federated machines in the same shape.

If the shape changes, the additions needed on the website side are:

| New field | Purpose |
|---|---|
| `node_type` | `'managed'` (current) vs `'federated'` (user-contributed) |
| `region` | Geographic region string for display |
| `provider` | Cloud provider slug (hetzner, aws, gcp, custom, ...) |
| `federation_id` | Opaque ID for federated cluster membership |
| `capabilities` | Array of strings: what workloads this node accepts |

### 9.3 Federated Onboarding — New Flows Needed

To allow users to register their own machines into the federation, the following new API routes will be needed on the website proxy layer:

```
POST /api/mahalaxmi/federation/register
  body: { name, host, port, auth_token, capabilities[] }
  → {PLATFORM}/api/v1/mahalaxmi/federation/nodes

GET  /api/mahalaxmi/federation/nodes
  → {PLATFORM}/api/v1/mahalaxmi/federation/nodes
  (list of all federated nodes for this user's account)

DELETE /api/mahalaxmi/federation/nodes/{id}
  → {PLATFORM}/api/v1/mahalaxmi/federation/nodes/{id}

GET /api/mahalaxmi/federation/nodes/{id}/status
  → real-time health: reachability, load, active sessions
```

All routes follow the existing proxy pattern: JWT auth, no PAK key needed for user-owned nodes.

### 9.4 Dashboard — UI Integration Points

The existing dashboard architecture supports federation with minimal changes:

- `ServersContent` polls `/api/mahalaxmi/servers` every 5 s — this stays as the unified fleet view
- `ServerCard` renders per-server action buttons — federation adds a `node_type` branch:
  - Managed nodes: existing stop/restart/configure actions
  - Federated nodes: deregister, health check, capability edit
- The configure flow (`PATCH .../configure`) already sets `project_name` — federated nodes use the same pattern for display naming

### 9.5 VS Code / Terminal Connection for Federated Nodes

The existing `vscode-config` endpoint returns a `deep_link` (`vscode://...`) for connecting to a managed server. Federated nodes need the same — the platform generates the deep link based on the node's `fqdn` and auth token. No website changes needed as long as the endpoint shape is the same.

### 9.6 Auth Boundary for Federated Machines

The current auth model (JWT in httpOnly cookie → forwarded to platform as Bearer token) is the correct boundary for federation:

- The website **never** holds credentials for federated machines
- The platform mediates all connections (it holds the node auth tokens)
- The user proves identity via `mahalaxmi_token`; the platform resolves which nodes they own

This means the dashboard can safely display and connect users to federated machines without the website handling any cross-machine credentials.

---

## 10. Data Security Summary

| Asset | Where stored | Exposure |
|---|---|---|
| `mahalaxmi_token` JWT | HttpOnly cookie | Server-side only; never in JS |
| PAK keys | Server env vars | Server-side only; never in responses |
| Binary download URLs | Proxied | Never sent to client; server streams bytes |
| Federated node tokens | Platform | Never touch the website layer |
| User email/id | JWT payload | Passed as optional metadata headers from client to proxy for logging only |

---

## 11. File Reference

| File | Role |
|---|---|
| `src/middleware.js` | Auth gate for `/dashboard`, `/account` |
| `src/contexts/AuthContext.jsx` | Client auth state (wraps `/api/auth/me`) |
| `src/lib/proxyHelpers.js` | `getUserToken`, `jwtHeaders`, `pakAndJwtHeaders`, `unauthorizedResponse` |
| `src/lib/productApi.js` | Server-side product/offering/release fetches |
| `src/lib/cloudPricing.js` | Transforms cloud offering → pricing display format |
| `src/lib/cloudConstants.js` | Tier labels, provider labels (server components) |
| `src/lib/api.js` | Client-side axios wrapper for all `/api/*` routes |
| `src/app/api/auth/*` | Auth proxy routes (8 routes) |
| `src/app/api/products/*` | Product catalog proxy (2 routes) |
| `src/app/api/releases/*` | Binary release proxy (2 routes) |
| `src/app/api/checkout/route.js` | Checkout session creation |
| `src/app/api/mahalaxmi/checkout/session/[sessionId]/route.js` | Provisioning status poll |
| `src/app/api/mahalaxmi/servers/*` | Full server management (7 sub-routes) |
| `src/app/api/mahalaxmi/projects/[id]/route.js` | Project delete |
| `src/app/api/mahalaxmi/billing/*` | Billing portal URL |
| `src/app/api/mahalaxmi/verification/*` | Student verification |
| `src/app/[locale]/dashboard/servers/ServersContent.jsx` | Fleet list UI (5 s poll) |
| `src/app/[locale]/dashboard/servers/ServerCard.jsx` | Per-server actions UI |
| `src/app/[locale]/checkout/success/MahalaxmiCheckoutSuccessContent.jsx` | Post-payment provisioning poll UI |
| `src/app/[locale]/cloud/pricing/BuyNowButton.jsx` | Auth gate + checkout initiation |
