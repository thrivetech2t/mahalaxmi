'use client';

import { useEffect, useState } from 'react';
import { useSearchParams } from 'next/navigation';
import PricingTiersDisplay from '@/components/Docs/PricingTiersDisplay';
import BuyNowButton from './BuyNowButton';
import StudentPricingButton from './StudentPricingButton';

export default function CloudPricingDisplay({ pricingData }) {
  const searchParams = useSearchParams();
  const billingCycleParam = searchParams.get('billing_cycle');
  const initialInterval = billingCycleParam === 'monthly' ? 'monthly'
    : billingCycleParam === 'annual' ? 'yearly'
    : undefined;

  const [isVerified, setIsVerified] = useState(false);

  useEffect(() => {
    const tiers = pricingData?.pricingTiers ?? [];
    const hasVerificationTier = tiers.some((t) => t.requires_verification);
    if (!hasVerificationTier) return;
    fetch('/api/mahalaxmi/verification/status')
      .then((r) => r.json())
      .then((d) => { if (d.verified) setIsVerified(true); })
      .catch(() => {});
  }, [pricingData]);

  return (
    <PricingTiersDisplay
      pricingData={pricingData}
      initialInterval={initialInterval}
      renderAction={(tier, billingInterval) => {
        const billingCycle = billingInterval === 'yearly' ? 'annual' : 'monthly';
        const variant = tier.isRecommended ? 'contained' : 'outlined';

        if (tier.requires_verification) {
          if (isVerified) {
            return (
              <BuyNowButton
                tier={tier.slug}
                billingCycle={billingCycle}
                label={`Start ${tier.name}`}
                variant={variant}
              />
            );
          }
          return (
            <StudentPricingButton
              tierId={tier.id}
              variant={variant}
              onVerified={() => setIsVerified(true)}
            />
          );
        }

        return (
          <BuyNowButton
            tier={tier.slug}
            billingCycle={billingCycle}
            label={`Start ${tier.name}`}
            variant={variant}
          />
        );
      }}
    />
  );
}
