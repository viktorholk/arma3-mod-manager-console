interface ReleaseAsset {
  name: string;
  browser_download_url: string;
}

interface ReleaseData {
  tag_name: string;
  assets: ReleaseAsset[];
}

let cached: ReleaseData | null = null;

export async function getLatestRelease(): Promise<ReleaseData> {
  if (cached) return cached;

  const res = await fetch(
    'https://api.github.com/repos/viktorholk/arma3-mod-manager-console/releases/latest'
  );

  if (!res.ok) {
    throw new Error(`GitHub API returned ${res.status}`);
  }

  cached = await res.json() as ReleaseData;
  return cached!;
}

export function findAssetUrl(assets: ReleaseAsset[], pattern: string): string {
  const asset = assets.find((a) => a.name.includes(pattern));
  return asset?.browser_download_url ?? '#';
}
