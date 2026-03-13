export async function POST(request) {
  const { email, provider } = await request.json();
  // TODO: wire to Platform waitlist endpoint when available
  console.log(`Waitlist signup: ${email} for ${provider}`);
  return Response.json({ status: 'ok' });
}
