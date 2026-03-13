import { NextResponse } from 'next/server';

export async function GET(request) {
  const platformUrl = process.env.MAHALAXMI_PLATFORM_API_URL;
  const pakKey = process.env.MAHALAXMI_TERMINAL_PAK_KEY;

  if (!platformUrl || !pakKey) {
    return NextResponse.json({ error: 'Releases not configured' }, { status: 503 });
  }

  const { searchParams } = new URL(request.url);
  const platform = searchParams.get('platform') || '';
  const architecture = searchParams.get('architecture') || '';

  const upstreamUrl = new URL(`${platformUrl}/api/v1/public/releases/latest`);
  if (platform) upstreamUrl.searchParams.set('platform', platform);
  if (architecture) upstreamUrl.searchParams.set('architecture', architecture);

  try {
    const res = await fetch(upstreamUrl.toString(), {
      headers: { 'X-Channel-API-Key': pakKey },
      next: { revalidate: 300 },
    });

    if (res.status === 404) {
      return NextResponse.json({ success: false, available: false }, { status: 200 });
    }

    if (!res.ok) {
      return NextResponse.json({ error: 'Releases unavailable' }, { status: 502 });
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch {
    return NextResponse.json({ error: 'Releases service unreachable' }, { status: 502 });
  }
}
