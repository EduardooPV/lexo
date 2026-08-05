import { describe, it, expect, beforeEach } from 'vitest';
import { hydrate, iconMarkup, setIcon } from '../../src/icons.js';

beforeEach(() => {
  document.body.innerHTML = '';
});

describe('iconMarkup', () => {
  it('renders an inline svg carrying the shared icon class', () => {
    const markup = iconMarkup('copy');
    expect(markup).toContain('<svg');
    expect(markup).toContain('class="icon"');
    expect(markup).toContain('viewBox="0 0 24 24"');
  });

  it('draws every glyph on the same grid and stroke so they look like one family', () => {
    for (const name of ['copy', 'close', 'search', 'star', 'trash', 'scan', 'swap']) {
      const markup = iconMarkup(name);
      expect(markup, name).toContain('viewBox="0 0 24 24"');
      expect(markup, name).toContain('stroke-width="2"');
      expect(markup, name).toContain('stroke="currentColor"');
    }
  });

  it('hides icons from assistive tech, since every one sits next to a label', () => {
    expect(iconMarkup('copy')).toContain('aria-hidden="true"');
  });

  it('returns empty string for an unknown glyph instead of broken markup', () => {
    expect(iconMarkup('does-not-exist')).toBe('');
    expect(iconMarkup(undefined)).toBe('');
  });
});

describe('hydrate', () => {
  it('fills every [data-icon] element in the tree', () => {
    document.body.innerHTML = `
      <button data-icon="copy"></button>
      <span data-icon="search"></span>
      <div><button data-icon="trash"></button></div>
    `;
    hydrate();
    expect(document.querySelectorAll('[data-icon] svg.icon')).toHaveLength(3);
  });

  it('replaces rather than appends, so re-rendering a list cannot stack icons', () => {
    document.body.innerHTML = '<button data-icon="copy"></button>';
    hydrate();
    hydrate();
    hydrate();
    expect(document.querySelectorAll('[data-icon] svg')).toHaveLength(1);
  });

  it('can be scoped to a subtree', () => {
    document.body.innerHTML = `
      <div id="inside"><button data-icon="copy"></button></div>
      <div id="outside"><button data-icon="trash"></button></div>
    `;
    hydrate(document.getElementById('inside'));
    expect(document.querySelectorAll('#inside svg')).toHaveLength(1);
    expect(document.querySelectorAll('#outside svg')).toHaveLength(0);
  });

  it('leaves an unknown glyph empty without throwing', () => {
    document.body.innerHTML = '<button data-icon="nope"></button>';
    expect(() => hydrate()).not.toThrow();
    expect(document.querySelector('[data-icon]').innerHTML).toBe('');
  });
});

describe('setIcon', () => {
  it('swaps the data attribute too, so a later hydrate cannot restore the old glyph', () => {
    document.body.innerHTML = '<button id="btn" data-icon="copy"></button>';
    const button = document.getElementById('btn');
    hydrate();

    setIcon(button, 'check');

    expect(button.dataset.icon).toBe('check');
    expect(button.querySelectorAll('svg')).toHaveLength(1);
    hydrate();
    expect(button.dataset.icon).toBe('check');
  });
});
