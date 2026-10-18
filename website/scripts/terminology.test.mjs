import assert from 'node:assert/strict';
import test from 'node:test';
import {readFile, readdir} from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {readDocuments, markdownBody} from '../plugins/text-docs/index.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const saved = JSON.parse(await readFile(path.join(root, 'website/versions.json'), 'utf8'));
const scopes = ['docs', ...saved.filter(v => ['1.0.0-alpha.10', '1.0.0-alpha.11'].includes(v))
  .map(v => 'website/versioned_docs/version-' + v)];
const images = new Set();

function formal(text, owner) {
  const remaining = text.replace(/\.pack\b/g, '').replace(/\bRevision\s+Packs?\b/g, '');
  const match = /\b[Pp]acks?\b/.exec(remaining);
  assert.ok(!match, owner + (match ? ': ' + remaining.slice(Math.max(0, match.index - 35), match.index + 75) : ''));
}

function prose(source) {
  const result = [];
  let fence = null;
  for (const line of markdownBody(source).replace(/<!--[\s\S]*?-->/g, '').split('\n')) {
    if (fence) {
      if (new RegExp('^ {0,3}' + fence[0] + '{' + fence.length + ',}\\s*$').test(line)) fence = null;
      continue;
    }
    const open = line.match(/^ {0,3}(`{3,}|~{3,})/);
    if (open) {fence = open[1]; continue;}
    result.push(line);
  }
  return (result.join('\n') + '\n' + (source.match(/^title:\s*(.+)$/m)?.[1] ?? ''))
    .replace(/`[^`]*`/g, '').replace(/\]\([^)]*\)/g, ']')
    .replace(/<[^>]*>/g, '').replace(/\{#[^}]+\}/g, '').replace(/\*/g, '');
}

test('current formal documentation and referenced diagram text use Revision Pack', async () => {
  const documents = [['README.md', await readFile(path.join(root, 'README.md'), 'utf8')]];
  for (const scope of scopes) {
    for (const [name, body] of await readDocuments(path.join(root, scope))) {
      if (name.endsWith('.md')) documents.push([scope + '/' + name, body]);
    }
  }
  for (const [name, body] of documents) {
    formal(prose(body), name);
    for (const [, href] of body.matchAll(/!\[[^\]]*\]\(([^)]+\.svg)\)/g)) {
      if (href.startsWith('/img/')) images.add(path.join(root, 'website/static', href.slice(1)));
      else if (!/^[a-z]+:/i.test(href)) images.add(path.resolve(root, path.dirname(name), href));
    }
  }
  assert.ok(images.size >= 5, 'Audit the actual diagrams referenced by readers');
  for (const folder of ['intro-alpha10', 'intro-alpha11', 'pack-authors']) {
    const directory = path.join(root, 'website/static/img', folder);
    for (const file of await readdir(directory)) if (file.endsWith('.svg')) images.add(path.join(directory, file));
  }
  for (const image of images) {
    const svg = await readFile(image, 'utf8');
    for (const [, text] of svg.matchAll(/<(?:title|desc|text|tspan)\b[^>]*>([\s\S]*?)<\/(?:title|desc|text|tspan)>/g))
      formal(text.replace(/<[^>]*>/g, ''), image);
  }
});

test('CLI help descriptions and current sidebar labels use the formal name', async () => {
  function visit(value, owner) {
    if (!value || typeof value !== 'object') return;
    for (const [key, item] of Object.entries(value)) {
      if (['description', 'label'].includes(key) && typeof item === 'string') formal(item, owner);
      if (key === 'notes') for (const note of item) formal(note, owner);
      visit(item, owner);
    }
  }
  visit(JSON.parse(await readFile(path.join(root, 'src/cli/help_catalog.json'), 'utf8')), 'CLI help');
  const sidebar = await readFile(path.join(root, 'website/sidebars.ts'), 'utf8');
  for (const [, label] of sidebar.matchAll(/label: '([^']+)'/g)) formal(label, 'development sidebar');
  for (const version of saved.filter(v => ['1.0.0-alpha.10', '1.0.0-alpha.11'].includes(v)))
    visit(JSON.parse(await readFile(path.join(root, `website/versioned_sidebars/version-${version}-sidebars.json`), 'utf8')), version);
});
