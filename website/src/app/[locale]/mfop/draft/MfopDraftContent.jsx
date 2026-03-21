'use client';

import { useState } from 'react';
import {
  Box, Container, Typography, Paper, Chip, Divider, Button,
  Grid, Table, TableBody, TableCell, TableContainer, TableHead, TableRow,
  Link as MuiLink,
} from '@mui/material';
import { Comment, NavigateNext } from '@mui/icons-material';
import Link from 'next/link';
import TableOfContents from '@/components/Mfop/TableOfContents';
import GiscusComments from '@/components/Mfop/GiscusComments';
import FeedbackForm from '@/components/Mfop/FeedbackForm';

function SectionBlock({ block }) {
  if (block.type === 'table') {
    return (
      <TableContainer component={Paper} elevation={0} variant="outlined" sx={{ mt: 2, overflowX: 'auto' }}>
        <Table size="small">
          <TableHead>
            <TableRow sx={{ bgcolor: 'rgba(255,255,255,0.05)' }}>
              {block.headers.map((h) => (
                <TableCell key={h} sx={{ fontWeight: 700, whiteSpace: 'nowrap', color: 'primary.light' }}>{h}</TableCell>
              ))}
            </TableRow>
          </TableHead>
          <TableBody>
            {block.rows.map((row, i) => (
              <TableRow key={i} sx={{ '&:last-child td': { border: 0 } }}>
                {row.map((cell, j) => (
                  <TableCell key={j} sx={{ verticalAlign: 'top', lineHeight: 1.6 }}>{cell}</TableCell>
                ))}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </TableContainer>
    );
  }
  return null;
}

function SpecSection({ section, onComment }) {
  return (
    <Box
      id={section.id}
      component="section"
      sx={{ mb: 6, scrollMarginTop: '80px' }}
    >
      <Box sx={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', gap: 2, mb: 2 }}>
        <Typography
          variant={section.level === 1 ? 'h5' : 'h6'}
          component={section.level === 1 ? 'h2' : 'h3'}
          sx={{ fontWeight: 700, pb: 1, borderBottom: '1px solid', borderColor: 'divider', flex: 1 }}
        >
          {section.title}
        </Typography>
        <Button
          size="small"
          startIcon={<Comment sx={{ fontSize: 14 }} />}
          onClick={() => onComment(section.id)}
          sx={{
            fontSize: '0.7rem',
            color: 'text.disabled',
            whiteSpace: 'nowrap',
            mt: 0.5,
            '&:hover': { color: 'primary.main' },
          }}
        >
          Comment
        </Button>
      </Box>

      {section.content && (
        <Typography
          variant="body1"
          sx={{ whiteSpace: 'pre-line', lineHeight: 1.85, color: 'text.primary' }}
          dangerouslySetInnerHTML={{ __html: section.content.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>') }}
        />
      )}

      {section.blocks?.map((block, i) => (
        <SectionBlock key={i} block={block} />
      ))}
    </Box>
  );
}

export default function MfopDraftContent({ meta: mfopMeta, sections: mfopSections }) {
  const [feedbackSection, setFeedbackSection] = useState('');
  const [feedbackOpen, setFeedbackOpen] = useState(false);

  function handleComment(sectionId) {
    setFeedbackSection(sectionId);
    setFeedbackOpen(true);
    setTimeout(() => {
      document.getElementById('feedback-form')?.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }, 50);
  }

  return (
    <Container maxWidth="xl" sx={{ py: { xs: 3, md: 6 } }}>
      {/* Breadcrumb */}
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5, mb: 3, color: 'text.secondary', fontSize: '0.85rem' }}>
        <MuiLink component={Link} href="/" color="inherit" underline="hover">Mahalaxmi</MuiLink>
        <NavigateNext sx={{ fontSize: 16 }} />
        <Typography component="span" fontSize="inherit">MFOP Specification</Typography>
      </Box>

      <Grid container spacing={4}>
        {/* ToC sidebar */}
        <Grid item xs={12} lg={3}>
          <TableOfContents sections={mfopSections} />
        </Grid>

        {/* Main content */}
        <Grid item xs={12} lg={9}>
          <Paper elevation={0} variant="outlined" sx={{ p: { xs: 3, md: 5 }, borderColor: 'rgba(0,200,200,0.12)' }}>

            {/* Status banner */}
            <Box sx={{ bgcolor: 'rgba(200,160,64,0.1)', border: '1px solid', borderColor: 'warning.dark', borderRadius: 1, px: 2.5, py: 1.5, mb: 4, display: 'flex', alignItems: 'center', gap: 1.5, flexWrap: 'wrap' }}>
              <Chip label="Pre-Publication Draft" color="warning" size="small" variant="outlined" />
              <Typography variant="body2" color="warning.light">
                Comments solicited. Direct feedback to{' '}
                <MuiLink href={`mailto:${mfopMeta.email}`} color="warning.light">{mfopMeta.email}</MuiLink>
                {' '}or use the form below.
              </Typography>
            </Box>

            {/* Document header */}
            <Box sx={{ borderBottom: '3px solid', borderColor: 'primary.main', pb: 4, mb: 5 }}>
              <Typography variant="overline" color="primary" sx={{ fontWeight: 700, letterSpacing: 2 }}>
                {mfopMeta.shortTitle}
              </Typography>
              <Typography variant="h3" component="h1" sx={{ fontWeight: 800, mt: 0.5, mb: 1, lineHeight: 1.2 }}>
                {mfopMeta.title}
              </Typography>
              <Typography variant="h6" color="text.secondary" sx={{ mb: 3 }}>
                Specification Version {mfopMeta.version} — {mfopMeta.status}
              </Typography>

              <Box sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr', sm: 'auto auto' }, gap: 0.5, fontSize: '0.875rem', color: 'text.secondary' }}>
                <Typography variant="body2"><strong>Author:</strong> {mfopMeta.author}</Typography>
                <Typography variant="body2"><strong>Organization:</strong> {mfopMeta.org}</Typography>
                <Typography variant="body2"><strong>Location:</strong> {mfopMeta.location}</Typography>
                <Typography variant="body2"><strong>Date:</strong> {mfopMeta.date}</Typography>
                <Typography variant="body2">
                  <strong>Contact:</strong>{' '}
                  <MuiLink href={`mailto:${mfopMeta.email}`} color="primary">{mfopMeta.email}</MuiLink>
                </Typography>
              </Box>
            </Box>

            {/* Spec sections */}
            {mfopSections.map((section) => (
              <SpecSection key={section.id} section={section} onComment={handleComment} />
            ))}

            <Divider sx={{ my: 5 }} />

            {/* GitHub Discussions comments */}
            <Box id="comments" sx={{ mb: 5, scrollMarginTop: '80px' }}>
              <Typography variant="h5" component="h2" sx={{ fontWeight: 700, mb: 3 }}>
                Discussion
              </Typography>
              <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
                Comments powered by GitHub Discussions. A GitHub account is required.
                Prefer email? Use the feedback form below.
              </Typography>
              <GiscusComments />
            </Box>

            <Divider sx={{ my: 5 }} />

            {/* Structured feedback form */}
            <Box id="feedback-form" sx={{ scrollMarginTop: '80px' }}>
              <Typography variant="h5" component="h2" sx={{ fontWeight: 700, mb: 1 }}>
                Submit Feedback
              </Typography>
              <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
                No account required. Your feedback is forwarded directly to the author.
              </Typography>
              <FeedbackForm initialSection={feedbackOpen ? feedbackSection : ''} sections={mfopSections} key={feedbackSection} />
            </Box>

          </Paper>
        </Grid>
      </Grid>
    </Container>
  );
}
