import { describe, it, expect } from 'vitest';
import { DEFAULT_APPEARANCE, THEMES, applyAppearance, readableOn } from '../../src/theme.js';

describe('readableOn', () => {
  it('picks dark text over light backgrounds and light text over dark ones', () => {
    expect(readableOn('#ffffff')).toBe('#1b1b1f');
    expect(readableOn('#000000')).toBe('#ffffff');
  });

  it('picks dark text over the default accent', () => {
    expect(readableOn(DEFAULT_APPEARANCE.accent)).toBe('#1b1b1f');
  });

  it('falls back to white for malformed input rather than throwing', () => {
    expect(readableOn('')).toBe('#ffffff');
    expect(readableOn('#fff')).toBe('#ffffff');
    expect(readableOn(null)).toBe('#ffffff');
    expect(readableOn(undefined)).toBe('#ffffff');
  });

  it('tolerates a missing leading hash', () => {
    expect(readableOn('ffffff')).toBe('#1b1b1f');
  });
});

describe('applyAppearance', () => {
  it('writes every themed value onto the document root', () => {
    applyAppearance({
      opacity: 0.8,
      bg: '#101010',
      accent: '#00ff00',
      text: '#fafafa',
      font: 'Arial, sans-serif'
    });

    const root = document.documentElement.style;
    expect(root.getPropertyValue('--parchment')).toBe('#101010');
    expect(root.getPropertyValue('--gold')).toBe('#00ff00');
    expect(root.getPropertyValue('--ink')).toBe('#fafafa');
    expect(root.getPropertyValue('--ui-font')).toBe('Arial, sans-serif');
    expect(root.getPropertyValue('--app-opacity')).toBe('0.8');
  });

  it('derives the on-accent colour instead of trusting the caller', () => {
    applyAppearance({ ...DEFAULT_APPEARANCE, accent: '#000000' });
    expect(document.documentElement.style.getPropertyValue('--on-accent')).toBe('#ffffff');
  });

  it('is a no-op when given nothing', () => {
    expect(() => applyAppearance(null)).not.toThrow();
    expect(() => applyAppearance(undefined)).not.toThrow();
  });
});

describe('THEMES', () => {
  it('gives every preset the three colours the appearance panel reads', () => {
    for (const theme of THEMES) {
      expect(theme.name).toBeTruthy();
      expect(theme.bg).toMatch(/^#[0-9a-f]{6}$/i);
      expect(theme.accent).toMatch(/^#[0-9a-f]{6}$/i);
      expect(theme.text).toMatch(/^#[0-9a-f]{6}$/i);
    }
  });

  it('keeps preset colours lowercase, or syncThemeSelect always reads "Custom"', () => {
    for (const theme of THEMES) {
      expect(theme.bg).toBe(theme.bg.toLowerCase());
      expect(theme.accent).toBe(theme.accent.toLowerCase());
      expect(theme.text).toBe(theme.text.toLowerCase());
    }
  });

  it('ships the default appearance as the first preset', () => {
    expect(THEMES[0].bg).toBe(DEFAULT_APPEARANCE.bg);
    expect(THEMES[0].accent).toBe(DEFAULT_APPEARANCE.accent);
    expect(THEMES[0].text).toBe(DEFAULT_APPEARANCE.text);
  });
});
