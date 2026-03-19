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

export function InterviewWizard({ projectId }: InterviewWizardProps) {
  const wizard = useInterviewWizard(projectId);
  const { step, data, saving, updateField, goNext, goBack, goToStep } = wizard;

  const stepProps = {
    data,
    updateField,
    projectId,
  };

  return (
    <div className="mx-auto max-w-2xl px-4 py-8">
      {/* Progress */}
      <div className="mb-8">
        <div className="flex items-center justify-between text-xs text-zinc-500 mb-2">
          <span>Step {step} of {TOTAL_STEPS}</span>
          <span>{STEP_NAMES[step - 1]}</span>
        </div>
        <div className="h-1 rounded-full bg-zinc-800">
          <div
            className="h-1 rounded-full bg-zinc-400 transition-all"
            style={{ width: `${(step / TOTAL_STEPS) * 100}%` }}
          />
        </div>
      </div>

      {/* Step Content */}
      <div className="mb-8">
        {step === 1 && <StepBasics {...stepProps} />}
        {step === 2 && <StepApplicant {...stepProps} />}
        {step === 3 && <StepProblem {...stepProps} />}
        {step === 4 && <StepDescription {...stepProps} />}
        {step === 5 && <StepNovelty {...stepProps} />}
        {step === 6 && <StepFigures {...stepProps} />}
        {step === 7 && <StepReview data={data} goToStep={goToStep} projectId={projectId} />}
      </div>

      {/* Navigation */}
      <div className="flex items-center justify-between">
        <button
          onClick={goBack}
          disabled={step === 1}
          className="rounded-md border border-zinc-700 px-4 py-2 text-sm text-zinc-300 hover:bg-zinc-800 disabled:opacity-30"
        >
          Back
        </button>
        {step < TOTAL_STEPS ? (
          <button
            onClick={goNext}
            disabled={saving}
            className="rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-900 hover:bg-zinc-200 disabled:opacity-50"
          >
            {saving ? "Saving..." : "Next"}
          </button>
        ) : null}
      </div>
    </div>
  );
}

const STEP_NAMES = [
  "Basics",
  "Applicant Details",
  "Problem & Prior Art",
  "Invention Description",
  "Novelty & Advantages",
  "Figures",
  "Review & Generate",
];
