MAHALAXMI WEBSITE
Manager Requirements Document & Implementation Phases

	
Document Version	v1.3 — March 2026 (API contracts locked from codebase, billing stub scoped)
Repository	thrivetech2t/mahalaxmi-website (Private)
Domain	mahalaxmi.ai
Owner	ThriveTech Services LLC
Integration Branch	integration (set as default in GitHub repo settings)
Worker Branch Strategy	feat/<phase>-<task> branched from integration, merged back on completion
AI Providers	Claude (Manager + consensus) | Ollama (Workers)
Production Server	5.161.189.182 — /opt/deployments/mahalaxmi-website
Port	4025 (replaces mahalaxmi-web thin shell)

  v1.3 Changes from v1.2
API contracts locked from provisioning-service codebase: (1) /servers list returns bare array, status=active not ready, api_key in single GET only, is_configured drives Configure button | (2) Deep link pre-built server-side — website uses vscode-config endpoint directly | (3) 9 status values: pending_payment, provisioning, active, degraded, stopping, stopped, deleting, deleted, failed | (4) Delete route is /api/projects/:id not /servers/:id, returns 202 {status:deleting}, server stays in list | (5) /dashboard/billing scoped as stub — tier + Stripe portal link, full dashboard when metering ships | (6) Two 409 codes: name_taken (inline modal) and already_configured (hard error) | (7) Proxy forwards x-user-id + x-user-email headers injected by API gateway — no JWT decoding in website
 
1. Purpose and Scope
This document is the authoritative requirements brief for the Mahalaxmi orchestration team building mahalaxmi.ai. Every phase, task, file path, API contract, and acceptance criterion defined here must be followed precisely. Workers pull from integration, implement exactly what is specified, and merge back. The Manager coordinates consensus and resolves conflicts.
The mahalaxmi-website repo is a standalone Next.js 14 application — not a fork of thrivetechWebsite. Source files are migrated selectively from thrivetechWebsite during Phase 0. After migration, the Mahalaxmi subdirectories in thrivetechWebsite are frozen. All future Mahalaxmi UI work happens in mahalaxmi-website only.

2. Locked Decisions
2.1 Domain Architecture
Domain	Purpose
mahalaxmi.ai	All Mahalaxmi customer-facing pages — marketing, auth, dashboard, docs, open source, products
thrivetechservice.com	Company site only — never visible to Mahalaxmi users
tokenapi.thrivetechservice.com	Platform API — server-side only, never in browser
tokenadmin.thrivetechservice.com	Activation API — server-side only, never in browser

2.2 Auth Architecture
•	All auth calls proxy to Platform auth API: https://tokenapi.thrivetechservice.com/api/v1/auth/*
•	Pass client_id: "mahalaxmi" on register, forgot-password, and resend-verification — triggers Mahalaxmi-branded emails
•	JWT stored in httpOnly cookie named mahalaxmi_token scoped to mahalaxmi.ai
•	Mahalaxmi users have role="user" and company_id=NULL — they cannot access the Product Platform
•	The ThriveTech website auth system (thrivetech_db) is NOT used for Mahalaxmi users

2.3 Deep Link Parameter — LOCKED
⚠ The VS Code extension deep link uses api_key as the parameter name. NOT license_key. Confirmed at extension.ts:429.
vscode://thrivetech.mahalaxmi/configure?endpoint=https%3A%2F%2F<fqdn>%3A17421&api_key=<api_key>
Every worker referencing this link in Phase 2, Phase 4, or any other phase must use api_key.

2.4 Email Addresses
Address	Used For
support@mahalaxmi.ai	Customer support, contact forms, error states, server cards, product fallback message
sales@mahalaxmi.ai	Enterprise CTA on pricing page, sales inquiries
legal@mahalaxmi.ai	Terms of service, privacy policy, CLA, NOTICE file, footer
security@mahalaxmi.ai	/open-source/security vulnerability disclosure page only
⚠ No @thrivetechservice.com address may appear anywhere visible to a Mahalaxmi user.

2.5 Design System
Token	Value
Background	#0A2A2A — deep teal, dark theme throughout
Primary accent	#00C8C8 — cyan teal
Gold accent	#C8A040
Framework	Next.js 14, React 18, MUI 5, Tailwind, @tanstack/react-query, axios
Theme	MUI dark theme applied globally via ThemeProvider + CssBaseline in layout
Logo	Mahalaxmi + Ganesha combined image — navbar brand + hero
Navbar	Logo left | Features, Pricing, Products, Open Source, Docs center | Login + Get Started right
Footer	GitHub, VS Code Marketplace, Docs, Support, legal@mahalaxmi.ai, ThriveTech copyright

