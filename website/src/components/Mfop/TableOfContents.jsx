'use client';

import { useState, useEffect } from 'react';
import {
  Box, Typography, Link, Accordion, AccordionSummary, AccordionDetails,
} from '@mui/material';
import { ExpandMore, MenuBook } from '@mui/icons-material';

export default function TableOfContents({ sections }) {
  const [activeId, setActiveId] = useState('');

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries.filter((e) => e.isIntersecting);
        if (visible.length > 0) {
          setActiveId(visible[0].target.id);
        }
      },
      { rootMargin: '-80px 0px -60% 0px', threshold: 0 }
    );

    sections.forEach(({ id }) => {
      const el = document.getElementById(id);
      if (el) observer.observe(el);
    });

    return () => observer.disconnect();
  }, [sections]);

  const TocList = () => (
    <Box component="nav" aria-label="Table of contents">
      {sections.map(({ id, title }) => {
        const isActive = activeId === id;
        const isAppendix = id.startsWith('appendix') || id === 'acknowledgements' || id === 'copyright' || id === 'status-memo';
        return (
          <Link
            key={id}
            href={`#${id}`}
            underline="none"
            sx={{
              display: 'block',
              py: 0.5,
              px: 1.5,
              borderLeft: '2px solid',
              borderColor: isActive ? 'primary.main' : 'transparent',
              color: isActive ? 'primary.main' : isAppendix ? 'text.disabled' : 'text.secondary',
              fontSize: '0.8rem',
              lineHeight: 1.4,
              transition: 'all 0.15s',
              '&:hover': { color: 'primary.light', borderColor: 'primary.light' },
            }}
          >
            {title}
          </Link>
        );
      })}
    </Box>
  );

  return (
    <>
      {/* Desktop sticky sidebar */}
      <Box
        sx={{
          display: { xs: 'none', lg: 'block' },
          position: 'sticky',
          top: 80,
          maxHeight: 'calc(100vh - 100px)',
          overflowY: 'auto',
          pr: 1,
          '&::-webkit-scrollbar': { width: 4 },
          '&::-webkit-scrollbar-thumb': { bgcolor: 'primary.dark', borderRadius: 2 },
        }}
      >
        <Typography
          variant="overline"
          sx={{ display: 'block', px: 1.5, mb: 1, color: 'text.disabled', fontWeight: 700, fontSize: '0.65rem' }}
        >
          Contents
        </Typography>
        <TocList />
      </Box>

      {/* Mobile accordion */}
      <Accordion
        disableGutters
        elevation={0}
        sx={{
          display: { xs: 'block', lg: 'none' },
          mb: 3,
          bgcolor: 'background.paper',
          border: '1px solid',
          borderColor: 'divider',
          borderRadius: '8px !important',
          '&:before': { display: 'none' },
        }}
      >
        <AccordionSummary expandIcon={<ExpandMore />} sx={{ minHeight: 48 }}>
          <MenuBook sx={{ mr: 1, fontSize: 18, color: 'primary.main' }} />
          <Typography variant="body2" sx={{ fontWeight: 600 }}>Table of Contents</Typography>
        </AccordionSummary>
        <AccordionDetails sx={{ pt: 0 }}>
          <TocList />
        </AccordionDetails>
      </Accordion>
    </>
  );
}
