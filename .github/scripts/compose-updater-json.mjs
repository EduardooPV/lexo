const PLATFORMS = [
  { keys: ['darwin-aarch64', 'darwin-x86_64', 'darwin-aarch64-app', 'darwin-x86_64-app'], match: /\.app\.tar\.gz$/ },
  { keys: ['windows-x86_64', 'windows-x86_64-msi'], match: /\.msi$/ },
  { keys: ['windows-x86_64-nsis'], match: /-setup\.exe$/ },
  { keys: ['linux-x86_64', 'linux-x86_64-appimage'], match: /\.AppImage$/ },
  { keys: ['linux-x86_64-deb'], match: /\.deb$/ },
  { keys: ['linux-x86_64-rpm'], match: /\.rpm$/ }
];

const { GITHUB_TOKEN, GITHUB_REPOSITORY, RELEASE_TAG, RELEASE_VERSION } = process.env;
const dryRun = process.argv.includes('--dry-run');

if (!GITHUB_TOKEN || !GITHUB_REPOSITORY || !RELEASE_TAG) {
  throw new Error('GITHUB_TOKEN, GITHUB_REPOSITORY and RELEASE_TAG are required');
}

const api = async (path, init = {}) => {
  const response = await fetch(`https://api.github.com${path}`, {
    ...init,
    headers: {
      Accept: 'application/vnd.github+json',
      Authorization: `Bearer ${GITHUB_TOKEN}`,
      'X-GitHub-Api-Version': '2022-11-28',
      ...init.headers
    }
  });
  if (!response.ok) {
    throw new Error(`${init.method || 'GET'} ${path} → ${response.status} ${await response.text()}`);
  }
  return response.status === 204 ? null : response.json();
};

const release = await api(`/repos/${GITHUB_REPOSITORY}/releases/tags/${RELEASE_TAG}`);

const download = async (asset) => {
  const response = await fetch(asset.url, {
    headers: {
      Accept: 'application/octet-stream',
      Authorization: `Bearer ${GITHUB_TOKEN}`
    }
  });
  if (!response.ok) throw new Error(`could not read ${asset.name} → ${response.status}`);
  return (await response.text()).trim();
};

const binaries = release.assets.filter((asset) => !asset.name.endsWith('.sig'));
const platforms = {};
const missing = [];

for (const { keys, match } of PLATFORMS) {
  const binary = binaries.find((asset) => match.test(asset.name));
  const signature = binary && release.assets.find((asset) => asset.name === `${binary.name}.sig`);

  if (!binary || !signature) {
    missing.push(keys[0]);
    continue;
  }

  const value = { signature: await download(signature), url: binary.browser_download_url };
  for (const key of keys) platforms[key] = value;
}

if (missing.length) {
  throw new Error(`no signed bundle on ${RELEASE_TAG} for: ${missing.join(', ')}`);
}

const manifest = {
  version: RELEASE_VERSION || RELEASE_TAG.replace(/^v/, ''),
  notes: release.body || '',
  pub_date: new Date().toISOString(),
  platforms
};

console.log(`composed latest.json for ${RELEASE_TAG} with ${Object.keys(platforms).length} platform keys`);

if (dryRun) {
  console.log(JSON.stringify(manifest, null, 2));
  process.exit(0);
}

const existing = release.assets.find((asset) => asset.name === 'latest.json');
if (existing) {
  await api(`/repos/${GITHUB_REPOSITORY}/releases/assets/${existing.id}`, { method: 'DELETE' });
}

const upload = await fetch(
  `https://uploads.github.com/repos/${GITHUB_REPOSITORY}/releases/${release.id}/assets?name=latest.json`,
  {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${GITHUB_TOKEN}`,
      'Content-Type': 'application/json',
      'X-GitHub-Api-Version': '2022-11-28'
    },
    body: JSON.stringify(manifest, null, 2)
  }
);

if (!upload.ok) {
  throw new Error(`upload failed → ${upload.status} ${await upload.text()}`);
}

console.log('uploaded latest.json');
