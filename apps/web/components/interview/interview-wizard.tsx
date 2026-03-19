"use client";

import { useInterviewWizard, TOTAL_STEPS, type WizardData } from "@/hooks/use-interview-wizard";
import { StepBasics } from "./step-basics";
import { StepApplicant } from "./step-applicant";
import { StepProblem } from "./step-problem";
import { StepDescription } from "./step-description";
import { StepNovelty } from "./step-novelty";
import { StepFigures } from "./step-figures";
import { StepReview } from "./step-review";

interface InterviewWizardProps {
  projectId: string;
}

const STEP_NAMES = [
  "Basics",
  "Applicant",
  "Problem & Prior Art",
  "Description",
  "Novelty",
  "Figures",
  "Review",
];

export function InterviewWizard({ projectId }: InterviewWizardProps) {
  const wizard = useInterviewWizard(projectId);
  const { step, data, saving, updateField, goNext, goBack, goToStep } = wizard;

  const stepProps = { data, updateField, projectId };

  return (
    <div className="mx-auto max-w-2xl px-6 py-10 animate-fade-in">
      {/* Step indicator */}
      <div className="mb-10">
        <div className="flex items-center justify-between mb-3">
          {STEP_NAMES.map((name, i) => {
            const stepNum = i + 1;
            const isActive = step === stepNum;
            const isComplete = step > stepNum;
            return (
              <button
                key={name}
                onClick={() => goToStep(stepNum)}
                className="flex items-center gap-1.5 transition-all duration-200"
              >
                <div
                  className="w-6 h-6 rounded-full flex items-center justify-center text-xs font-mono transition-all duration-300"
                  style={{
                    background: isActive ? "var(--accent)" : isComplete ? "rgba(200, 169, 110, 0.15)" : "var(--surface-raised)",
                    color: isActive ? "var(--background)" : isComplete ? "var(--accent)" : "var(--muted)",
                    border: isActive ? "none" : `1px solid ${isComplete ? "var(--accent-dim)" : "var(--border)"}`,
                  }}
                >
                  {isComplete ? "✓" : stepNum}
                </div>
                <span className="hidden sm:inline text-xs" style={{ color: isActive ? "var(--foreground)" : "var(--muted)" }}>
                  {name}
                </span>
              </button>
            );
          })}
        </div>
        <div className="h-px w-full" style={{ background: "var(--border)" }}>
          <div
            className="h-px transition-all duration-500 ease-out"
            style={{ width: `${((step - 1) / (TOTAL_STEPS - 1)) * 100}%`, background: "var(--accent)" }}
          />
        </div>
      </div>

      {/* Step Content */}
      <div className="mb-10 min-h-[400px]">
        {step === 1 && <StepBasics {...stepProps} />}
        {step === 2 && <StepApplicant {...stepProps} />}
        {step === 3 && <StepProblem {...stepProps} />}
        {step === 4 && <StepDescription {...stepProps} />}
        {step === 5 && <StepNovelty {...stepProps} />}
        {step === 6 && <StepFigures {...stepProps} />}
        {step === 7 && <StepReview data={data} goToStep={goToStep} projectId={projectId} />}
      </div>

      {/* Navigation */}
      <div className="flex items-center justify-between border-t pt-6" style={{ borderColor: "var(--border)" }}>
        <button
          onClick={goBack}
          disabled={step === 1}
          className="rounded border px-4 py-2 text-xs font-medium transition-all disabled:opacity-20"
          style={{ borderColor: "var(--border)", color: "var(--muted)" }}
        >
          ← Back
        </button>

        <span className="text-xs font-mono" style={{ color: "var(--muted)" }}>
          {step} / {TOTAL_STEPS}
        </span>

        {step < TOTAL_STEPS && (
          <button
            onClick={goNext}
            disabled={saving}
            className="rounded px-5 py-2 text-xs font-medium transition-all disabled:opacity-40"
            style={{ background: "var(--accent)", color: "var(--background)" }}
          >
            {saving ? "Saving..." : "Next →"}
          </button>
        )}
        {step === TOTAL_STEPS && <div />}
      </div>
    </div>
  );
}
