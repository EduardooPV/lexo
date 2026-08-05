import { describe, it, expect, beforeEach } from 'vitest';
import { mountOverlay, flush, el } from '../helpers/mount.js';

function mouse(type, clientX, clientY, button = 0) {
  window.dispatchEvent(new MouseEvent(type, { clientX, clientY, button, bubbles: true }));
}

let mock;

beforeEach(async () => {
  mock = await mountOverlay();
});

describe('idle state', () => {
  it('starts with the hint up and no selection drawn', () => {
    expect(el('selection').hidden).toBe(true);
    expect(el('hint').hidden).toBe(false);
  });
});

describe('dragging a region', () => {
  it('reveals the selection and gets the hint out of the way', () => {
    mouse('mousedown', 100, 120);
    expect(el('selection').hidden).toBe(false);
    expect(el('hint').hidden).toBe(true);
    expect(el('backdrop').style.opacity).toBe('0');
  });

  it('tracks the rectangle and labels it with its size', () => {
    mouse('mousedown', 100, 120);
    mouse('mousemove', 320, 210);

    expect(el('selection').style.left).toBe('100px');
    expect(el('selection').style.top).toBe('120px');
    expect(el('selection').style.width).toBe('220px');
    expect(el('selection').style.height).toBe('90px');
    expect(el('size').textContent).toBe('220 × 90');
  });

  it('normalises a rectangle dragged up-and-left instead of inverting it', () => {
    mouse('mousedown', 100, 120);
    mouse('mousemove', 40, 60);

    expect(el('selection').style.left).toBe('40px');
    expect(el('selection').style.top).toBe('60px');
    expect(el('selection').style.width).toBe('60px');
    expect(el('selection').style.height).toBe('60px');
  });

  it('ignores movement before a drag has started', () => {
    mouse('mousemove', 300, 300);
    expect(el('selection').hidden).toBe(true);
  });
});

describe('finishing a drag', () => {
  it('sends the rectangle in CSS pixels and resets the overlay', async () => {
    mouse('mousedown', 100, 120);
    mouse('mousemove', 320, 210);
    mouse('mouseup', 320, 210);
    await flush(1);

    expect(mock.callsFor('ocr_region')[0].args).toEqual({
      x: 100,
      y: 120,
      width: 220,
      height: 90
    });
    expect(el('selection').hidden).toBe(true);
    expect(el('hint').hidden).toBe(false);
    expect(el('backdrop').style.opacity).toBe('1');
  });

  it('cancels instead of capturing when the box is a stray click', async () => {
    mouse('mousedown', 100, 120);
    mouse('mousemove', 103, 122);
    mouse('mouseup', 103, 122);
    await flush(1);

    expect(mock.callsFor('ocr_region')).toHaveLength(0);
    expect(mock.callsFor('cancel_region_capture')).toHaveLength(1);
  });

  it('does nothing on mouseup without a drag in progress', async () => {
    mouse('mouseup', 50, 50);
    await flush(1);
    expect(mock.callsFor('ocr_region')).toHaveLength(0);
    expect(mock.callsFor('cancel_region_capture')).toHaveLength(0);
  });
});

describe('cancelling', () => {
  it('backs out on Escape', async () => {
    mouse('mousedown', 100, 120);
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    await flush(1);

    expect(mock.callsFor('cancel_region_capture')).toHaveLength(1);
    expect(el('selection').hidden).toBe(true);
  });

  it('backs out on a right click', async () => {
    mouse('mousedown', 100, 120, 2);
    await flush(1);
    expect(mock.callsFor('cancel_region_capture')).toHaveLength(1);
  });
});
