"use client";

import type { WizardData } from "@/hooks/use-interview-wizard";

interface Props {
  data: WizardData;
  updateField: (field: keyof WizardData, value: string) => void;
}

const FIELDS = ["Mechanical", "Software", "Chemical", "Electrical", "Biotech", "Other"];

export function StepBasics({ data, updateField }: Props) {
  return (
    <div className="space-y-5">
      <h2 className="text-lg font-semibold text-zinc-100">Basics</h2>

      <div className="space-y-2">
        <label className="text-sm font-medium text-zinc-300">Invention Title</label>
        <input
          value={data.title}
          onChange={(e) => updateField("title", e.target.value)}
          className="w-full rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 focus:border-zinc-500 focus:outline-none"
          placeholder="e.g., Automated Irrigation Controller"
        />
      </div>

      <div className="space-y-2">
        <label className="text-sm font-medium text-zinc-300">Patent Type</label>
        <div className="flex gap-4">
          {["complete", "provisional"].map((t) => (
            <label key={t} className="flex items-center gap-2 text-sm text-zinc-300">
              <input
                type="radio"
                name="patent_type"
                value={t}
                checked={data.patent_type === t}
                onChange={() => updateField("patent_type", t)}
                className="accent-zinc-400"
              />
              {t === "complete" ? "Complete Specification" : "Provisional Specification"}
            </label>
          ))}
        </div>
      </div>

      <div className="space-y-2">
        <label className="text-sm font-medium text-zinc-300">Technical Field</label>
        <select
          value={data.technical_field}
          onChange={(e) => updateField("technical_field", e.target.value)}
          className="w-full rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 focus:border-zinc-500 focus:outline-none"
        >
          <option value="">Select a field...</option>
          {FIELDS.map((f) => (
            <option key={f} value={f}>{f}</option>
          ))}
        </select>
      </div>
    </div>
  );
}
