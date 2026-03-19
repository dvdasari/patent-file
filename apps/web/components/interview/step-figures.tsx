"use client";

import type { WizardData } from "@/hooks/use-interview-wizard";

interface Props {
  data: WizardData;
  updateField: (field: keyof WizardData, value: string) => void;
  projectId: string;
}

export function StepFigures({ projectId }: Props) {
  // Figures upload will be wired to the figures API
  // For now, show a placeholder that the feature is available
  return (
    <div className="space-y-5">
      <h2 className="text-lg font-semibold text-zinc-100">Figures <span className="text-zinc-500 text-sm font-normal">(optional)</span></h2>
      <p className="text-sm text-zinc-400">
        Upload sketches or diagrams of your invention. These will be referenced in the drawings description section.
      </p>

      <div className="rounded-lg border border-dashed border-zinc-700 p-8 text-center">
        <p className="text-sm text-zinc-500">
          Drag and drop images here, or click to browse
        </p>
        <input
          type="file"
          accept="image/*"
          multiple
          className="mt-3 text-sm text-zinc-400 file:mr-4 file:rounded-md file:border-0 file:bg-zinc-800 file:px-3 file:py-1.5 file:text-sm file:text-zinc-300 hover:file:bg-zinc-700"
          onChange={() => {
            // TODO: wire to figures API
          }}
        />
        <p className="mt-2 text-xs text-zinc-600">PNG, JPG up to 10MB each</p>
      </div>

      <p className="text-xs text-zinc-600">
        You can skip this step and add figures later from the editor.
      </p>
    </div>
  );
}
