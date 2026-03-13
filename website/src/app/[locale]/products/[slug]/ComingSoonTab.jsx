'use client';
import { useState } from 'react';

export default function ComingSoonTab({ provider }) {
  const [email, setEmail] = useState('');
  const [submitted, setSubmitted] = useState(false);

  const handleSubmit = async () => {
    await fetch('/api/mahalaxmi/waitlist', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, provider }),
    });
    setSubmitted(true);
  };

  return (
    <div className="coming-soon-tab">
      <h3>{provider.toUpperCase()} support coming soon</h3>
      <p>Deploy Mahalaxmi on {provider.toUpperCase()} infrastructure.</p>
      {submitted ? (
        <p>You are on the waitlist. We will notify you at launch.</p>
      ) : (
        <div className="waitlist-form">
          <input
            type="email"
            placeholder="your@email.com"
            value={email}
            onChange={e => setEmail(e.target.value)}
          />
          <button onClick={handleSubmit}>Notify me</button>
        </div>
      )}
    </div>
  );
}
