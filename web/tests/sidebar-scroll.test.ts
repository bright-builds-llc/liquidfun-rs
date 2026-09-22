import { describe, expect, it } from "vitest";

import { scrollTopToCenterItem } from "../src/components/sidebar-scroll";

describe("scrollTopToCenterItem", () => {
  it("returns zero when the container has no visible height", () => {
    // Arrange / Act
    const scrollTop = scrollTopToCenterItem({
      containerClientHeight: 0,
      containerScrollHeight: 400,
      itemOffset: 200,
      itemHeight: 40,
    });

    // Assert
    expect(scrollTop).toBe(0);
  });

  it("keeps the first item at the top of the sidebar", () => {
    // Arrange / Act
    const scrollTop = scrollTopToCenterItem({
      containerClientHeight: 200,
      containerScrollHeight: 800,
      itemOffset: 0,
      itemHeight: 40,
    });

    // Assert
    expect(scrollTop).toBe(0);
  });

  it("centers an item that fits in the sidebar viewport", () => {
    // Arrange / Act
    const scrollTop = scrollTopToCenterItem({
      containerClientHeight: 200,
      containerScrollHeight: 800,
      itemOffset: 300,
      itemHeight: 40,
    });

    // Assert
    expect(scrollTop).toBe(220);
  });

  it("aligns an item taller than the viewport to its start", () => {
    // Arrange / Act
    const scrollTop = scrollTopToCenterItem({
      containerClientHeight: 100,
      containerScrollHeight: 800,
      itemOffset: 250,
      itemHeight: 180,
    });

    // Assert
    expect(scrollTop).toBe(250);
  });

  it("clamps the last item to the maximum scroll offset", () => {
    // Arrange / Act
    const scrollTop = scrollTopToCenterItem({
      containerClientHeight: 200,
      containerScrollHeight: 800,
      itemOffset: 760,
      itemHeight: 40,
    });

    // Assert
    expect(scrollTop).toBe(600);
  });
});
