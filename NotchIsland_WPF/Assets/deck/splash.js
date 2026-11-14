'use strict';

// The opening: NotchPixel's pixel N builds itself tile by tile, then flies up into the phone's own island as the deck rises.
window.Splash = (() => {
  const SIZE = 7;
  // The logo's tiles: both columns, then the diagonal down to the one accent tile.
  const TILES = [
    ...[0, 1, 2, 3, 4, 5, 6].map((row) => [0, row]),
    [1, 1], [1, 2], [2, 2], [2, 3], [3, 3], [3, 4], [4, 4], [4, 5],
    ...[6, 5, 4, 3, 2, 1, 0].map((row) => [6, row]),
    [5, 5],
  ];
  const STEP_MS = 34;
  const OUT_MS = 720;
  const CODE_DIGITS = 6;

  const splash = document.getElementById('splash');
  const note = document.getElementById('splash-note');
  const enter = document.getElementById('enter');

  const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const reduced = () => matchMedia('(prefers-reduced-motion: reduce)').matches;

  function buildLogo() {
    const logo = document.getElementById('logo');
    TILES.forEach(([x, y], index) => {
      const tile = document.createElement('i');
      tile.style.setProperty('--x', x);
      tile.style.setProperty('--y', y);
      tile.style.setProperty('--d', `${index * STEP_MS}ms`);
      tile.style.setProperty('--shade', (index % 2 ? 0.74 : 1).toString());
      if (index === TILES.length - 1) tile.className = 'accent';
      logo.append(tile);
    });
    logo.style.setProperty('--size', SIZE);
  }

  async function play() {
    buildLogo();
    await wait(30);
    splash.dataset.stage = 'build';
    await wait(reduced() ? 80 : TILES.length * STEP_MS + 320);
    splash.dataset.stage = 'built';
    await wait(reduced() ? 0 : 260);
  }

  function say(text, kind = '') {
    note.textContent = text;
    note.dataset.kind = kind;
  }

  // Tapping is what lets a page go full screen and buzz, so the deck opens on the person's own tap.
  function waitForTap() {
    say('');
    enter.hidden = false;
    return new Promise((resolve) => {
      enter.addEventListener('click', () => {
        enter.hidden = true;
        resolve();
      }, { once: true });
    });
  }

  // The digits from the island, on keys of the page's own: a keyboard would cover half of it, most of it sideways.
  function askCode() {
    const entry = document.getElementById('code-entry');
    const slots = document.getElementById('code-slots');
    const keys = document.getElementById('code-keys');
    let digits = '';

    const showDigits = () => {
      slots.replaceChildren(...Array.from({ length: CODE_DIGITS }, (_, index) => {
        const slot = document.createElement('span');
        slot.textContent = digits[index] ?? '';
        if (index === digits.length) slot.className = 'next';
        return slot;
      }));
      slots.setAttribute('aria-label', digits.length ? `Code so far: ${digits.split('').join(' ')}` : 'Code');
    };

    return new Promise((resolve) => {
      const makeKey = (label, onPress, content) => {
        const key = document.createElement('button');
        key.type = 'button';
        key.setAttribute('aria-label', label);
        key.append(content ?? label);
        key.addEventListener('pointerdown', () => window.haptic('tick'));
        key.addEventListener('click', onPress);
        return key;
      };
      const press = (digit) => {
        if (digits.length === CODE_DIGITS) return;
        digits += digit;
        showDigits();
        if (digits.length < CODE_DIGITS) return;
        entry.hidden = true;
        delete splash.dataset.entry;
        resolve(digits);
      };
      const erase = () => {
        digits = digits.slice(0, -1);
        showDigits();
      };
      const gap = document.createElement('span');
      keys.replaceChildren(
        ...['1', '2', '3', '4', '5', '6', '7', '8', '9'].map((digit) => makeKey(digit, () => press(digit))),
        gap,
        makeKey('0', () => press('0')),
        makeKey('Erase', erase, Ui.icon('erase')),
      );
      showDigits();
      splash.dataset.entry = 'on';
      entry.hidden = false;
    });
  }

  async function leave() {
    splash.dataset.stage = 'out';
    await wait(reduced() ? 60 : OUT_MS);
    splash.hidden = true;
  }

  return { play, say, waitForTap, askCode, leave };
})();
