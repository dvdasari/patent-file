"use client";

import type { WizardData } from "@/hooks/use-interview-wizard";

interface Props {
  data: WizardData;
  updateField: (field: keyof WizardData, value: string) => void;
}

export function StepNovelty({ data, updateField }: Props) {
  return (
    <div className="space-y-5">
      <h2 className="text-lg font-semibold text-zinc-100">Novelty & Advantages</h2>

      <div className="space-y-2">
        <label className="text-sm font-medium text-zinc-300">What is new/novel about your invention?</label>
        <textarea value={data.novelty} onChange={(e) => updateField("novelty", e.target.value)} rows={4}
          className="w-full rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 focus:border-zinc-500 focus:outline-none"
          placeholder="How does this differ from existing solutions?" />
      </div>

      <div className="space-y-2">
        <label className="text-sm font-medium text-zinc-300">What are the advantages/benefits?</label>
        <textarea value={data.advantages} onChange={(e) => updateField("advantages", e.target.value)} rows={4}
          className="w-full rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 focus:border-zinc-500 focus:outline-none" />
      </div>

      <div className="space-y-2">
        <label className="text-sm font-medium text-zinc-300">
          Alternative embodiments <span className="text-zinc-500">(optional)</span>
        </label>
        <textarea value={data.alternative_embodiments} onChange={(e) => updateField("alternative_embodiments", e.target.value)} rows={3}
          className="w-full rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 focus:border-zinc-500 focus:outline-none"
          placeholder="Are there other ways to implement this invention?" />
      </div>
    </div>
  );
}
