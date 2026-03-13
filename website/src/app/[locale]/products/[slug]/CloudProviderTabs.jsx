'use client';
import { useState } from 'react';
import { PROVIDER_LABELS } from '@/lib/cloudConstants';

const PROVIDERS = [
  { key: 'hetzner', ...PROVIDER_LABELS.hetzner, available: true },
  { key: 'aws',     ...PROVIDER_LABELS.aws,     available: false },
  { key: 'gcp',     ...PROVIDER_LABELS.gcp,     available: false },
];

export default function CloudProviderTabs({ onProviderChange }) {
  const [active, setActive] = useState('hetzner');

  const handleSelect = (key) => {
    if (!PROVIDERS.find(p => p.key === key).available) return;
    setActive(key);
    onProviderChange(key);
  };

  return (
    <div className="cloud-provider-tabs">
      {PROVIDERS.map(p => (
        <button
          key={p.key}
          onClick={() => handleSelect(p.key)}
          disabled={!p.available}
          data-active={active === p.key}
          style={{ '--provider-color': p.color }}
        >
          {p.name}
          {!p.available && (
            <span className="coming-soon-badge">Coming Soon</span>
          )}
        </button>
      ))}
    </div>
  );
}
