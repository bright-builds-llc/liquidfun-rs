// Adapted from https://github.com/hngngn/shadcn-solid (MIT).
import type { ComponentProps, JSX, ValidComponent } from "solid-js";
import { Match, Show, Switch, mergeProps, splitProps } from "solid-js";

import { callHandler } from "../../lib/call-handler";
import { cx } from "../../lib/cva";
import { Button } from "./button";
import { Drawer, DrawerContent } from "./drawer";
import {
  SIDEBAR_WIDTH_MOBILE,
  SidebarProvider,
  useSidebar,
} from "./sidebar-context";

export { SidebarProvider, useSidebar };

export type SidebarProps = ComponentProps<"div"> & {
  side?: "left" | "right"
  variant?: "sidebar" | "floating" | "inset"
  collapsible?: "offcanvas" | "icon" | "none"
}

export const Sidebar = (props: SidebarProps) => {
  const merge = mergeProps<SidebarProps[]>(
    {
      side: "left",
      variant: "sidebar",
      collapsible: "offcanvas",
    },
    props,
  )
  const [, rest] = splitProps(merge, [
    "side",
    "variant",
    "collapsible",
    "class",
    "children",
  ])

  const { isMobile, state, openMobile, setOpenMobile } = useSidebar()

  return (
    <Switch
      fallback={
        <div
          class="text-sidebar-foreground group peer hidden md:block"
          data-state={state()}
          data-collapsible={state() === "collapsed" ? merge.collapsible : ""}
          data-variant={merge.variant}
          data-side={merge.side}
          data-slot="sidebar"
        >
          {/* This is what handles the sidebar gap on desktop */}
          <div
            data-slot="sidebar-gap"
            class={cx(
              "relative w-(--sidebar-width) bg-transparent transition-[width] duration-200 ease-linear",
              "group-data-[collapsible=offcanvas]:w-0",
              "group-data-[side=right]:rotate-180",
              merge.variant === "floating" || merge.variant === "inset"
                ? "group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4)))]"
                : "group-data-[collapsible=icon]:w-(--sidebar-width-icon)",
            )}
          />
          <div
            data-slot="sidebar-container"
            class={cx(
              "fixed inset-y-0 z-10 hidden h-svh w-(--sidebar-width) transition-[left,right,width] duration-200 ease-linear md:flex",
              merge.side === "left"
                ? "left-0 group-data-[collapsible=offcanvas]:left-[calc(var(--sidebar-width)*-1)]"
                : "right-0 group-data-[collapsible=offcanvas]:right-[calc(var(--sidebar-width)*-1)]",
              // Adjust the padding for floating and inset variants.
              merge.variant === "floating" || merge.variant === "inset"
                ? "p-2 group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4))+2px)]"
                : "group-data-[collapsible=icon]:w-(--sidebar-width-icon) group-data-[side=left]:border-r group-data-[side=right]:border-l",
              merge.class,
            )}
            {...rest}
          >
            <div
              data-sidebar="sidebar"
              data-slot="sidebar-inner"
              class="bg-sidebar group-data-[variant=floating]:border-sidebar-border flex h-full w-full flex-col group-data-[variant=floating]:rounded-lg group-data-[variant=floating]:border group-data-[variant=floating]:shadow-sm"
            >
              {merge.children}
            </div>
          </div>
        </div>
      }
    >
      <Match when={merge.collapsible === "none"}>
        <div
          data-slot="sidebar"
          class={cx(
            "bg-sidebar text-sidebar-foreground flex h-full w-(--sidebar-width) flex-col",
            merge.class,
          )}
          {...rest}
        >
          {merge.children}
        </div>
      </Match>
      <Match when={isMobile()}>
        <Drawer
          open={openMobile()}
          onOpenChange={setOpenMobile}
          side={merge.side ?? "left"}
        >
          <DrawerContent
            data-sidebar="sidebar"
            data-slot="sidebar"
            data-mobile="true"
            class="bg-sidebar text-sidebar-foreground w-(--sidebar-width) p-0 [&>button]:hidden"
            style={{
              "--sidebar-width": SIDEBAR_WIDTH_MOBILE,
            }}
          >
            <div class="flex h-full w-full flex-col">{merge.children}</div>
          </DrawerContent>
        </Drawer>
      </Match>
    </Switch>
  )
}

