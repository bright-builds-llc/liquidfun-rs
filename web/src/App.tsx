import { PlaygroundStage } from "./components/PlaygroundStage";
import { createPlaygroundSession } from "./player/playground-session";
import { cachedWaterSupport } from "./render/water-capability";

/** One-session playground shell with hash routing and bounded playback. */
export function App() {
  const session = createPlaygroundSession();
  // Probed once per page on a detached canvas (UI-D48).
  const waterAvailable = cachedWaterSupport();

  return (
    <PlaygroundStage
      canvasStage={session.canvasStage}
      route={session.route}
      view={session.view}
      lastPointerKind={session.lastPointerKind}
      pointerAccepted={session.pointerAccepted}
      renderMode={session.appearance.renderMode}
      waterAvailable={waterAvailable}
      wireframeStrokeWidth={session.appearance.wireframeStrokeWidth}
      densityShading={session.appearance.densityShading}
      renderedParticleDraft={session.renderedParticleDraft}
      panEnabled={session.panEnabled}
      maybePixelsPerMeter={session.maybePixelsPerMeter}
      tiltGravityEnabled={session.tiltGravityEnabled}
      tiltDebug={session.tiltDebug}
      debugEnabled={session.debugEnabled}
      maybeDebugFrame={session.maybeDebugFrame}
      stepsThisFrame={session.stepsThisFrame}
      fpsTicks={session.fpsTicks}
      resetGeneration={session.resetGeneration}
      constructionValues={session.constructionValues}
      assignCanvas={session.assignCanvas}
      assignParticleSurface={session.assignParticleSurface}
      onPlay={session.playScene}
      onPause={session.pauseScene}
      onReset={session.recreateScene}
      onRenderModeChange={session.appearance.changeRenderMode}
      onWireframeStrokeWidthChange={session.appearance.changeWireframeStrokeWidth}
      onDensityShadingChange={session.appearance.changeDensityShading}
      onDebugEnabledChange={session.setDebugEnabled}
      onRenderedParticleDraft={session.changeRenderedParticleDraft}
      onZoomIn={session.zoomIn}
      onZoomOut={session.zoomOut}
      onResetZoom={session.resetCameraView}
      onPanEnabledChange={session.changePanEnabled}
      onHudObstaclesChange={session.changeHudObstacles}
      onTiltGravityEnabledChange={session.changeTiltGravityEnabled}
      onApplyControl={session.applySceneControl}
      onApplyAction={session.applySceneAction}
      onCreateSvgExportRequest={session.createSvgExportRequest}
      puzzleController={session.puzzleController}
      puzzleProgress={session.puzzle.progress}
      assignPuzzleCanvas={session.assignPuzzleCanvas}
    />
  );
}
