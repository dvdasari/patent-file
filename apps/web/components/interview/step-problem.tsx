"use client";

import type { WizardData } from "@/hooks/use-interview-wizard";

interface Props {
  data: WizardData;
  updateField: (field: keyof WizardData, value: string) => void;
}

export function StepProblem({ data, updateField }: Props) {
  return (
    <div className="space-y-5">
      <h2 className="text-lg font-semibold text-zinc-100">Problem & Prior Art</h2>

      <div className="space-y-2">
        <label className="text-sm font-medium text-zinc-300">What problem does this invention solve?</label>
        <textarea value={data.problem} onChange={(e) => updateField("problem", e.target.value)} rows={4}
          className="w-full rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 focus:border-zinc-500 focus:outline-none" />
      </div>

      <div className="space-y-2">
        <label className="text-sm font-medium text-zinc-300">How is this problem currently solved?</label>
        <textarea value={data.current_solutions} onChange={(e) => updateField("current_solutions", e.target.value)} rows={4}
          className="w-full rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 focus:border-zinc-500 focus:outline-none" />
      </div>

      <div className="space-y-2">
        <label className="text-sm font-medium text-zinc-300">What are the limitations of existing solutions?</label>
        <textarea value={data.limitations} onChange={(e) => updateField("limitations", e.target.value)} rows={4}
          className="w-full rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 focus:border-zinc-500 focus:outline-none" />
      </div>
    </div>
  );
}
