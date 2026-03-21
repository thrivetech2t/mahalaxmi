import { setRequestLocale } from 'next-intl/server';
import { locales } from '@/i18n/routing';
import { getMfopSpec } from '@/lib/mfop/index';
import MfopDraftContent from './MfopDraftContent';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata({ params }) {
  const { locale } = await params;
  const { mfopMeta } = await getMfopSpec(locale);
  return {
    title: `${mfopMeta.shortTitle} Specification v${mfopMeta.version} — ${mfopMeta.status} | Mahalaxmi`,
    description: `Mahalaxmi Federation and Orchestration Protocol (MFOP) v${mfopMeta.version} — pre-publication draft circulated for peer review. Comments solicited.`,
    openGraph: {
      title: `${mfopMeta.shortTitle} Specification v${mfopMeta.version} — ${mfopMeta.status}`,
      description: `A protocol for federated distributed AI orchestration across heterogeneous compute nodes with compliance-zone-aware routing, cryptographically signed billing receipts, and configurable economic settlement.`,
      type: 'article',
    },
  };
}

export default async function MfopDraftPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);
  const { mfopMeta, mfopSections } = await getMfopSpec(locale);
  return <MfopDraftContent meta={mfopMeta} sections={mfopSections} />;
}
