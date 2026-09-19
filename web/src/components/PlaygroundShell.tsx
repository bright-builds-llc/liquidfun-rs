import { Dialog } from "@kobalte/core/dialog";
import { createEffect, createSignal, type JSX } from "solid-js";

import type { SceneId } from "../catalog/scenes";
import { DemoNavigation } from "./DemoNavigation";
import { SiteFooter } from "./SiteFooter";
import { SiteHeader } from "./SiteHeader";

export type PlaygroundShellProps = {
  readonly maybeCurrentSceneId: SceneId | undefined;
  readonly routeIdentity: string;
  readonly children: JSX.Element;
};

/** Responsive playground chrome with desktop navigation and a modal drawer. */
export function PlaygroundShell(props: PlaygroundShellProps) {
  const [drawerOpen, setDrawerOpen] = createSignal(false);

  createEffect(() => {
    props.routeIdentity;
    setDrawerOpen(false);
  });

  return (
    <Dialog open={drawerOpen()} onOpenChange={setDrawerOpen} modal>
      <div class="app-shell">
        <SiteHeader
          mobileNavigationTrigger={
            <Dialog.Trigger class="mobile-demos-trigger">
              Demos
            </Dialog.Trigger>
          }
        />
        <div class="app-body">
          <aside class="demo-sidebar">
            <DemoNavigation
              label="Demos"
              maybeCurrentSceneId={props.maybeCurrentSceneId}
            />
          </aside>
          {props.children}
        </div>
        <SiteFooter />
      </div>
      <Dialog.Portal>
        <Dialog.Overlay class="demo-drawer-overlay" />
        <div class="demo-drawer-positioner">
          <Dialog.Content class="demo-drawer">
            <div class="demo-drawer-header">
              <Dialog.Title>Demos</Dialog.Title>
              <Dialog.CloseButton>Close</Dialog.CloseButton>
            </div>
            <Dialog.Description>
              Choose a LiquidFun simulation.
            </Dialog.Description>
            <DemoNavigation
              label="Mobile demos"
              maybeCurrentSceneId={props.maybeCurrentSceneId}
              onNavigate={() => setDrawerOpen(false)}
            />
          </Dialog.Content>
        </div>
      </Dialog.Portal>
    </Dialog>
  );
}
