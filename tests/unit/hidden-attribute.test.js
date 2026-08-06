import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const read = (name) => readFileSync(join(process.cwd(), 'src', name), 'utf8');

const MARKUP = ['index.html', 'mini.html', 'overlay.html'];
const STYLESHEETS = ['base.css', 'styles.css', 'mini.css', 'overlay.css'];

function classesOnHiddenElements() {
  const found = new Set();
  for (const file of MARKUP) {
    const html = read(file);
    for (const tag of html.match(/<[a-z]+[^>]*\shidden[\s/>]/gi) || []) {
      const classes = tag.match(/class="([^"]+)"/i);
      if (classes) classes[1].split(/\s+/).forEach((name) => found.add(name));
    }
  }
  return [...found];
}

function displayRulesFor(className) {
  const rules = [];
  for (const file of STYLESHEETS) {
    let css;
    try {
      css = read(file);
    } catch (_) {
      continue;
    }
    const pattern = new RegExp(`\\.${className}\\s*(?:,[^{]*)?\\{([^}]*)\\}`, 'g');
    for (const match of css.matchAll(pattern)) {
      if (/(^|;)\s*display\s*:/.test(match[1])) rules.push(`${file}: .${className}`);
    }
  }
  return rules;
}

describe('the hidden attribute', () => {
  it('is enforced by a rule strong enough to beat any author display', () => {
    const guard = read('base.css').match(/\[hidden\]\s*\{([^}]*)\}/);

    expect(guard, 'base.css must declare a [hidden] rule').not.toBeNull();
    expect(guard[1].replace(/\s+/g, '')).toContain('display:none!important');
  });

  it('covers every class that ships hidden and also sets its own display', () => {
    const risky = classesOnHiddenElements().flatMap(displayRulesFor);

    expect(risky.length, `these need the [hidden] guard: ${risky.join(', ')}`).toBeGreaterThan(0);
  });
});
