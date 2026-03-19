"use client";

import type { WizardData } from "@/hooks/use-interview-wizard";

interface Props {
  data: WizardData;
  updateField: (field: keyof WizardData, value: string) => void;
}

export function StepDescription({ data, updateField }: Props) {
  return (
    <div className="space-y-5">
      <h2 className="text-lg font-semibold text-zinc-100">Invention Description</h2>

      <div className="space-y-2">
        <label className="text-sm font-medium text-zinc-300">Describe your invention in plain language</label>
        <textarea value={data.description} onChange={(e) => updateField("description", e.target.value)} rows={5}
          className="w-full rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 focus:border-zinc-500 focus:outline-none"
          placeholder="Explain what your invention is and how it works..." />
      </div>

      <div className="space-y-2">
        <label className="text-sm font-medium text-zinc-300">What are the key components/elements?</label>
        <textarea value={data.key_components} onChange={(e) => updateField("key_components", e.target.value)} rows={4}
          className="w-full rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 focus:border-zinc-500 focus:outline-none"
          placeholder="List the main parts, modules, or elements..." />
      </div>

      <div className="space-y-2">
        <label className="text-sm font-medium text-zinc-300">How do the components work together? (Step-by-step)</label>
        <textarea value={data.process_steps} onChange={(e) => updateField("process_steps", e.target.value)} rows={5}
          className="w-full rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 focus:border-zinc-500 focus:outline-none"
          placeholder="Describe the process or method step by step..." />
      </div>
    </div>
  );
}
