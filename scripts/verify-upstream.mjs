import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const project = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const source = path.join(project, 'upstream', 'LiquidBounce');
const research = path.join(project, 'research');
const pinnedCommit = 'ae0eb396c07f22383789b28d5912d714c6ca0a86';
const pinnedArchiveSha256 = '98aa5c9007b80b78bae8edf85ca80d87fa65bebea41cc662ab8c2221016eb0cb';
const commit = JSON.parse(await readFile(path.join(research, 'upstream-commit.json'), 'utf8'));
const tree = JSON.parse(await readFile(path.join(research, 'upstream-tree.json'), 'utf8'));

if (commit.sha !== pinnedCommit || tree.sha !== commit.commit.tree.sha || tree.truncated) {
  throw new Error('The saved GitHub commit/tree metadata does not match the pinned source.');
}

const archive = await readFile(path.join(research, 'LiquidBounce-source.zip'));
const archiveSha256 = createHash('sha256').update(archive).digest('hex');
if (archiveSha256 !== pinnedArchiveSha256) {
  throw new Error('The source archive SHA-256 does not match the downloaded baseline.');
}

const blobs = tree.tree.filter(entry => entry.type === 'blob');
const mismatches = [];
const lineEndingOnlyDifferences = [];
for (const entry of blobs) {
  const file = path.resolve(source, ...entry.path.split('/'));
  const relative = path.relative(source, file);
  if (relative.startsWith('..') || path.isAbsolute(relative)) {
    throw new Error(`Unsafe path in source manifest: ${entry.path}`);
  }
  try {
    const contents = await readFile(file);
    const actualSha = createHash('sha1')
      .update(`blob ${contents.length}\0`)
      .update(contents)
      .digest('hex');
    if (actualSha !== entry.sha) {
      // GitHub's archive honors *.bat eol=crlf in the upstream .gitattributes.
      // Git blobs store the corresponding canonical LF text.
      const canonical = entry.path.endsWith('.bat')
        ? Buffer.from(contents.toString('utf8').replace(/\r\n/g, '\n'))
        : contents;
      const canonicalSha = createHash('sha1')
        .update(`blob ${canonical.length}\0`)
        .update(canonical)
        .digest('hex');
      if (canonicalSha === entry.sha) {
        lineEndingOnlyDifferences.push(entry.path);
      } else {
        mismatches.push({ path: entry.path, expected: entry.sha, actual: actualSha });
      }
    }
  } catch (error) {
    mismatches.push({ path: entry.path, error: error.code ?? error.message });
  }
}

const gitlinks = tree.tree.filter(entry => entry.type === 'commit');
const report = {
  repository: 'https://github.com/CCBlueX/LiquidBounce',
  branch: 'main',
  commit: pinnedCommit,
  upstreamCommitTimeUtc: commit.commit.committer.date,
  acquisition: 'GitHub official codeload source archive; no Git history included',
  archiveUrl: `https://codeload.github.com/CCBlueX/LiquidBounce/zip/${pinnedCommit}`,
  archiveSha256,
  treeSha: tree.sha,
  verifiedFiles: blobs.length,
  submodules: gitlinks.map(entry => ({ path: entry.path, commit: entry.sha })),
  trackedSourceMatchesUpstream: mismatches.length === 0 && gitlinks.length === 0,
  lineEndingOnlyDifferences,
  mismatches,
};
await writeFile(path.join(research, 'source-integrity.json'), `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
if (!report.trackedSourceMatchesUpstream) process.exitCode = 1;
