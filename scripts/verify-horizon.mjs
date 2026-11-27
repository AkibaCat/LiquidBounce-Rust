import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { crc32, inflateRawSync } from 'node:zlib';

const project = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const source = path.join(project, 'upstream', 'Horizon');
const archive = await readFile(path.join(project, 'research', 'Horizon-source.zip'));
const commit = '9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0';
const expectedArchiveSha256 = '4a9798eaf7b5dce1f711307efe638f4f9eed8eafc3d58cb0415ef63a38f1c95e';
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
if (sha256(archive) !== expectedArchiveSha256) throw new Error('Horizon archive SHA-256 mismatch.');

let end = archive.length - 22;
const minimum = Math.max(0, archive.length - 65557);
while (end >= minimum && archive.readUInt32LE(end) !== 0x06054b50) end--;
if (end < minimum) throw new Error('Missing ZIP end record.');
const archiveCommit = archive.subarray(end + 22, end + 22 + archive.readUInt16LE(end + 20)).toString();
if (archiveCommit !== commit) throw new Error('Horizon archive commit does not match the pinned Nightly.');
const centralOffset = archive.readUInt32LE(end + 16);
const files = [];
const mismatches = [];
let cursor = 0;
while (cursor < centralOffset) {
  if (archive.readUInt32LE(cursor) !== 0x04034b50) throw new Error('Invalid ZIP local header.');
  const flags = archive.readUInt16LE(cursor + 6);
  const method = archive.readUInt16LE(cursor + 8);
  const compressedSize = archive.readUInt32LE(cursor + 18);
  const nameLength = archive.readUInt16LE(cursor + 26);
  const extraLength = archive.readUInt16LE(cursor + 28);
  const name = archive.subarray(cursor + 30, cursor + 30 + nameLength).toString();
  const start = cursor + 30 + nameLength + extraLength;
  if ((flags & 8) || start + compressedSize > centralOffset) throw new Error('Unsupported ZIP descriptor.');
  if (!name.startsWith('mc-horizon-9f42064/')) throw new Error(`Unexpected archive root: ${name}`);
  if (!name.endsWith('/')) {
    const archived = method === 0 ? archive.subarray(start, start + compressedSize)
      : method === 8 ? inflateRawSync(archive.subarray(start, start + compressedSize))
        : (() => { throw new Error(`Unsupported ZIP compression: ${method}`); })();
    if (crc32(archived) !== archive.readUInt32LE(cursor + 14)) throw new Error(`CRC mismatch: ${name}`);
    const relativeName = name.slice('mc-horizon-9f42064/'.length);
    const file = path.resolve(source, ...relativeName.split('/'));
    const relative = path.relative(source, file);
    if (relative.startsWith('..') || path.isAbsolute(relative)) throw new Error(`Unsafe path: ${name}`);
    const expected = sha256(archived);
    files.push({ path: relativeName, sha256: expected });
    try {
      const actual = sha256(await readFile(file));
      if (actual !== expected) mismatches.push({ path: relativeName, expected, actual });
    } catch (error) {
      mismatches.push({ path: relativeName, error: error.code ?? error.message });
    }
  }
  cursor = start + compressedSize;
}
const report = {
  originalRepository: 'https://github.com/horizon-rust/horizon',
  recoveryRepository: 'https://github.com/Jijjy/mc-horizon',
  commit,
  localNightly: 'Nightly 2026-11-13 · 9f42064',
  archiveUrl: 'https://codeload.github.com/Jijjy/mc-horizon/zip/9f42064',
  archiveSha256: expectedArchiveSha256,
  acquisition: 'GitHub codeload source archive with full Git commit in ZIP comment; no Git history included',
  archiveCrcVerified: true,
  verifiedFiles: files.length,
  extractedSourceMatchesArchive: mismatches.length === 0,
  mismatches,
  files,
};
await writeFile(path.join(project, 'research', 'horizon-source-integrity.json'), `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify({ commit, archiveCrcVerified: true, verifiedFiles: files.length,
  extractedSourceMatchesArchive: report.extractedSourceMatchesArchive, mismatches }, null, 2));
if (mismatches.length) process.exitCode = 1;
