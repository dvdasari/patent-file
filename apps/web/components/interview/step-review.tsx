"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import type { WizardData } from "@/hooks/use-interview-wizard";

interface Props {
  data: WizardData;
  goToStep: (step: number) => void;
  projectId: string;
}

const SECTIONS = [
  { step: 1, title: "Basics", fields: [
    { key: "title", label: "Title" },
    { key: "patent_type", label: "Type" },
    { key: "technical_field", label: "Field" },
  ]},
  { step: 2, title: "Applicant", fields: [
    { key: "applicant_name", label: "Applicant" },
    { key: "inventor_name", label: "Inventor" },
  ]},
  { step: 3, title: "Problem & Prior Art", fields: [
    { key: "problem", label: "Problem" },
    { key: "current_solutions", label: "Current Solutions" },
    { key: "limitations", label: "Limitations" },
  ]},
  { step: 4, title: "Description", fields: [
    { key: "description", label: "Description" },
    { key: "key_components", label: "Components" },
    { key: "process_steps", label: "Process" },
  ]},
  { step: 5, title: "Novelty", fields: [
    { key: "novelty", label: "Novelty" },
    { key: "advantages", label: "Advantages" },
  ]},
];

export function StepReview({ data, goToStep, projectId }: Props) {
  const router = useRouter();
  const [generating, setGenerating] = useState(false);

  // Check required fields
  const required = ["title", "technical_field", "problem", "current_solutions", "limitations", "description", "key_components", "process_steps", "novelty", "advantages"];
  const missing = required.filter((k) => !(data as unknown as Record<string, string>)[k]?.trim());
  const canGenerate = missing.length === 0;

  async function handleGenerate() {
    setGenerating(true);
    // Navigate to editor with generate flag — editor will start SSE generation
    router.push(`/projects/${projectId}?generate=true`);
  }

  return (
    <div className="space-y-6">
      <h2 className="text-lg font-semibold text-zinc-100">Review & Generate</h2>

      {SECTIONS.map((section) => (
        <div key={section.step} className="rounded-lg border border-zinc-800 p-4">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-sm font-medium text-zinc-200">{section.title}</h3>
            <button
              onClick={() => goToStep(section.step)}
              className="text-xs text-zinc-500 hover:text-zinc-300"
            >
              Edit
            </button>
          </div>
          <div className="space-y-1">
            {section.fields.map((f) => {
              const val = (data as unknown as Record<string, string>)[f.key];
              return (
                <div key={f.key} className="text-sm">
                  <span className="text-zinc-500">{f.label}: </span>
                  <span className={val ? "text-zinc-300" : "text-red-400"}>
                    {val ? (val.length > 100 ? val.slice(0, 100) + "..." : val) : "Missing"}
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      ))}

      {!canGenerate && (
        <p className="text-sm text-red-400">
          Please fill in all required fields before generating.
        </p>
      )}

      <button
        onClick={handleGenerate}
        disabled={!canGenerate || generating}
        className="w-full rounded-md bg-zinc-100 px-4 py-3 text-sm font-semibold text-zinc-900 hover:bg-zinc-200 disabled:opacity-50"
      >
        {generating ? "Starting generation..." : "Generate Patent Draft"}
      </button>

      <p className="text-xs text-zinc-500 text-center">
        Estimated generation time: ~60-90 seconds
      </p>
    </div>
  );
}
