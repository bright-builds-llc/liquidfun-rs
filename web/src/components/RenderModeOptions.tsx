import { RENDER_MODE_GROUPS, renderModeOptionView } from "../render/mode";

/**
 * Optgroups for a Particles picker. Shared by the scene and puzzle pickers so
 * they never disagree. On an incapable device Water reads "Water
 * (unavailable)" and is disabled; a stored `water` value stays selected
 * (D-05, UI-D49). Imports no WebGL or shader module.
 */
export function RenderModeOptions(props: { readonly waterAvailable: boolean }) {
  return (
    <>
      {RENDER_MODE_GROUPS.map((group) => (
        <optgroup label={group.label}>
          {group.options.map((option) => {
            const view = () => renderModeOptionView(option, props.waterAvailable);
            return (
              <option value={option.value} disabled={view().disabled}>
                {view().label}
              </option>
            );
          })}
        </optgroup>
      ))}
    </>
  );
}
