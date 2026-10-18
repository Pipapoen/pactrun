import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {readFile, writeFile, mkdir, copyFile} from 'node:fs/promises';
import {constants} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {commandHelp} from '../plugins/document-catalog/index.mjs';

const siteDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const root = path.dirname(siteDir);
const [version, ...extra] = process.argv.slice(2);
if (!version || (extra.length && (extra.length !== 2 || extra[0] !== '--from')))
  throw new Error('Usage: pnpm docs:version <Pactrun release version> [--from <saved edition>]');
const from = extra[1];
const help = await commandHelp(root);
if (version !== help.version) throw new Error('The documentation version must match Cargo.toml');
const versions = JSON.parse(await readFile(path.join(siteDir, 'versions.json'), 'utf8'));
if (versions.includes(version)) throw new Error('Documentation version already exists');
if (from && !versions.includes(from)) throw new Error('Base documentation version must be registered');
const sourceDocs = from ? `website/versioned_docs/version-${from}` : null;
const sourceSidebar = from ? `website/versioned_sidebars/version-${from}-sidebars.json` : null;
const inputs = ['docs', 'website/sidebars.ts', 'src/cli.rs', 'src/cli/help.rs', 'src/cli/help_catalog.json', 'Cargo.toml'];
if (from) inputs.push(sourceDocs, sourceSidebar);
const changed = execFileSync('git', ['status', '--porcelain', '--', ...inputs], {cwd: root, encoding: 'utf8'});
if (changed.trim()) throw new Error('Commit the reviewed documentation, sidebar and CLI inputs before capturing a release');
const source_commit = execFileSync('git', ['rev-parse', 'HEAD'], {cwd: root, encoding: 'utf8'}).trim();
if (from) {
  const files = execFileSync('git', ['ls-files', '-z', '--', sourceDocs], {cwd: root, encoding: 'utf8'}).split('\0').filter(Boolean);
  if (!files.length) throw new Error('Base edition has no committed documents');
  const destination = path.join(siteDir, 'versioned_docs', 'version-' + version);
  await mkdir(destination);
  for (const file of files) {
    const target = path.join(destination, path.posix.relative(sourceDocs, file));
    await mkdir(path.dirname(target), {recursive: true});
    await copyFile(path.join(root, file), target, constants.COPYFILE_EXCL);
  }
  await copyFile(path.join(root, sourceSidebar), path.join(siteDir, 'versioned_sidebars', `version-${version}-sidebars.json`), constants.COPYFILE_EXCL);
  await writeFile(path.join(siteDir, 'versions.json'), JSON.stringify([version, ...versions], null, 2) + '\n');
} else {
  const require = createRequire(import.meta.url);
  const cli = path.join(path.dirname(require.resolve('@docusaurus/core/package.json')), 'bin/docusaurus.mjs');
  execFileSync(process.execPath, [cli, 'docs:version', version], {cwd: siteDir, stdio: 'inherit'});
}
await mkdir(path.join(siteDir, 'versioned_help'), {recursive: true});
await writeFile(path.join(siteDir, 'versioned_help', version + '.json'),
  JSON.stringify({...help, source_commit, release_tag: 'v' + version}, null, 2) + '\n');
