'use strict';

// Named feelings rather than raw durations, so every press in the deck speaks the same language.
(() => {
  const PATTERNS = {
    soft: 4,
    tick: 8,
    click: 14,
    press: [10, 30, 18],
    confirm: [16, 40, 22],
    success: [10, 40, 10, 40, 26],
    warn: [28, 60, 28],
    open: [6, 40, 10, 40, 16, 40, 30],
  };

  // Safari has no vibration call, but flipping a native switch taps the phone on iOS 18 and later.
  let iosSwitch = null;
  function tapIos() {
    if (!iosSwitch) {
      iosSwitch = document.createElement('label');
      iosSwitch.className = 'sr';
      iosSwitch.setAttribute('aria-hidden', 'true');
      const input = document.createElement('input');
      input.type = 'checkbox';
      input.setAttribute('switch', '');
      input.tabIndex = -1;
      iosSwitch.append(input);
      document.body.append(iosSwitch);
    }
    iosSwitch.click();
  }

  window.haptic = (kind) => {
    try {
      if (navigator.vibrate) navigator.vibrate(PATTERNS[kind] ?? 0);
      else if (kind !== 'soft') tapIos();
    } catch {
      // A nicety: it must never get in the way of the action itself.
    }
  };
})();
