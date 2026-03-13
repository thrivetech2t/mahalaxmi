import { setRequestLocale } from 'next-intl/server';
import { locales } from '@/i18n/routing';
import {
  Box, Button, Container, Divider, Typography,
} from '@mui/material';
import { Email, Business } from '@mui/icons-material';
import Link from 'next/link';

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export const metadata = {
  title: 'Contact Sales — Mahalaxmi',
  description: 'Get in touch with the Mahalaxmi team for enterprise pricing, custom deployments, or general support.',
};

export default async function ContactPage({ params }) {
  const { locale } = await params;
  setRequestLocale(locale);

  return (
    <Container maxWidth="sm" sx={{ py: { xs: 6, md: 10 } }}>
      <Typography variant="h3" component="h1" sx={{ fontWeight: 800, mb: 1 }}>
        Contact us
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 5 }}>
        Whether you need enterprise pricing, a custom deployment, or just have a question — we respond within one business day.
      </Typography>

      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
        {/* Sales */}
        <Box sx={{ p: 3, border: '1px solid', borderColor: 'divider', borderRadius: 2 }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
            <Business sx={{ color: 'primary.main' }} />
            <Typography variant="h6" sx={{ fontWeight: 700 }}>Enterprise &amp; sales</Typography>
          </Box>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
            Team licensing, volume pricing, HIPAA/FedRAMP profiles, and custom SLAs.
          </Typography>
          <Button
            component="a"
            href="mailto:sales@mahalaxmi.ai?subject=Enterprise%20Inquiry"
            variant="contained"
            startIcon={<Email />}
          >
            Email sales
          </Button>
        </Box>

        <Divider />

        {/* Support */}
        <Box sx={{ p: 3, border: '1px solid', borderColor: 'divider', borderRadius: 2 }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
            <Email sx={{ color: 'primary.main' }} />
            <Typography variant="h6" sx={{ fontWeight: 700 }}>Support</Typography>
          </Box>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
            Bug reports, billing questions, account issues, and general help.
          </Typography>
          <Button
            component="a"
            href="mailto:support@mahalaxmi.ai"
            variant="outlined"
            startIcon={<Email />}
          >
            Email support
          </Button>
        </Box>
      </Box>

      <Typography variant="body2" color="text.secondary" sx={{ mt: 5, textAlign: 'center' }}>
        Looking for documentation?{' '}
        <Link href="/docs/quickstart" style={{ color: 'inherit' }}>Browse the docs →</Link>
      </Typography>
    </Container>
  );
}
