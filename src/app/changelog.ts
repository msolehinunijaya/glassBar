// CHANGELOG.md -> versions with grouped bullet points, for Settings › About.
// Only what the changelog uses: "## version — date", "### group", "- item" with **bold**.
export type Release = { version: string; date: string; groups: { title: string; items: string[] }[] };

export function parseChangelog(md: string): Release[] {
  const out: Release[] = [];
  for (const raw of md.split(/\r?\n/)) {
    const line = raw.trim();
    const rel = /^## (\S+)(?:\s+[—-]\s+(.+))?$/.exec(line);
    if (rel) { out.push({ version: rel[1], date: rel[2] ?? '', groups: [] }); continue; }
    const r = out.at(-1);
    if (!r) continue;
    if (line.startsWith('### ')) r.groups.push({ title: line.slice(4), items: [] });
    else if (line.startsWith('- ')) (r.groups.at(-1) ?? (r.groups[0] = { title: '', items: [] })).items.push(line.slice(2));
  }
  return out;
}

/** "a **b** c" -> [["a ", false], ["b", true], [" c", false]] */
export const boldParts = (s: string): [string, boolean][] =>
  s.split(/\*\*(.+?)\*\*/).map((t, i) => [t, i % 2 === 1] as [string, boolean]).filter(([t]) => t);
