import { maybeSceneById } from "../catalog/scenes";
import { controlStoresAppliedValue } from "../components/scene-controls";
import { reapplyStoredTiltGravity } from "../input/tilt-binding";
import { isWorkerSession } from "../physics/live-session";
import { maybeReadySceneId } from "./runtime";
import type { SceneRuntime } from "./scene-runtime";
import {
  beginWorkerMutation,
  publishWorkerFrame,
  resetWorkerTime,
  takeUnpublishedWorkerSteps,
} from "./worker-frame-loop";

/** Accepted mutations and their fresh snapshots share one barrier with scheduler admission. */
export async function applyWorkerControl(
  session: SceneRuntime,
  name: string,
  value: string,
): Promise<void> {
  const owner = session.maybeSession,
    context = session.maybeContext;
  if (owner === undefined || !isWorkerSession(owner) || context === undefined)
    throw new Error("Live worker control owner is unavailable");
  const generation = session.generation;
  const current = () =>
    session.maybeSession === owner && session.generation === generation;
  const beforeKind = session.view().kind;
  const sceneId = maybeReadySceneId(session.route());
  const scene = sceneId === undefined ? undefined : maybeSceneById(sceneId);
  const control = scene?.controls.find((control) => control.id === name);
  const barrier = beginWorkerMutation(session.clock, owner);
  try {
    if (control?.recreates) {
      session.maybeCanvasPointer?.cancel();
      session.setView({ kind: "loading" });
    }
    const recreated = await owner.applyControl(name, value);
    if (!current()) return;
    if (controlStoresAppliedValue(control))
      session.constructionValues = {
        ...session.constructionValues,
        [name]: value,
      };
    if (recreated) {
      resetWorkerTime(session.clock, owner);
      session.clock.maybePreviousFrame = undefined;
      session.setMaybeDebugFrame(undefined);
      session.setStepsThisFrame(0);
      session.setFpsTicks([]);
      for (const live of scene?.controls ?? []) {
        if (live.recreates || live.kind === "action") continue;
        const maybeValue = session.constructionValues[live.id];
        if (maybeValue !== undefined)
          await owner.applyControl(live.id, maybeValue);
        if (!current()) return;
      }
    }
    reapplyStoredTiltGravity(
      session.tiltBinding,
      () => session.maybeSession,
      session.gravitySliderMagnitude,
      session.setTiltDebug,
      (error) => {
        if (current()) session.failScene(error);
      },
    );
    const frame = recreated
      ? await owner.nextFrame(1)
      : await owner.captureFrame();
    if (!current() || !barrier.latest()) return;
    const kind =
      session.view().kind === "paused" ||
      (session.view().kind === "loading" && beforeKind === "paused")
        ? "paused"
        : "playing";
    const ran =
      Number(recreated) + takeUnpublishedWorkerSteps(session.clock, owner);
    publishWorkerFrame(
      session.clock,
      context,
      session.frameDeps(),
      frame,
      ran,
      kind,
      recreated,
      owner.maybeLastTiming,
    );
  } finally {
    barrier.release(context, session.frameDeps());
  }
}

export async function applyWorkerAction(
  session: SceneRuntime,
  name: string,
): Promise<void> {
  const owner = session.maybeSession,
    context = session.maybeContext;
  if (owner === undefined || !isWorkerSession(owner) || context === undefined)
    throw new Error("Live worker action owner is unavailable");
  const generation = session.generation;
  const current = () =>
    session.maybeSession === owner && session.generation === generation;
  const barrier = beginWorkerMutation(session.clock, owner);
  try {
    await owner.applyAction(name);
    if (!current()) return;
    const paused = session.view().kind === "paused";
    if (paused) {
      const settled = await owner.captureFrame();
      if (!current() || !barrier.latest()) return;
      publishWorkerFrame(
        session.clock,
        context,
        session.frameDeps(),
        settled,
        0,
        "paused",
      );
    }
    const frame = paused
      ? await owner.nextFrame(1)
      : await owner.captureFrame();
    if (!current() || !barrier.latest()) return;
    const ran =
      Number(paused) + takeUnpublishedWorkerSteps(session.clock, owner);
    publishWorkerFrame(
      session.clock,
      context,
      session.frameDeps(),
      frame,
      ran,
      session.view().kind === "paused" ? "paused" : "playing",
      false,
      owner.maybeLastTiming,
    );
  } finally {
    barrier.release(context, session.frameDeps());
  }
}
