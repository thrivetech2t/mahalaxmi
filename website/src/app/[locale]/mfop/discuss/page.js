import { redirect } from 'next/navigation';
import { locales } from '@/i18n/routing';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export const metadata = {
  title: 'MFOP Discussion | Mahalaxmi',
  description: 'Discussion threads for the MFOP specification peer review.',
};

export default function MfopDiscussPage() {
  redirect('/mfop/draft#comments');
}
