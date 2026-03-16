import { getCloudProductOffering } from '@/lib/productApi';

/**
 * Transforms a Platform API tier (snake_case) into the shape PricingTiersDisplay expects.
 */
function transformTier(t) {
  return {
    id:          t.id,
    name:        t.name,
    slug:        t.slug,
    description: t.description ?? null,
    pricing: {
      monthly:  t.price_monthly  ?? null,
      yearly:   t.price_annual   ?? null,
      lifetime: t.price_lifetime ?? null,
      currency: 'USD',
    },
    isRecommended:        t.is_recommended   ?? false,
    trial: {
      enabled:      t.trial_enabled      ?? false,
      durationDays: t.trial_duration_days ?? null,
    },
    features:             t.features            ?? [],
    requires_verification: t.requires_verification ?? false,
  };
}

/**
 * Fetches cloud pricing from the Platform API via /api/v1/products/offering.
 * Returns { pricingTiers: [...] } in the format PricingTiersDisplay expects,
 * or null if the platform is unreachable.
 * The PAK key is never sent to the browser — this runs server-side only.
 */
export async function fetchCloudPricing() {
  try {
    const offering = await getCloudProductOffering();
    const rawTiers = (offering.pricing_tiers ?? []).filter((t) => !t.is_addon);
    if (!rawTiers.length) return null;
    return { pricingTiers: rawTiers.map(transformTier) };
  } catch {
    return null;
  }
}
