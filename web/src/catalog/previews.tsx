import {
  ColorMixerPreview,
  DamBreakPreview,
  FloatOrSinkPreview,
  FountainPreview,
  JellyDropPreview,
  WaterWheelPreview,
  WaveMachinePreview,
} from "./previews-classic";
import {
  HydraulicFountainPreview,
  LiquidBubblerPreview,
  StackedDripPreview,
  TeslaValvePreview,
  WashingMachinePreview,
  WaveTankPreview,
} from "./previews-machines";
import {
  DrawingParticlesPreview,
  ElasticParticlesPreview,
  ImpulsePreview,
  LiquidTimerPreview,
  LiquidTumblerPreview,
  ParticlesPreview,
  RigidParticlesPreview,
  SoupPreview,
  SoupStirrerPreview,
  SparkyPreview,
  SurfaceTensionPreview,
  TheoJansenPreview,
} from "./previews-testbed";
import type { SceneId } from "./scenes";

export type ScenePreviewProps = {
  readonly sceneId: SceneId;
};

/** Static token-only SVG preview. Never starts a WASM session. */
export function ScenePreview(props: ScenePreviewProps) {
  switch (props.sceneId) {
    case "dam-break":
      return <DamBreakPreview />;
    case "fountain":
      return <FountainPreview />;
    case "float-or-sink":
      return <FloatOrSinkPreview />;
    case "color-mixer":
      return <ColorMixerPreview />;
    case "jelly-drop":
      return <JellyDropPreview />;
    case "water-wheel":
      return <WaterWheelPreview />;
    case "particles":
      return <ParticlesPreview />;
    case "liquid-timer":
      return <LiquidTimerPreview />;
    case "surface-tension":
      return <SurfaceTensionPreview />;
    case "elastic-particles":
      return <ElasticParticlesPreview />;
    case "rigid-particles":
      return <RigidParticlesPreview />;
    case "soup":
      return <SoupPreview />;
    case "soup-stirrer":
      return <SoupStirrerPreview />;
    case "impulse":
      return <ImpulsePreview />;
    case "wave-machine":
      return <WaveMachinePreview />;
    case "theo-jansen":
      return <TheoJansenPreview />;
    case "liquid-tumbler":
      return <LiquidTumblerPreview />;
    case "drawing-particles":
      return <DrawingParticlesPreview />;
    case "sparky":
      return <SparkyPreview />;
    case "hydraulic-fountain":
      return <HydraulicFountainPreview />;
    case "wave-tank":
      return <WaveTankPreview />;
    case "liquid-bubbler":
      return <LiquidBubblerPreview />;
    case "stacked-drip":
      return <StackedDripPreview />;
    case "washing-machine":
      return <WashingMachinePreview />;
    case "tesla-valve":
      return <TeslaValvePreview />;
  }
  // A catalog id without a case narrows to a non-never type here and fails to compile.
  const unreachable: never = props.sceneId;
  return unreachable;
}
