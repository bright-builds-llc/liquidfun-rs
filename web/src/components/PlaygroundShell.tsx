import { createEffect, Show, type JSX } from "solid-js";

import { PUZZLES_HASH } from "../catalog/puzzle-levels";
import type { SceneId } from "../catalog/scenes";
import { DemoNavigation } from "./DemoNavigation";
import { SiteFooter } from "./SiteFooter";
import { SiteHeader } from "./SiteHeader";
import { DrawerDescription, DrawerLabel } from "./ui/drawer";
import {
  Sidebar,
  SidebarContent,
  SidebarProvider,
  SidebarTrigger,
  useSidebar,
} from "./ui/sidebar";

export type PlaygroundShellProps = {
  readonly canvasStage: boolean;
  readonly maybeCurrentSceneId: SceneId | undefined;
  /** `aria-current` of the sidebar Puzzles link (UI-D24). */
  readonly maybePuzzlesCurrent: "page" | "true" | undefined;
  readonly routeIdentity: string;
  readonly children: JSX.Element;
};

/** Playground chrome whose catalog sidebar slides away on desktop and becomes a drawer on small screens. */
export function PlaygroundShell(props: PlaygroundShellProps) {
  return (
    <SidebarProvider>
      <PlaygroundFrame
        canvasStage={props.canvasStage}
        maybeCurrentSceneId={props.maybeCurrentSceneId}
        maybePuzzlesCurrent={props.maybePuzzlesCurrent}
        routeIdentity={props.routeIdentity}
      >
        {props.children}
      </PlaygroundFrame>
    </SidebarProvider>
  );
}

function PlaygroundFrame(props: PlaygroundShellProps) {
  const sidebar = useSidebar();
  let mobileDrawerWasOpen = false;

  createEffect(() => {
    props.routeIdentity;
    sidebar.setOpenMobile(false);
  });

  createEffect(() => {
    const isOpen = sidebar.openMobile();
    if (mobileDrawerWasOpen && !isOpen) {
      queueMicrotask(() => {
        document
          .querySelector<HTMLButtonElement>("[data-slot='sidebar-trigger']")
          ?.focus();
      });
    }
    mobileDrawerWasOpen = isOpen;
  });

  return (
    <>
      <Sidebar class="demo-sidebar" collapsible="offcanvas">
        <Show when={sidebar.isMobile()}>
          <DrawerLabel class="visually-hidden">Demos</DrawerLabel>
          <DrawerDescription class="visually-hidden">
            Choose a LiquidFun simulation.
          </DrawerDescription>
        </Show>
        <SidebarContent class="demo-sidebar-content">
          {/* Text only: no preview or icon, so the scene list preview counts hold (D-03). */}
          <nav class="puzzles-navigation" aria-label="Puzzles">
            <a
              class="puzzles-nav-link"
              href={PUZZLES_HASH}
              aria-current={props.maybePuzzlesCurrent}
              onClick={() => sidebar.setOpenMobile(false)}
            >
              Puzzles
            </a>
          </nav>
          <DemoNavigation
            label="Demos"
            maybeCurrentSceneId={props.maybeCurrentSceneId}
            revealCurrent={
              sidebar.isMobile() ? sidebar.openMobile() : sidebar.open()
            }
            onNavigate={() => sidebar.setOpenMobile(false)}
          />
        </SidebarContent>
      </Sidebar>
      <div
        class="app-shell"
        aria-hidden={sidebar.openMobile() ? "true" : undefined}
      >
        <SiteHeader
          canvasStage={props.canvasStage}
          mobileNavigationTrigger={
            <SidebarTrigger class="mobile-demos-trigger" aria-label="Demos" />
          }
        />
        <div class="app-body">{props.children}</div>
        <Show when={!props.canvasStage}>
          <SiteFooter />
        </Show>
      </div>
    </>
  );
}
