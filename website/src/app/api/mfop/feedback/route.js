import { NextResponse } from 'next/server';
import { mfopSections, mfopMeta } from '@/lib/mfopSpec';

const RECIPIENT = 'Ami.nunez@mahalaxmi.ai';

function buildEmailBody({ name, email, section, type, comment }) {
  const sectionLabel = section
    ? (mfopSections.find((s) => s.id === section)?.title ?? section)
    : 'General (no specific section)';

  return `
MFOP Specification Peer Review Feedback
========================================

From:    ${name} <${email}>
Section: ${sectionLabel}
Type:    ${type}
Date:    ${new Date().toISOString()}

Comment:
--------
${comment}

--
Submitted via https://mahalaxmi.ai/mfop/draft
`.trim();
}

// To enable email delivery, install nodemailer (`npm install nodemailer`)
// and uncomment the function below, then update the POST handler to call it.
//
// async function sendViaNodemailer(payload) {
//   const nodemailer = require('nodemailer');
//   const transport = nodemailer.createTransport({
//     host: process.env.SMTP_HOST,
//     port: parseInt(process.env.SMTP_PORT ?? '587', 10),
//     secure: process.env.SMTP_SECURE === 'true',
//     auth: { user: process.env.SMTP_USER, pass: process.env.SMTP_PASS },
//   });
//   await transport.sendMail({
//     from: `"MFOP Review" <${process.env.SMTP_FROM ?? process.env.SMTP_USER}>`,
//     to: RECIPIENT,
//     replyTo: `${payload.name} <${payload.email}>`,
//     subject: `[MFOP Peer Review] ${payload.type} — ${payload.section || 'General'}`,
//     text: buildEmailBody(payload),
//   });
// }

export async function POST(request) {
  let body;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: 'Invalid JSON' }, { status: 400 });
  }

  const { name, email, section, type, comment } = body ?? {};

  if (!name?.trim() || !email?.trim() || !comment?.trim()) {
    return NextResponse.json({ error: 'name, email, and comment are required' }, { status: 422 });
  }
  if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
    return NextResponse.json({ error: 'Invalid email address' }, { status: 422 });
  }

  const payload = {
    name: String(name).slice(0, 200),
    email: String(email).slice(0, 200),
    section: String(section ?? '').slice(0, 100),
    type: String(type ?? 'general').slice(0, 50),
    comment: String(comment).slice(0, 10000),
  };

  // Log feedback to server output.
  // Replace this block with a call to sendViaNodemailer() (see above) once
  // SMTP credentials are configured and nodemailer is installed.
  console.log('[mfop/feedback] Feedback received:\n', buildEmailBody(payload));

  return NextResponse.json({ success: true });
}
