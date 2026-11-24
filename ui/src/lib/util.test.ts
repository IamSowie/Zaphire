import { describe, expect, it } from "vitest";
import { createMock } from "./mock";
import type { Event, State } from "./types";
import { activeTab, displayUrl, tabTitle } from "./util";

describe("displayUrl", () => {
  it("strips scheme and www", () => {
    expect(displayUrl("https://www.rust-lang.org/")).toBe("rust-lang.org");
    expect(displayUrl("https://github.com/IamSowie/zaphire")).toBe("github.com/IamSowie/zaphire");
  });
  it("hides the new-tab page and keeps internal urls", () => {
    expect(displayUrl("zaphire://app/newtab")).toBe("");
    expect(displayUrl("chrome://extensions")).toBe("chrome://extensions");
  });
});

describe("mock core", () => {
  const run = () => {
    let last: State | null = null;
    const send = createMock((e: Event) => {
      if (e.type === "state") last = e;
    });
    return { send, get: () => last! };
  };

  it("opens a tab after the active one and closes to the right neighbour", async () => {
    const m = run();
    m.send({ type: "newTab", url: null });
    await new Promise((r) => setTimeout(r, 1));
    const ws = m.get().session.workspaces[0];
    expect(ws.tabs).toEqual([2, 3, 10, 4, 5]);
    expect(tabTitle(activeTab(m.get().session)!)).toBe("Nueva pestaña");
    m.send({ type: "closeTab", id: 10 });
    await new Promise((r) => setTimeout(r, 1));
    expect(m.get().session.workspaces[0].active).toBe(4);
  });
});

describe("applyTheme", () => {
  it("sets theme, background style and custom colours", async () => {
    const { applyTheme } = await import("./util");
    const m = (() => {
      let st: State | null = null;
      createMock((e) => {
        if (e.type === "state") st = e;
      })({ type: "ready" });
      return () => st!;
    })();
    await new Promise((r) => setTimeout(r, 1));
    const settings = m().settings;
    settings.theme = { ...settings.theme, mode: "light", background: "solid", background_color: "#ffffff", density: "compact", scale: 2 };
    applyTheme(settings);
    const root = document.documentElement;
    expect(root.dataset.theme).toBe("light");
    expect(root.dataset.bg).toBe("solid");
    expect(root.dataset.density).toBe("compact");
    expect(root.style.getPropertyValue("--z-bg")).toBe("#ffffff");
    expect(root.style.getPropertyValue("zoom")).toBe("1.25");
    settings.theme.background_color = "not-a-colour";
    applyTheme(settings);
    expect(root.style.getPropertyValue("--z-bg")).toBe("");
  });
});
