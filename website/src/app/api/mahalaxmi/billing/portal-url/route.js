import { getUserToken, pakAndJwtHeaders, unauthorizedResponse } from '@/lib/proxyHelpers';

export async function POST(request) {
  const token = getUserToken(request);
  if (!token) return unauthorizedResponse();

  const platformRes = await fetch(
    `${process.env.MAHALAXMI_PLATFORM_API_URL}/api/v1/mahalaxmi/billing/portal`,
    {
      method: 'POST',
      headers: pakAndJwtHeaders(process.env.MAHALAXMI_CLOUD_PAK_KEY, token),
    }
  );

  if (!platformRes.ok) {
    const error = await platformRes.text();
    console.error(`[billing-portal] platform error ${platformRes.status}`, error);
    return Response.json(
      { error: 'billing_portal_failed', detail: error },
      { status: platformRes.status }
    );
  }

  const { url } = await platformRes.json();
  return Response.json({ url });
}
