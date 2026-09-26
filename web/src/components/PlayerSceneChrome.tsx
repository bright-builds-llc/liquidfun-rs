import { Show } from "solid-js";

import type { SceneRecord } from "../catalog/scenes";
import { sceneControlsIdentity } from "../player/runtime";
import { sceneControlsForSurface } from "./scene-controls";
import { SceneControls } from "./SceneControls";
import { SceneCredits } from "./SceneCredits";

export type PlayerSceneChromeProps = {
  readonly scene: SceneRecord;
  readonly resetGeneration: number;
  readonly disabled: boolean;
  readonly maybeValues: Readonly<Record<string, string>>;
  readonly onApplyControl: (name: string, value: string) => void;
  readonly onApplyAction: (name: string) => void;
};

/** Keyed scene controls plus credits that stay mounted across Reset remounts. */
export function PlayerSceneChrome(props: PlayerSceneChromeProps) {
  return (
    <>
      <Show
        when={sceneControlsIdentity(props.scene.id, props.resetGeneration)}
        keyed
      >
        {(_controlsIdentity) => (
          <SceneControls
            controls={sceneControlsForSurface(props.scene.controls, "panel")}
            disabled={props.disabled}
            maybeValues={props.maybeValues}
            onApplyControl={props.onApplyControl}
            onApplyAction={props.onApplyAction}
          />
        )}
      </Show>
      <SceneCredits
        implementationPath={props.scene.credits.implementationPath}
        inspiration={props.scene.credits.inspiration}
      />
    </>
  );
}
