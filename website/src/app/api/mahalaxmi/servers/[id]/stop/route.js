import { getUserToken, pakAndJwtHeaders, unauthorizedResponse } from '@/lib/proxyHelpers';

export async function POST(request, { params }) {
  const token = getUserToken(request);
  if (!token) return unauthorizedResponse();
  const { id } = await params;

  const platformRes = await fetch(
    `${process.env.MAHALAXMI_PLATFORM_API_URL}/api/v1/mahalaxmi/servers/${id}/stop`,
    {
      method: 'POST',
      headers: pakAndJwtHeaders(process.env.MAHALAXMI_CLOUD_PAK_KEY, token),
    }
  );

  if (!platformRes.ok) {
    const error = await platformRes.text();
    console.error(`[stop] platform error ${platformRes.status}`, error);
    return Response.json(
      { error: 'stop_failed', detail: error },
      { status: platformRes.status }
    );
  }

  return Response.json(await platformRes.json());
}
