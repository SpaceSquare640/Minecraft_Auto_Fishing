// Asks GitHub for the latest release and compares it with this version.
const LATEST = "https://api.github.com/repos/SpaceSquare640/Minecraft_Auto_Fishing/releases/latest";

export type UpdateResult =
  | { kind: "latest" }
  | { kind: "newer"; version: string }
  | { kind: "none" }
  | { kind: "limited" }
  | { kind: "failed" };

function parse(version: string): number[] {
  return version.replace(/^v/i, "").split(/[.-]/).slice(0, 3).map((part) => Number.parseInt(part, 10) || 0);
}

export function isNewer(candidate: string, current: string): boolean {
  const [a, b] = [parse(candidate), parse(current)];
  for (let i = 0; i < 3; i += 1) {
    if ((a[i] ?? 0) !== (b[i] ?? 0)) return (a[i] ?? 0) > (b[i] ?? 0);
  }
  return false;
}

export async function checkForUpdate(current: string): Promise<UpdateResult> {
  try {
    const response = await fetch(LATEST, { headers: { Accept: "application/vnd.github+json" } });
    if (response.status === 404) return { kind: "none" };
    // GitHub allows 60 anonymous requests per hour per IP address.
    if (response.status === 403 || response.status === 429) return { kind: "limited" };
    if (!response.ok) return { kind: "failed" };
    const release = (await response.json()) as { tag_name?: unknown };
    if (typeof release.tag_name !== "string") return { kind: "failed" };
    return isNewer(release.tag_name, current) ? { kind: "newer", version: release.tag_name.replace(/^v/i, "") } : { kind: "latest" };
  } catch {
    return { kind: "failed" };
  }
}