export type SidebarTriggerProps<T extends ValidComponent = "button"> =
  ComponentProps<typeof Button<T>>

export const SidebarTrigger = <T extends ValidComponent = "button">(
  props: SidebarTriggerProps<T>,
) => {
  const [, rest] = splitProps(props as SidebarTriggerProps, [
    "class",
    "onClick",
  ])
  const { toggleSidebar, open } = useSidebar()

  const handleOnclick: JSX.EventHandlerUnion<HTMLButtonElement, MouseEvent> = (
    event,
  ) => {
    callHandler(event, props.onClick)
    toggleSidebar()
  }

  return (
    <Button
      data-sidebar="trigger"
      data-slot="sidebar-trigger"
      variant="ghost"
      size="icon"
      class={cx("size-7", props.class)}
      onClick={handleOnclick}
      {...rest}
    >
      <Show
        when={!open()}
        fallback={
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="size-4"
            viewBox="0 0 24 24"
          >
            <g
              fill="none"
              stroke="currentColor"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
            >
              <rect width="18" height="18" x="3" y="3" rx="2" />
              <path d="M9 3v18m7-6l-3-3l3-3" />
            </g>
          </svg>
        }
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="size-4"
          viewBox="0 0 24 24"
        >
          <g
            fill="none"
            stroke="currentColor"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
          >
            <rect width="18" height="18" x="3" y="3" rx="2" />
            <path d="M9 3v18m5-12l3 3l-3 3" />
          </g>
        </svg>
      </Show>
    </Button>
  )
}

export type SidebarRailProps = ComponentProps<"button">

export const SidebarRail = (props: SidebarRailProps) => {
  const { toggleSidebar } = useSidebar()
  const [, rest] = splitProps(props, ["class", "onClick"])

  return (
    <button
      data-sidebar="rail"
      data-slot="sidebar-rail"
      aria-label="Toggle Sidebar"
      tabIndex={-1}
      onClick={toggleSidebar}
      title="Toggle Sidebar"
      class={cx(
        "hover:after:bg-sidebar-border absolute inset-y-0 z-20 hidden w-4 -translate-x-1/2 transition-all ease-linear group-data-[side=left]:-right-4 group-data-[side=right]:left-0 after:absolute after:inset-y-0 after:left-1/2 after:w-[2px] sm:flex",
        "in-data-[side=left]:cursor-w-resize in-data-[side=right]:cursor-e-resize",
        "[[data-side=left][data-state=collapsed]_&]:cursor-e-resize [[data-side=right][data-state=collapsed]_&]:cursor-w-resize",
        "hover:group-data-[collapsible=offcanvas]:bg-sidebar group-data-[collapsible=offcanvas]:translate-x-0 group-data-[collapsible=offcanvas]:after:left-full",
        "[[data-side=left][data-collapsible=offcanvas]_&]:-right-2",
        "[[data-side=right][data-collapsible=offcanvas]_&]:-left-2",
        props.class,
      )}
      {...rest}
    />
  )
}

export type SidebarInsetProps = ComponentProps<"main">

export const SidebarInset = (props: SidebarInsetProps) => {
  const [, rest] = splitProps(props, ["class"])

  return (
    <main
      data-slot="sidebar-inset"
      class={cx(
        "bg-background relative flex w-full flex-1 flex-col",
        "md:peer-data-[variant=inset]:m-2 md:peer-data-[variant=inset]:ml-0 md:peer-data-[variant=inset]:rounded-xl md:peer-data-[variant=inset]:shadow-sm md:peer-data-[variant=inset]:peer-data-[state=collapsed]:ml-2",
        props.class,
      )}
      {...rest}
    />
  )
}

export type SidebarContentProps = ComponentProps<"div">

export const SidebarContent = (props: SidebarContentProps) => {
  const [, rest] = splitProps(props, ["class"])

  return (
    <div
      data-slot="sidebar-content"
      data-sidebar="content"
      class={cx(
        "flex min-h-0 flex-1 flex-col gap-2 overflow-auto group-data-[collapsible=icon]:overflow-hidden",
        props.class,
      )}
      {...rest}
    />
  )
}
