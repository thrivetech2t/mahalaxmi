'use client';

import { useEffect, useRef } from 'react';
import { Box, Typography, Alert } from '@mui/material';

// Giscus requires a public GitHub repo with Discussions enabled.
// Configure these values after setting up giscus at https://giscus.app/
const GISCUS_CONFIG = {
  repo: process.env.NEXT_PUBLIC_GISCUS_REPO || '',
  repoId: process.env.NEXT_PUBLIC_GISCUS_REPO_ID || '',
  category: process.env.NEXT_PUBLIC_GISCUS_CATEGORY || 'Peer Review',
  categoryId: process.env.NEXT_PUBLIC_GISCUS_CATEGORY_ID || '',
};

const configured = Boolean(GISCUS_CONFIG.repo && GISCUS_CONFIG.repoId && GISCUS_CONFIG.categoryId);

export default function GiscusComments() {
  const containerRef = useRef(null);

  useEffect(() => {
    if (!configured || !containerRef.current) return;

    const existing = containerRef.current.querySelector('script');
    if (existing) return;

    const script = document.createElement('script');
    script.src = 'https://giscus.app/client.js';
    script.setAttribute('data-repo', GISCUS_CONFIG.repo);
    script.setAttribute('data-repo-id', GISCUS_CONFIG.repoId);
    script.setAttribute('data-category', GISCUS_CONFIG.category);
    script.setAttribute('data-category-id', GISCUS_CONFIG.categoryId);
    script.setAttribute('data-mapping', 'pathname');
    script.setAttribute('data-strict', '0');
    script.setAttribute('data-reactions-enabled', '1');
    script.setAttribute('data-emit-metadata', '0');
    script.setAttribute('data-input-position', 'top');
    script.setAttribute('data-theme', 'dark');
    script.setAttribute('data-lang', 'en');
    script.setAttribute('data-loading', 'lazy');
    script.crossOrigin = 'anonymous';
    script.async = true;
    containerRef.current.appendChild(script);
  }, []);

  if (!configured) {
    return (
      <Alert
        severity="info"
        sx={{ bgcolor: 'rgba(0,200,200,0.08)', color: 'text.secondary', border: '1px solid', borderColor: 'primary.dark' }}
      >
        <Typography variant="body2">
          GitHub Discussions comments will appear here once giscus is configured.
          Set <code>NEXT_PUBLIC_GISCUS_REPO</code>, <code>NEXT_PUBLIC_GISCUS_REPO_ID</code>, and{' '}
          <code>NEXT_PUBLIC_GISCUS_CATEGORY_ID</code> environment variables.
          Visit <strong>giscus.app</strong> to get these values for your repository.
        </Typography>
      </Alert>
    );
  }

  return (
    <Box ref={containerRef} sx={{ '.giscus': { maxWidth: '100%' } }} />
  );
}
