"use client";

import { useCallback, useEffect, useState } from "react";
import { api } from "@/lib/api-client";

export const TOTAL_STEPS = 7;

export interface WizardData {
  // Step 1: Basics
  title: string;
  patent_type: string;
  technical_field: string;
  // Step 2: Applicant
  applicant_name: string;
  applicant_address: string;
  applicant_nationality: string;
  inventor_name: string;
  inventor_address: string;
  inventor_nationality: string;
  agent_name: string;
  agent_registration_no: string;
  assignee_name: string;
  priority_date: string;
  priority_country: string;
  priority_application_no: string;
  // Step 3: Problem
  problem: string;
  current_solutions: string;
  limitations: string;
  // Step 4: Description
  description: string;
  key_components: string;
  process_steps: string;
  // Step 5: Novelty
  novelty: string;
  advantages: string;
  alternative_embodiments: string;
}

const DEFAULT_DATA: WizardData = {
  title: "", patent_type: "complete", technical_field: "",
  applicant_name: "", applicant_address: "", applicant_nationality: "Indian",
  inventor_name: "", inventor_address: "", inventor_nationality: "Indian",
  agent_name: "", agent_registration_no: "", assignee_name: "",
  priority_date: "", priority_country: "", priority_application_no: "",
  problem: "", current_solutions: "", limitations: "",
  description: "", key_components: "", process_steps: "",
  novelty: "", advantages: "", alternative_embodiments: "",
};

export function useInterviewWizard(projectId: string | null) {
  const [step, setStep] = useState(1);
  const [data, setData] = useState<WizardData>(DEFAULT_DATA);
  const [saving, setSaving] = useState(false);

  // Load existing data on mount
  useEffect(() => {
    if (!projectId) return;
    Promise.all([
      api.getInterview(projectId).catch(() => []),
      api.getApplicant(projectId).catch(() => null),
      api.getProject(projectId).catch(() => null),
    ]).then(([responses, applicant, project]) => {
      const d = { ...DEFAULT_DATA };
      // Map interview responses
      for (const r of responses as Array<{ question_key: string; response_text: string | null }>) {
        if (r.question_key in d) {
          (d as Record<string, string>)[r.question_key] = r.response_text || "";
        }
      }
      // Map applicant
      if (applicant) {
        const a = applicant as Record<string, string>;
        for (const key of Object.keys(a)) {
          if (key in d) (d as Record<string, string>)[key] = a[key] || "";
        }
      }
      // Map project basics
      if (project) {
        const p = (project as { project: Record<string, string> }).project;
        if (p.title) d.title = p.title;
        if (p.patent_type) d.patent_type = p.patent_type;
      }
      setData(d);
    });
  }, [projectId]);

  const updateField = useCallback((field: keyof WizardData, value: string) => {
    setData((prev) => ({ ...prev, [field]: value }));
  }, []);

  const saveCurrentStep = useCallback(async () => {
    if (!projectId) return;
    setSaving(true);
    try {
      if (step === 1) {
        await api.updateProject(projectId, {
          title: data.title,
          patent_type: data.patent_type,
        });
      } else if (step === 2) {
        await api.upsertApplicant(projectId, {
          applicant_name: data.applicant_name,
          applicant_address: data.applicant_address,
          applicant_nationality: data.applicant_nationality,
          inventor_name: data.inventor_name,
          inventor_address: data.inventor_address,
          inventor_nationality: data.inventor_nationality,
          agent_name: data.agent_name || null,
          agent_registration_no: data.agent_registration_no || null,
          assignee_name: data.assignee_name || null,
          priority_date: data.priority_date || null,
          priority_country: data.priority_country || null,
          priority_application_no: data.priority_application_no || null,
        });
      } else if (step >= 3 && step <= 5) {
        const stepFields: Record<number, string[]> = {
          3: ["problem", "current_solutions", "limitations"],
          4: ["description", "key_components", "process_steps"],
          5: ["novelty", "advantages", "alternative_embodiments"],
        };
        const fields = stepFields[step] || [];
        const responses = fields.map((key, i) => ({
          step_number: step,
          question_key: key,
          question_text: key.replace(/_/g, " "),
          response_text: (data as unknown as Record<string, string>)[key] || null,
        }));
        // Also save step 1 fields as interview responses for the AI pipeline
        if (step === 3) {
          responses.push({
            step_number: 1,
            question_key: "technical_field",
            question_text: "technical field",
            response_text: data.technical_field || null,
          });
        }
        await api.saveInterview(projectId, responses);
      }
    } finally {
      setSaving(false);
    }
  }, [projectId, step, data]);

  const goNext = useCallback(async () => {
    await saveCurrentStep();
    setStep((s) => Math.min(s + 1, TOTAL_STEPS));
  }, [saveCurrentStep]);

  const goBack = useCallback(() => {
    setStep((s) => Math.max(s - 1, 1));
  }, []);

  const goToStep = useCallback((s: number) => {
    setStep(Math.max(1, Math.min(s, TOTAL_STEPS)));
  }, []);

  return {
    step, data, saving,
    updateField, goNext, goBack, goToStep, saveCurrentStep,
  };
}
