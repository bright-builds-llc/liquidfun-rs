import { createEffect, Show, type JSX } from "solid-js";

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
  readonly maybeCurrentSceneId: SceneId | undefined;
  readonly routeIdentity: string;
  readonly children: JSX.Element;
};

/** Playground chrome whose catalog sidebar slides away on desktop and becomes a drawer on small screens. */
export function PlaygroundShell(props: PlaygroundShellProps) {
  return (
    <SidebarProvider>
      <PlaygroundFrame
        maybeCurrentSceneId={props.maybeCurrentSceneId}
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
          mobileNavigationTrigger={
            <SidebarTrigger class="mobile-demos-trigger" aria-label="Demos" />
          }
        />
        <div class="app-body">{props.children}</div>
        <SiteFooter />
      </div>
    </>
  );
}