2.6 Environment Variables
⚠ All six must be present in .env.local and docker-compose.yml before any code runs.
MAHALAXMI_AUTH_API_URL=https://tokenapi.thrivetechservice.com
MAHALAXMI_PLATFORM_API_URL=https://tokenapi.thrivetechservice.com
MAHALAXMI_ACTIVATION_API_URL=https://tokenadmin.thrivetechservice.com
MAHALAXMI_CLOUD_PAK_KEY=pak_live_...        # mahalaxmi-headless-orchestration
MAHALAXMI_TERMINAL_PAK_KEY=pak_live_...     # mahalaxmi-ai-terminal-orchestration
MAHALAXMI_VSCODE_PAK_KEY=pak_live_...     # mahalaxmi-vscode-extension
NOTE: AUTH_API_URL and PLATFORM_API_URL currently point to the same server. Both are defined separately so infrastructure can split them without a code change. AUTH_API_URL is for JWT verification only. PLATFORM_API_URL is for checkout, server, and product operations.
NOTE: MAHALAXMI_TERMINAL_PAK_KEY value already exists in thrivetechWebsite/backend/.env under PRODUCT_PLATFORM_PAK_KEYS. Copy it — do not generate a new one.

2.7 Products — PAK Key Map and CTA Rules
Product Slug	Env Var	Primary CTA	CTA Target
mahalaxmi-ai-terminal-orchestration	MAHALAXMI_TERMINAL_PAK_KEY	Download	GitHub releases URL from Platform response
mahalaxmi-headless-orchestration	MAHALAXMI_CLOUD_PAK_KEY	Get Started	/cloud/pricing
mahalaxmi-vscode-extension	MAHALAXMI_VSCODE_PAK_KEY	Install in VS Code	VS Code Marketplace placeholder
NOTE: VS Code Extension PAK key is live. All three products are active and fetch from the Platform in parallel.
NOTE: Stripe hosted checkout — same flow as BuyNowButton: auth-gated, server-side PAK, redirect to Stripe hosted checkout, return to /checkout/success.

3. Repository Setup
3.1 Current State of /home/alex/thrivetech/mahalaxmi-website
•	Files present: .gitignore, CLAUDE.md, README.md (47-byte stub)
•	No Next.js app, no src/, no package.json
•	Only main branch — integration branch does not exist yet
•	GitHub remote connected: origin → https://github.com/thrivetech2t/mahalaxmi-website.git

