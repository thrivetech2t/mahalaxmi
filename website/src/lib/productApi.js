// Central product data fetcher — all product details come from Platform API via PAK key.
// Used by server components only (async). Never imported by client components.

const PLATFORM_API_URL = process.env.MAHALAXMI_PLATFORM_API_URL;
const CLOUD_PAK_KEY   = process.env.MAHALAXMI_CLOUD_PAK_KEY;
const DESKTOP_PAK_KEY = process.env.MAHALAXMI_DESKTOP_PAK_KEY;

async function fetchOffering(pakKey) {
  const res = await fetch(
    `${PLATFORM_API_URL}/api/v1/products/offering`,
    {
      headers: { 'X-Channel-API-Key': pakKey },
      next: { revalidate: 300 }, // Next.js ISR — 5 min
    }
  );
  if (!res.ok) throw new Error(`Offering fetch failed: ${res.status}`);
  return res.json();
}

export async function getDesktopProductOffering() {
  return fetchOffering(DESKTOP_PAK_KEY);
}

export async function getCloudProductOffering() {
  return fetchOffering(CLOUD_PAK_KEY);
}

// Desktop-scoped helpers
export async function getPricingTiers() {
  const offering = await getDesktopProductOffering();
  return offering.pricing_tiers ?? [];
}

export async function getDownloads() {
  const offering = await getDesktopProductOffering();
  return offering.downloads ?? {};
}

// Cloud-scoped helpers
export async function getProviderCatalog() {
  const offering = await getCloudProductOffering();
  return offering.provider_catalog ?? [];
}

export async function getProductMeta() {
  const offering = await getDesktopProductOffering();
  return offering.product ?? {};
}
