// Adapted from https://github.com/hngngn/shadcn-solid (MIT).
import type { Accessor, ComponentProps, Setter } from "solid-js"
import {
  createContext,
  createEffect,
  createMemo,
  createSignal,
  mergeProps,
  onCleanup,
  splitProps,
  useContext,
} from "solid-js"

import { useIsMobile } from "../../hooks/use-mobile"
import { combineStyle } from "../../lib/combine-style"
import { cx } from "../../lib/cva"

const SIDEBAR_COOKIE_NAME = "sidebar_state"
const SIDEBAR_COOKIE_MAX_AGE = 60 * 60 * 24 * 7
const SIDEBAR_WIDTH = "16rem"
export const SIDEBAR_WIDTH_MOBILE = "18rem"
const SIDEBAR_WIDTH_ICON = "3rem"
const SIDEBAR_KEYBOARD_SHORTCUT = "b"

interface SidebarContextProps {
  state: Accessor<"expanded" | "collapsed">
  open: Accessor<boolean>
  setOpen: (value: boolean | ((value: boolean) => boolean)) => void
  openMobile: Accessor<boolean>
  setOpenMobile: Setter<boolean>
  isMobile: Accessor<boolean>
  toggleSidebar: () => void
}

const SidebarContext = createContext<SidebarContextProps | null>(null)

export const useSidebar = () => {
  const context = useContext(SidebarContext)
  if (!context) {
    throw new Error("useSidebar must be used within a SidebarProvider.")
  }

  return context
}

export type SidebarProviderProps = ComponentProps<"div"> & {
  defaultOpen?: boolean
  open?: boolean
  onOpenChange?: (open: boolean) => void
}

export const SidebarProvider = (props: SidebarProviderProps) => {
  const merge = mergeProps<SidebarProviderProps[]>(
    {
      defaultOpen: true,
    },
    props,
  )
  const [, rest] = splitProps(merge, [
    "defaultOpen",
    "open",
    "onOpenChange",
    "class",
    "style",
    "children",
  ])

  const isMobile = useIsMobile()
  const [openMobile, setOpenMobile] = createSignal(false)

  // This is the internal state of the sidebar.
  // We use openProp and setOpenProp for control from outside the component.
  const [_open, _setOpen] = createSignal(merge.defaultOpen)
  const open = createMemo(() => merge.open ?? _open() ?? true)
  const setOpen = (value: boolean | ((value: boolean) => boolean)) => {
    const openState = typeof value === "function" ? value(open()) : value
    if (merge.onOpenChange) {
      merge.onOpenChange(openState)
    } else {
      _setOpen(openState)
    }

    // This sets the cookie to keep the sidebar state.
    document.cookie = `${SIDEBAR_COOKIE_NAME}=${openState}; path=/; max-age=${SIDEBAR_COOKIE_MAX_AGE}`
  }

  // Helper to toggle the sidebar.
  const toggleSidebar = () => {
    if (isMobile()) {
      setOpenMobile((prev) => !prev)
    } else {
      setOpen((prev) => !prev)
    }
  }

  const handleKeyDown = (event: KeyboardEvent) => {
    if (
      event.key === SIDEBAR_KEYBOARD_SHORTCUT &&
      (event.metaKey || event.ctrlKey)
    ) {
      event.preventDefault()
      toggleSidebar()
    }
  }

  createEffect(() => {
    window.addEventListener("keydown", handleKeyDown)

    onCleanup(() => {
      window.removeEventListener("keydown", handleKeyDown)
    })
  })

  const state = createMemo(() => (open() ? "expanded" : "collapsed"))

  const contextValue: SidebarContextProps = {
    state,
    open,
    setOpen,
    isMobile,
    openMobile,
    setOpenMobile,
    toggleSidebar,
  }

  return (
    <SidebarContext.Provider value={contextValue}>
      <div
        data-slot="sidebar-wrapper"
        style={combineStyle(
          {
            "--sidebar-width": SIDEBAR_WIDTH,
            "--sidebar-width-icon": SIDEBAR_WIDTH_ICON,
          },
          merge.style,
        )}
        class={cx(
          "group/sidebar-wrapper has-data-[variant=inset]:bg-sidebar flex min-h-svh w-full",
          merge.class,
        )}
        {...rest}
      >
        {merge.children}
      </div>
    </SidebarContext.Provider>
  )
}
