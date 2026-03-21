'use client';

import { useState } from 'react';
import {
  Box, Typography, TextField, Select, MenuItem, FormControl,
  InputLabel, Button, Alert, Divider, Chip,
} from '@mui/material';
import { Send, CheckCircle } from '@mui/icons-material';
const COMMENT_TYPES = [
  { value: 'technical', label: 'Technical Issue' },
  { value: 'editorial', label: 'Editorial / Typo' },
  { value: 'general', label: 'General Feedback' },
  { value: 'question', label: 'Question' },
];

const INITIAL = { name: '', email: '', section: '', type: 'general', comment: '' };

export default function FeedbackForm({ initialSection = '', sections = [] }) {
  const [form, setForm] = useState({ ...INITIAL, section: initialSection });
  const [errors, setErrors] = useState({});
  const [status, setStatus] = useState('idle'); // idle | submitting | success | error

  function validate() {
    const e = {};
    if (!form.name.trim()) e.name = 'Name is required';
    if (!form.email.trim()) e.email = 'Email is required';
    else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(form.email)) e.email = 'Invalid email address';
    if (!form.comment.trim()) e.comment = 'Comment is required';
    if (form.comment.trim().length < 10) e.comment = 'Please provide a more detailed comment';
    return e;
  }

  function handleChange(field) {
    return (e) => {
      setForm((prev) => ({ ...prev, [field]: e.target.value }));
      if (errors[field]) setErrors((prev) => { const next = { ...prev }; delete next[field]; return next; });
    };
  }

  async function handleSubmit(e) {
    e.preventDefault();
    const validation = validate();
    if (Object.keys(validation).length) { setErrors(validation); return; }

    setStatus('submitting');
    try {
      const res = await fetch('/api/mfop/feedback', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(form),
      });
      if (!res.ok) throw new Error('Submission failed');
      setStatus('success');
      setForm(INITIAL);
    } catch {
      setStatus('error');
    }
  }

  if (status === 'success') {
    return (
      <Box sx={{ textAlign: 'center', py: 4 }}>
        <CheckCircle sx={{ fontSize: 48, color: 'success.main', mb: 2 }} />
        <Typography variant="h6" sx={{ mb: 1 }}>Thank you for your feedback!</Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
          Your comments have been sent to the author and will be considered for the next revision.
        </Typography>
        <Button variant="outlined" size="small" onClick={() => setStatus('idle')}>
          Submit another comment
        </Button>
      </Box>
    );
  }

  return (
    <Box component="form" onSubmit={handleSubmit} noValidate>
      <Box sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr', sm: '1fr 1fr' }, gap: 2, mb: 2 }}>
        <TextField
          label="Your Name"
          value={form.name}
          onChange={handleChange('name')}
          error={Boolean(errors.name)}
          helperText={errors.name}
          required
          size="small"
          fullWidth
        />
        <TextField
          label="Email Address"
          type="email"
          value={form.email}
          onChange={handleChange('email')}
          error={Boolean(errors.email)}
          helperText={errors.email || 'Not published — used for follow-up only'}
          required
          size="small"
          fullWidth
        />
      </Box>

      <Box sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr', sm: '1fr 1fr' }, gap: 2, mb: 2 }}>
        <FormControl size="small" fullWidth>
          <InputLabel>Section (optional)</InputLabel>
          <Select
            value={form.section}
            onChange={handleChange('section')}
            label="Section (optional)"
          >
            <MenuItem value=""><em>General — no specific section</em></MenuItem>
            {sections.map(({ id, title }) => (
              <MenuItem key={id} value={id}>{title}</MenuItem>
            ))}
          </Select>
        </FormControl>

        <FormControl size="small" fullWidth>
          <InputLabel>Comment Type</InputLabel>
          <Select
            value={form.type}
            onChange={handleChange('type')}
            label="Comment Type"
          >
            {COMMENT_TYPES.map(({ value, label }) => (
              <MenuItem key={value} value={value}>{label}</MenuItem>
            ))}
          </Select>
        </FormControl>
      </Box>

      <TextField
        label="Your Comment"
        value={form.comment}
        onChange={handleChange('comment')}
        error={Boolean(errors.comment)}
        helperText={errors.comment}
        required
        multiline
        rows={5}
        fullWidth
        sx={{ mb: 2 }}
        placeholder="Describe your comment, question, or suggested change in detail…"
      />

      {status === 'error' && (
        <Alert severity="error" sx={{ mb: 2 }}>
          Submission failed. Please try again or email directly to{' '}
          <strong>Ami.nunez@mahalaxmi.ai</strong>.
        </Alert>
      )}

      <Button
        type="submit"
        variant="contained"
        endIcon={<Send />}
        disabled={status === 'submitting'}
        sx={{ minWidth: 160 }}
      >
        {status === 'submitting' ? 'Sending…' : 'Submit Feedback'}
      </Button>

      <Typography variant="caption" color="text.disabled" sx={{ display: 'block', mt: 1.5 }}>
        Comments are forwarded to the author. Your email will not be published or shared.
      </Typography>
    </Box>
  );
}