3.2 Phase 0 Bootstrap — Manual Setup
⚠ Do NOT run npx create-next-app. It fails on non-empty directories. Use manual package.json setup.
1.	git checkout -b integration && git push -u origin integration
2.	Set integration as the default branch in GitHub repository settings
3.	Create package.json manually — next@14, react@18, typescript, @mui/material@5, @mui/icons-material@5, @mui/material-nextjs@5, @emotion/react, @emotion/styled, next-intl, @tanstack/react-query, axios
4.	Create tsconfig.json with paths: "@/*" → "./src/*"
5.	Create tailwind.config.ts, postcss.config.js, next.config.js (withNextIntl wrapper, output: "standalone")
6.	Run npm install
7.	Create src/app/layout.tsx — ThemeProvider (dark, #0A2A2A, #00C8C8 primary), CssBaseline, QueryClientProvider, Navbar, Footer
8.	Create src/app/globals.css — base dark styles
9.	Create src/components/Navbar.tsx and Footer.tsx per Section 2.5
10.	Create stub AuthContext.jsx per Section 3.5
11.	Create stub /api/auth/me/route.js per Section 3.5
12.	Create src/lib/api.js per Section 4.5
13.	Copy i18n files and messages/*.json per Section 3.3
14.	Migrate all source files per Section 3.3 — update imports, remove /mahalaxmi prefix
15.	Write new middleware.js per Section 3.4 — auth guard + next-intl only
16.	Create .env.local (six vars) and .env.example (key names + comments, values blank)
17.	Create Dockerfile and docker-compose.yml per Section 3.6
18.	Run npx next build — must exit 0 with zero errors
19.	Push to integration

3.3 Files to Migrate from thrivetechWebsite
All source paths relative to thrivetech/thrivetechWebsite/frontend/src/. Destination relative to mahalaxmi-website/src/.

i18n and locale:
•	i18n/routing.ts → src/i18n/routing.ts (verbatim)
•	i18n/request.js → src/i18n/request.js (verbatim)
•	messages/*.json (all 10 locale files) → messages/*.json (verbatim)
•	utils/i18nMetadata.js → src/utils/i18nMetadata.js — CHANGE SITE_URL to https://mahalaxmi.ai

Marketing pages:
•	app/[locale]/mahalaxmi/page.js → app/[locale]/page.js
•	app/[locale]/mahalaxmi/features/page.js → app/[locale]/features/page.js
•	app/[locale]/mahalaxmi/pricing/page.js → app/[locale]/pricing/page.js
•	app/[locale]/mahalaxmi/use-cases/page.js → app/[locale]/use-cases/page.js
•	app/[locale]/mahalaxmi/whitepaper/page.js → app/[locale]/whitepaper/page.js
•	app/[locale]/mahalaxmi/cloud/page.js → app/[locale]/cloud/page.js
•	app/[locale]/mahalaxmi/cloud/pricing/page.js → app/[locale]/cloud/pricing/page.js
•	app/[locale]/mahalaxmi/cloud/pricing/BuyNowButton.jsx → same relative path
•	app/[locale]/mahalaxmi/cloud/pricing/CloudPricingDisplay.jsx → same relative path

Products pages (verbatim — zero component changes needed):
•	app/[locale]/products/page.js → app/[locale]/products/page.js
•	app/[locale]/products/ProductsContent.js → same
•	app/[locale]/products/loading.js → same
•	app/[locale]/products/[slug]/page.js → same
•	app/[locale]/products/[slug]/ProductDetailContent.js → same

Checkout and dashboard:
•	app/[locale]/mahalaxmi/checkout/success/page.js → app/[locale]/checkout/success/page.js
•	app/[locale]/mahalaxmi/checkout/success/MahalaxmiCheckoutSuccessContent.jsx → same relative path
•	app/[locale]/mahalaxmi/dashboard/servers/page.js → app/[locale]/dashboard/servers/page.js
•	app/[locale]/mahalaxmi/dashboard/servers/ServersContent.jsx → same relative path
•	app/[locale]/mahalaxmi/dashboard/servers/ProjectNameModal.jsx → same relative path

Legal pages:
•	app/[locale]/legal/mahalaxmi/privacy/page.js → app/[locale]/legal/privacy/page.js — update to legal@mahalaxmi.ai
•	app/[locale]/legal/mahalaxmi/terms/page.js → app/[locale]/legal/terms/page.js — update to legal@mahalaxmi.ai

API proxy routes:
•	app/api/checkout/route.js → app/api/checkout/route.js
•	app/api/checkout/session/[sessionId]/route.js → same
•	app/api/mahalaxmi/servers/route.js → same
•	app/api/mahalaxmi/servers/[id]/route.js → same
•	app/api/mahalaxmi/servers/[id]/configure/route.js → same

Assets:
•	public/mahalaxmi_logo.png and public/mahalaxmi_logo.jpg → public/

3.4 Middleware — New Implementation
⚠ Remove the MAHALAXMI_PATH_MAP entirely. That was for multi-tenant routing in thrivetechWebsite only.
New middleware.js handles two concerns only:
1.	Protected route guard: /dashboard/* and /account/* with no mahalaxmi_token cookie → redirect to /login?redirect=<path>
2.	next-intl locale handling for all routes
NOTE: Unauthenticated /dashboard/servers redirects to /login which returns 404 until Phase 1. This is expected — not a bug.
NOTE: Navbar links to /open-source and /docs return 404 until Phases 3 and 4 complete respectively. Do not add stubs.

3.5 Phase 0 Stub Files
These two stubs prevent build failures in Phase 0. Both are replaced completely in Phase 1.

src/contexts/AuthContext.jsx:
"use client";
import { createContext, useContext } from "react";
const ctx = { user: null, isAuthenticated: false, login: async () => {}, logout: async () => {} };
const AuthContext = createContext(ctx);
export function AuthProvider({ children }) { return <AuthContext.Provider value={ctx}>{children}</AuthContext.Provider>; }
export function useAuth() { return useContext(AuthContext); }

src/app/api/auth/me/route.js:
import { NextResponse } from "next/server";
export async function GET() { return NextResponse.json({ user: null, isAuthenticated: false }); }

3.6 Docker and Deployment
•	Dockerfile: Next.js standalone build — COPY .next/standalone ./, COPY .next/static ./.next/static, EXPOSE 4025, CMD ["node", "server.js"]
•	docker-compose.yml: service mahalaxmi_web, build: ., ports: ["4025:3000"], env_file: .env.local, restart: unless-stopped
•	Existing Nginx config proxies mahalaxmi.ai → 127.0.0.1:4025 — no Nginx changes required
•	Deploy: SSH to 5.161.189.182, cd /opt/deployments/mahalaxmi-website, git pull origin integration, docker compose up --build -d
•	After confirming healthy: stop old mahalaxmi-web container in /opt/deployments/thrivetech-website

4. API Contracts
All proxy routes are server-side Next.js route handlers. PAK keys and JWT are never sent to the browser.
4.1 Auth Proxy Routes
Route	Platform Endpoint	Notes
POST /api/auth/register	POST /api/v1/auth/register	Add clientId: "mahalaxmi" to body
POST /api/auth/login	POST /api/v1/auth/login	Set httpOnly cookie mahalaxmi_token on success
POST /api/auth/logout	(local only)	Clear mahalaxmi_token cookie, return 200
GET /api/auth/me	GET /api/v1/auth/me	Forward mahalaxmi_token as Bearer. Stub returns {user:null} in Phase 0
POST /api/auth/forgot-password	POST /api/v1/auth/forgot-password	Add clientId: "mahalaxmi" to body
POST /api/auth/reset-password	POST /api/v1/auth/reset-password	Forward body verbatim
POST /api/auth/resend-verification	POST /api/v1/auth/resend-verification	Add clientId: "mahalaxmi" to body

4.2 Server Proxy Routes — Status Lifecycle
Complete status lifecycle (9 values):
pending_payment → provisioning → active
                               active → degraded → failed  (terminal, unrecoverable)
                               active/degraded → stopping → stopped  (user stop OR idle timeout)
                               stopped → provisioning  (user restart)
                               any → deleting → deleted  (user delete)

Status	Meaning	UI Treatment
pending_payment	Stripe checkout not yet confirmed	Progress bar, "Awaiting payment"
provisioning	VM being created on Hetzner	Progress bar, animated
active	Running — VS Code can connect	Green badge "Active"
degraded	Running but unhealthy	Yellow badge "Degraded"
stopping	Shutdown in progress	Progress bar, "Stopping"
stopped	VM destroyed, subscription preserved, restartable	Gray badge "Stopped" + Restart button
deleting	Teardown in progress	Progress bar, "Deleting", disable all buttons
deleted	Fully torn down — remains in list	Gray badge "Deleted", no actions
failed	Terminal error — unrecoverable	Red badge "Failed" + support@mahalaxmi.ai link
NOTE: stopped = VM is destroyed but subscription is preserved. User can restart (moves back to provisioning). failed = unrecoverable — user must contact support. Idle timeout auto-stops active servers when VS Code heartbeat goes quiet.

GET /api/v1/mahalaxmi/servers response shape — bare array, no envelope: — bare array, no envelope:
[ { "id": "uuid", "project_name": "my-project" | null, "fqdn": "my-project.mahalaxmi.ai" | null,
    "status": "pending_payment"|"provisioning"|"active"|"degraded"|"stopping"|"stopped"|"deleting"|"deleted"|"failed",
    "tier": "Cloud Builder", "created_at": "2026-...", "is_configured": true|false } ]
⚠ api_key is NOT in the list response. It is only returned by GET /api/v1/mahalaxmi/servers/:id.
GET /api/v1/mahalaxmi/servers/:id/vscode-config response shape:
{ "deep_link": "vscode://thrivetech.mahalaxmi/configure?endpoint=https://my-project.mahalaxmi.ai:17421&api_key=...",
  "config_json": { "endpoint": "https://my-project.mahalaxmi.ai:17421", "api_key": "mhx_..." } }
NOTE: Deep link is pre-built server-side. The website uses deep_link directly — no client-side URL construction needed.

Route	Platform Endpoint	Auth Headers
GET /api/mahalaxmi/servers	GET /api/v1/mahalaxmi/servers	PAK (CLOUD) + x-user-id (gateway header)
GET /api/mahalaxmi/servers/:id	GET /api/v1/mahalaxmi/servers/:id	PAK (CLOUD) + x-user-id
PATCH /api/mahalaxmi/servers/:id/configure	PATCH .../configure	PAK (CLOUD) + x-user-id
DELETE /api/mahalaxmi/projects/:id	DELETE /api/v1/mahalaxmi/projects/:id — 202 async	PAK (CLOUD) + x-user-id
GET /api/mahalaxmi/servers/:id/vscode-config	GET .../vscode-config	PAK (CLOUD) + x-user-id

⚠ DELETE route is /api/projects/:id not /api/servers/:id. Returns 202 { status: "deleting" } — NOT 204. Deletion is async. Server remains in list with deleted status until Hetzner VM + DNS + Stripe are torn down. If server is still provisioning when delete is requested, deletion is deferred — a stale-provisioning worker completes it later.
NOTE: The website proxy forwards x-user-id and x-user-email as plain headers — these are injected by the API gateway after JWT validation. The website does NOT decode the JWT itself. Extract x-user-id from the incoming gateway-forwarded headers and pass them through.

4.3 Billing and Checkout Proxy Routes
Route	Platform Endpoint	Notes
GET /api/mahalaxmi/billing/portal-url	POST /api/v1/mahalaxmi/billing/portal	PAK (CLOUD) + x-user-id — returns { url } for Stripe portal redirect
GET /api/checkout	GET /api/v1/public/product	PAK (CLOUD) only — replaces cloudPricing.js fetch
POST /api/checkout	POST /api/v1/mahalaxmi/checkout/session	PAK (CLOUD) only
GET /api/checkout/session/:id	GET .../checkout/session/:id	PAK (CLOUD) only — polling
NOTE: GET /api/checkout is a NEW route that replaces the cloudPricing.js fetch pattern from thrivetechWebsite. Returns cloud tier pricing server-side via PAK to CloudPricingDisplay.
NOTE: Billing endpoint: GET /api/v1/mahalaxmi/billing does NOT exist at launch. Full billing dashboard (cycle usage, overage, invoices) ships when cycle metering is enabled. Phase 2 builds a stub only — see Section 5.4.

4.4 Products Catalog Proxy Routes
These routes replicate the ThriveTech Express backend products/categories pattern. Response shapes match exactly what ProductsContent.js and ProductDetailContent.js already consume — zero component changes needed.

GET /api/categories — hardcoded, no Platform call:
const CATEGORIES = [
  { id: "cat-terminal", name: "Terminal Orchestration", slug: "terminal-orchestration",
    description: "Local AI terminal orchestration — runs on your machine",
    icon: "psychology", color: "#00C8C8", product_count: 1 },
  { id: "cat-cloud", name: "Cloud Orchestration", slug: "cloud-orchestration",
    description: "Fully managed cloud servers — connect VS Code in one click",
    icon: "cloud", color: "#C8A040", product_count: 1 },
  { id: "cat-vscode", name: "VS Code Extension", slug: "vscode-extension",
    description: "Orchestration inside your editor",
    icon: "code", color: "#00C8C8", product_count: 1 },
];

PAK key map and fetchPlatformProduct helper:
const PAK_MAP = {
  "mahalaxmi-ai-terminal-orchestration": { key: process.env.MAHALAXMI_TERMINAL_PAK_KEY,
    category_id: "cat-terminal", category_name: "Terminal Orchestration",
    image: "/mahalaxmi_logo.png", is_featured: true },
  "mahalaxmi-headless-orchestration": { key: process.env.MAHALAXMI_CLOUD_PAK_KEY,
    category_id: "cat-cloud", category_name: "Cloud Orchestration",
    image: "/mahalaxmi_logo.png", is_featured: true },
  "mahalaxmi-vscode-extension": { key: process.env.MAHALAXMI_VSCODE_PAK_KEY,
    category_id: "cat-vscode", category_name: "VS Code Extension",
    image: "/mahalaxmi_logo.png", is_featured: false },
};

async function fetchPlatformProduct(slug, meta) {
  try {
    const res = await fetch(`${process.env.MAHALAXMI_PLATFORM_API_URL}/api/v1/public/product`,
      { headers: { "X-Channel-API-Key": meta.key }, next: { revalidate: 30 } });
    if (!res.ok) throw new Error();
    const data = await res.json();
    return { ...data, slug, ...meta, is_platform_connected: true, data_source: "platform" };
  } catch {
    const names = {
      "mahalaxmi-ai-terminal-orchestration": "Mahalaxmi AI Terminal Orchestration",
      "mahalaxmi-headless-orchestration": "Mahalaxmi Headless Orchestration",
      "mahalaxmi-vscode-extension": "Mahalaxmi VS Code Extension",
    };
    return { slug, ...meta, pricing_options: [], pricing_type: "unavailable",
      name: names[slug] || slug,
      is_platform_connected: false, data_source: "placeholder",
      platform_status_message: "Pricing temporarily unavailable. Contact support@mahalaxmi.ai" };
  }
}

Route	Behavior	PAK Key(s)
GET /api/categories	Return hardcoded CATEGORIES array — no Platform call	None
GET /api/products	Fetch all 3 products in parallel. Filter by ?category= slug if provided.	TERMINAL + CLOUD + VSCODE
GET /api/products/:slug	Single product via PAK_MAP. Return 404 if slug not in map.	Slug-specific
GET /api/releases/latest	Proxy /api/v1/public/releases/latest. Used by ProductDetailContent.js for Download button.	TERMINAL

Response shape expected by ProductsContent.js:
{ "success": true, "data": { "data": { "products": [...] } } }

4.5 src/lib/api.js — Axios Client
Same structure as thrivetechWebsite frontend/src/lib/api.js stripped to routes that exist in this repo. Same-origin calls — no NEXT_PUBLIC_API_URL needed.
import axios from "axios";
const apiClient = axios.create({ baseURL: "/api", timeout: 10000 });
export const productsAPI = {
  getAll: (params = {}) => apiClient.get("/products", { params }),
  getBySlug: (slug) => apiClient.get(`/products/${slug}`),
};
export const categoriesAPI = { getAll: () => apiClient.get("/categories") };
export const releasesAPI = { getLatest: () => apiClient.get("/releases/latest") };
export const checkoutAPI = {
  getPricing: () => apiClient.get("/checkout"),
  createSession: (data) => apiClient.post("/checkout", data),
  getSession: (id) => apiClient.get(`/checkout/session/${id}`),
};

5. Complete Page Inventory
5.1 Marketing Pages — migrate + update
Route	Description	Source
/	Hero, logo, value prop, CTA to /pricing and /open-source	Migrate page.js
/features	Full feature reference	Migrate features/page.js
/pricing	Desktop pricing — live from Platform via PAK	Migrate pricing/page.js
/use-cases	10 use cases	Migrate
/whitepaper	Technical whitepaper	Migrate
/cloud	Cloud SaaS landing	Migrate cloud/page.js
/cloud/pricing	Cloud pricing — live from GET /api/checkout	Migrate cloud/pricing/*

5.2 Products Pages — migrate verbatim
Route	Description	Notes
/products	Catalog — categories sidebar, product cards, live from /api/products	Migrate ProductsContent.js verbatim
/products/mahalaxmi-ai-terminal-orchestration	Desktop detail — tiers, Download CTA, releases data	Migrate ProductDetailContent.js verbatim
/products/mahalaxmi-headless-orchestration	Cloud detail — tiers, Get Started → /cloud/pricing	Migrate ProductDetailContent.js verbatim
/products/mahalaxmi-vscode-extension	VS Code Extension detail — tiers, Install in VS Code CTA → Marketplace placeholder	Migrate ProductDetailContent.js verbatim
NOTE: All three product detail pages are built in Phase 0 migration using ProductDetailContent.js verbatim.

5.3 Auth Pages — Phase 1 new build
Route	Behavior
/register	firstName, lastName, email, password → POST /api/auth/register with clientId: "mahalaxmi" → /verify-email
/login	email, password → POST /api/auth/login → mahalaxmi_token cookie → read ?redirect param → redirect. Default: /dashboard/servers
/verify-email	Notice + resend → POST /api/auth/resend-verification with clientId: "mahalaxmi"
/forgot-password	Email → POST /api/auth/forgot-password with clientId: "mahalaxmi"
/reset-password	New password → POST /api/auth/reset-password

5.4 Dashboard Pages — Phase 2, protected
Route	Description
/dashboard/servers	Server list — poll GET /api/mahalaxmi/servers every 5s. ServersContent.jsx + ProjectNameModal.jsx
/dashboard/billing	Billing stub — current tier name + Manage Billing button → Stripe Customer Portal (GET /api/mahalaxmi/billing/portal-url). Full dashboard ships when cycle metering enabled.
/dashboard/account	Email display, change password, delete account with confirmation

5.5 Server Card Spec — Phase 2
•	Project name + FQDN (my-project.mahalaxmi.ai)
•	Status badge: Provisioning / Ready / Stopped / Error — color coded
•	Linear progress bar while status = Provisioning
•	Tier label: Cloud Solo / Cloud Builder / Cloud Power / Cloud Team
•	Configure button — shown when is_configured = false (not null check on project_name)
•	Open in VS Code button — shown when status = active AND is_configured = true
NOTE: Deep link is NOT constructed client-side. Call GET /api/mahalaxmi/servers/:id/vscode-config and use the pre-built deep_link field from the response. The api_key is never exposed in the list response — only retrieved via vscode-config.
•	Copy config fallback — copies config_json.endpoint + config_json.api_key from vscode-config response to clipboard
•	Stop button with confirmation modal — calls stop endpoint
•	Delete button with confirmation modal → DELETE /api/mahalaxmi/projects/:id — returns 202, server moves to deleting status, stays in list
⚠ Delete calls /api/projects/:id NOT /api/servers/:id. Response is 202 { status: "deleting" } — poll the list to confirm deleted status.
•	Error state shows support@mahalaxmi.ai contact link

5.6 Open Source Section — Phase 3
Route	Content
/open-source	Landing — hero, GitHub link, VS Code Marketplace placeholder, MIT badge, Provider SDK callout
/open-source/docs	Getting started, 3-command quick start, prerequisites
/open-source/architecture	Manager-Worker consensus engine, DAG overview, PTY control, provider routing
/open-source/providers	Claude Code, Copilot, Grok, Ollama, Gemini, Provider Plugin SDK
/open-source/contributing	CLA requirement, scope definition, CONTRIBUTING.md link
/open-source/roadmap	Phase timeline, AWS ECS, GCP, cycle metering, Phase 28
/open-source/changelog	Version history — static markdown at launch
/open-source/security	Responsible disclosure — 48h SLA, 14d patch SLA, security@mahalaxmi.ai, no public GitHub issues for vulns

5.7 Docs Section — Phase 4
Route	Content
/docs	Getting started hub — links to all doc sections
/docs/quickstart	Install → configure one provider → run first cycle → review PRs
/docs/vscode	VS Code extension install, connect to cloud server, plan approval, file acceptance
/docs/cloud	Cloud server setup, project name, deep link walkthrough
/docs/api	Headless API reference for power users
/docs/faq	Common questions

5.8 Content Pages — Phase 5
Route	Content
/support	Contact form → support@mahalaxmi.ai, FAQ accordion
/about	ThriveTech Services LLC, product story, Ganesha → Mahalaxmi evolution
/legal/terms	Migrated — update to legal@mahalaxmi.ai, remove ThriveTech branding
/legal/privacy	Migrated — update to legal@mahalaxmi.ai, remove ThriveTech branding

6. Implementation Phases
Each phase has a completion gate. Workers branch from integration, implement, run npx next build, merge back. Manager verifies gate before next phase starts.

  PHASE 0: FOUNDATION   1-2 days   Depends: None — blocks all other phases
Branch: feat/phase0-foundation | Single worker
1.	Create integration branch, push to origin, set as default in GitHub
2.	Manual package.json setup — do NOT run create-next-app
3.	Create tsconfig.json, tailwind.config.ts, postcss.config.js, next.config.js (withNextIntl, standalone output)
4.	Run npm install (includes @tanstack/react-query, axios)
5.	Create layout.tsx with ThemeProvider, CssBaseline, QueryClientProvider, Navbar, Footer
6.	Create stub AuthContext.jsx per Section 3.5
7.	Create stub /api/auth/me/route.js per Section 3.5
8.	Create src/lib/api.js per Section 4.5
9.	Copy i18n files and all 10 messages/*.json locale files per Section 3.3
10.	Copy utils/i18nMetadata.js — update SITE_URL to https://mahalaxmi.ai
11.	Migrate all source files per Section 3.3 — remove /mahalaxmi prefix from imports and routes
12.	Write new middleware.js per Section 3.4 — auth guard + next-intl, no MAHALAXMI_PATH_MAP
13.	Create .env.local (six vars) and .env.example (key names + comments)
14.	Create Dockerfile and docker-compose.yml per Section 3.6
15.	Run npx next build — must exit 0
16.	Push to integration

Completion gate:
•	npx next build exits 0 — zero errors, zero warnings
•	/products renders (placeholder data acceptable if PAK keys not set)
•	Middleware redirects /dashboard/servers to /login (404 on /login is acceptable until Phase 1)
•	grep -r "thrivetechservice.com" src/ — zero user-visible results (proxy routes excepted)

  PHASE 1: AUTH PAGES   1 day   Depends: Phase 0
Branch: feat/phase1-auth | Single worker
1.	Build real AuthContext.jsx — fetches GET /api/auth/me on mount, exposes user, isAuthenticated, login(), logout()
2.	Build real /api/auth/me route — proxies to Platform with mahalaxmi_token as Bearer
3.	Build all auth proxy routes per Section 4.1
4.	Build /register, /login (with ?redirect handling), /verify-email, /forgot-password, /reset-password
5.	Wire BuyNowButton.jsx and /products Buy Now buttons to real AuthContext — redirect to /login?redirect=<path>&tier=<tier> if unauthenticated
6.	Run npx next build — must exit 0
7.	Push to integration

Completion gate:
•	Register → Mahalaxmi-branded verification email (dark header, #00C9A7 teal CTA, mahalaxmi.ai links)
•	Verify → login → redirect to /dashboard/servers (404 until Phase 2 — redirect is correct)
•	Buy Now on /products and /cloud/pricing redirects to /login when unauthenticated

  PHASE 2: DASHBOARD   2 days   Depends: Phase 1
Branch: feat/phase2-dashboard | Single worker
1.	Build all server and billing proxy routes per Sections 4.2 and 4.3
2.	Build /dashboard/servers — wire ServersContent.jsx to GET /api/mahalaxmi/servers, 5s polling
3.	Implement full server card per Section 5.5 — all states, deep link uses api_key
4.	Build ProjectNameModal — PATCH configure, slug validation (URL-safe, 3-40 chars)
5.	Handle both 409 codes from configure endpoint: name_taken → inline error "That name is already taken"; already_configured → hard error, close modal, refresh server list
6.	Build /dashboard/billing stub — show tier name, GET /api/mahalaxmi/billing/portal-url, redirect to Stripe portal on click
7.	Build /dashboard/account — email display, change password, delete account with confirmation
8.	Run npx next build — must exit 0
9.	Push to integration

Completion gate:
•	/dashboard/servers empty state — no console errors
•	Server card renders all states correctly — deep link confirmed to use api_key
•	Delete shows confirmation modal before calling DELETE

  PHASE 3: OPEN SOURCE SECTION   2 days   Depends: Phase 0 (parallel with P4, P5)
Branch: feat/phase3-open-source | Build all 8 pages per Section 5.6. security@mahalaxmi.ai must be prominent on /open-source/security. npx next build clean. Push to integration.

  PHASE 4: DOCS SECTION   1 day   Depends: Phase 0 (parallel with P3, P5)
Branch: feat/phase4-docs | Build all 6 pages per Section 5.7. /docs/quickstart must show 3-command quick start. npx next build clean. Push to integration.

  PHASE 5: CONTENT AND LEGAL PAGES   1 day   Depends: Phase 0 (parallel with P3, P4)
Branch: feat/phase5-content | Build /support (→ support@mahalaxmi.ai), /about, migrate /legal/terms and /legal/privacy with legal@mahalaxmi.ai. npx next build clean. Push to integration.

  PHASE 6: MARKETING + PRODUCTS UPDATES   1 day   Depends: Phase 1 (auth gate required)
Branch: feat/phase6-marketing | Single worker
1.	Update / — confirm hero links and logo
2.	Update /pricing — live pricing from GET /api/checkout (new GET route), Buy Now auth-gated
3.	Update /cloud/pricing — CloudPricingDisplay uses new GET /api/checkout, Buy Now auth-gated
4.	Confirm /products renders live data for all 3 products from /api/products and /api/categories
5.	Confirm /products/mahalaxmi-ai-terminal-orchestration shows Download CTA from releases data
6.	Confirm /products/mahalaxmi-headless-orchestration shows Get Started → /cloud/pricing
7.	Confirm /products/mahalaxmi-vscode-extension shows Install in VS Code CTA → Marketplace placeholder
8.	Update Navbar — confirm Products link present alongside Features, Pricing, Open Source, Docs
9.	Remove any /mahalaxmi prefix leaks in migrated pages
10.	Run npx next build — must exit 0
11.	Push to integration

  PHASE 7: SMOKE TEST AND DEPLOYMENT   1 day   Depends: All phases complete
Branch: feat/phase7-deploy | Manager-supervised

All 13 checks must pass before deploying:
1.	npx next build on integration — zero errors, zero warnings
2.	grep -r "thrivetechservice.com" src/ — zero user-visible results
3.	Register → Mahalaxmi-branded email → verify → login → /dashboard/servers loads
4.	/dashboard/servers empty state — no console errors
5.	Server card Open in VS Code button calls vscode-config endpoint — deep_link used directly, not constructed
6.	/products loads all 3 products with live pricing
7.	/products/mahalaxmi-ai-terminal-orchestration shows Download CTA
8.	/products/mahalaxmi-headless-orchestration shows Get Started → /cloud/pricing
9.	/products/mahalaxmi-vscode-extension shows Install in VS Code CTA
10.	Buy Now on /cloud/pricing → /login when unauthenticated
11.	/open-source loads with GitHub link and MIT badge
12.	/open-source/security shows security@mahalaxmi.ai
13.	/docs/quickstart shows 3-command quick start
14.	Footer renders all links — including legal@mahalaxmi.ai
15.	All six env vars confirmed in .env.local and docker-compose.yml on production server

Deploy steps:
1.	SSH to 5.161.189.182
2.	mkdir -p /opt/deployments/mahalaxmi-website && cd /opt/deployments/mahalaxmi-website
3.	git clone https://github.com/thrivetech2t/mahalaxmi-website.git . (first deploy only)
4.	git pull origin integration
5.	Copy .env.local with production values
6.	docker compose up --build -d
7.	Confirm mahalaxmi.ai returns 200 — no redirect to thrivetechservice.com
8.	Stop old mahalaxmi-web container in /opt/deployments/thrivetech-website once confirmed healthy

7. Out of Scope — Phase 2+
•	Cycle metering usage display on dashboard cards
•	AWS ECS or GCP Cloud Run provider selection on pricing page
•	Team / multi-seat management
•	Blog or news section
•	Enterprise contact / custom pricing form
•	VS Code Marketplace live link — placeholder only until extension published
•	OAuth single sign-on across ThriveTech products

8. Worker Rules
1.	Read this entire document before writing a single line of code
2.	Confirm all six env vars are present in .env.local before starting
3.	Branch from integration using the exact branch name specified for the phase
4.	Run npx next build before pushing — never push a build that fails
5.	Never hardcode API URLs — always use the locked env vars
6.	Never expose PAK keys or JWT to the browser — server-side proxy routes only
7.	Never use @thrivetechservice.com in any user-visible content
8.	Deep link uses api_key — never license_key (Section 2.3)
9.	Do NOT run create-next-app — repo is non-empty (Section 3.2)
10.	Do not add files in scope for Phase 2+ — stay strictly within your phase
11.	Merge to integration only after the phase completion gate passes
12.	Report back with: branch name, files changed, gate status, build output

9. Repo Hygiene
•	.env.local is gitignored — never commit secrets
•	.env.example committed with all six key names, values blank, with comments explaining AUTH vs PLATFORM split
•	node_modules, .next, .DS_Store in .gitignore
•	CLAUDE.md — update after each phase with completion status
•	README.md — update after Phase 7 with full deploy instructions
•	Commit format: <type>(<phase>): <description>

10. Quick Reference
Phase Summary
#	Phase	Duration	Parallel?	Gate Summary
0	Foundation	1-2 days	No — blocks all	next build clean, /products loads, middleware redirects
1	Auth Pages	1 day	No — needs P0	Register/login/verify works end to end
2	Dashboard	2 days	No — needs P1	Server card all states, deep link api_key correct
3	Open Source	2 days	Yes — with P4, P5	All 8 pages load, security@mahalaxmi.ai visible
4	Docs	1 day	Yes — with P3, P5	All 6 pages load, quickstart present
5	Content/Legal	1 day	Yes — with P3, P4	Support, about, terms, privacy load clean
6	Marketing + Products	1 day	No — needs P1	Live pricing, auth gate, Products nav link, /products live
7	Smoke Test + Deploy	1 day	No — needs all	All 13 pre-deploy checks pass, mahalaxmi.ai live

Critical path: P0 → P1 → P2 → P6 → P7 = 7 days. With parallel P3, P4, P5 total wall time ~8-9 days.

Key Locations
Item	Location
Migrate pages from	/home/alex/thrivetech/thrivetechWebsite/frontend/src/app/[locale]/
New repo root	/home/alex/thrivetech/mahalaxmi-website
GitHub repo	https://github.com/thrivetech2t/mahalaxmi-website (Private)
Production deploy path	/opt/deployments/mahalaxmi-website on 5.161.189.182
Nginx config	nginx/sites-available/thrivetech-product-platform.conf (in thrivetechWebsite)
Old shell to decommission	/opt/deployments/thrivetech-website — mahalaxmi-web container
Platform API base	https://tokenapi.thrivetechservice.com
Activation API base	https://tokenadmin.thrivetechservice.com
Open source repo	https://github.com/thrivetech2t/mahalaxmi (Public, MIT)
TERMINAL PAK key source	thrivetech/thrivetechWebsite/backend/.env — PRODUCT_PLATFORM_PAK_KEYS

