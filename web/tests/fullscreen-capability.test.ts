import { describe, expect, it } from "vitest";

import {
  androidPhone,
  canvasStageActive,
  fullscreenApiAvailable,
  type FormFactorSource,
} from "../src/player/fullscreen-capability";

const CHROME_ANDROID_PHONE =
  "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Mobile Safari/537.36";
const CHROME_ANDROID_TABLET =
  "Mozilla/5.0 (Linux; Android 14; SM-X900) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";
const FIREFOX_ANDROID_PHONE =
  "Mozilla/5.0 (Android 14; Mobile; rv:130.0) Gecko/130.0 Firefox/130.0";
const DESKTOP_CHROME =
  "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

function source(partial: FormFactorSource): FormFactorSource {
  return partial;
}

describe("fullscreen capability", () => {
  it("keeps the page layout when element fullscreen is available", () => {
    // Act
    const available = fullscreenApiAvailable(true);
    const canvasStage = canvasStageActive({
      fullscreenEnabled: true,
      androidPhone: false,
    });

    // Assert
    expect(available).toBe(true);
    expect(canvasStage).toBe(false);
  });

  it("uses the canvas stage when element fullscreen is unavailable", () => {
    // Act
    const available = fullscreenApiAvailable(false);
    const canvasStage = canvasStageActive({
      fullscreenEnabled: false,
      androidPhone: false,
    });

    // Assert
    expect(available).toBe(false);
    expect(canvasStage).toBe(true);
  });

  it("uses the canvas stage on an Android phone that can enter fullscreen", () => {
    // Act
    const canvasStage = canvasStageActive({
      fullscreenEnabled: true,
      androidPhone: true,
    });

    // Assert
    expect(canvasStage).toBe(true);
  });
});

describe("android phone detection", () => {
  it("trusts Chromium client hints for an Android phone", () => {
    // Arrange
    const phone = source({
      userAgent: CHROME_ANDROID_PHONE,
      maybeClientHints: { platform: "Android", mobile: true },
    });

    // Act
    const detected = androidPhone(phone);

    // Assert
    expect(detected).toBe(true);
  });

  it("keeps an Android tablet on the page layout", () => {
    // Arrange
    const tablet = source({
      userAgent: CHROME_ANDROID_TABLET,
      maybeClientHints: { platform: "Android", mobile: false },
    });

    // Act
    const detected = androidPhone(tablet);

    // Assert
    expect(detected).toBe(false);
  });

  it("ignores a spoofed Android user agent when client hints name another platform", () => {
    // Arrange
    const desktop = source({
      userAgent: CHROME_ANDROID_PHONE,
      maybeClientHints: { platform: "macOS", mobile: false },
    });

    // Act
    const detected = androidPhone(desktop);

    // Assert
    expect(detected).toBe(false);
  });

  it("uses the Mobile token when client hints are missing", () => {
    // Arrange
    const firefoxPhone = source({
      userAgent: FIREFOX_ANDROID_PHONE,
      maybeClientHints: null,
    });
    const desktop = source({
      userAgent: DESKTOP_CHROME,
      maybeClientHints: null,
    });

    // Act
    const phoneDetected = androidPhone(firefoxPhone);
    const desktopDetected = androidPhone(desktop);

    // Assert
    expect(phoneDetected).toBe(true);
    expect(desktopDetected).toBe(false);
  });
});
