"use client";

import type { WizardData } from "@/hooks/use-interview-wizard";

interface Props {
  data: WizardData;
  updateField: (field: keyof WizardData, value: string) => void;
}

function Field({ label, field, data, updateField, type = "text", required = true, placeholder = "" }: {
  label: string; field: keyof WizardData; data: WizardData;
  updateField: (f: keyof WizardData, v: string) => void;
  type?: string; required?: boolean; placeholder?: string;
}) {
  const isTextarea = type === "textarea";
  const Component = isTextarea ? "textarea" : "input";
  return (
    <div className="space-y-1">
      <label className="text-sm font-medium text-zinc-300">
        {label} {!required && <span className="text-zinc-500">(optional)</span>}
      </label>
      <Component
        value={data[field]}
        onChange={(e) => updateField(field, e.target.value)}
        className="w-full rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 focus:border-zinc-500 focus:outline-none"
        placeholder={placeholder}
        {...(isTextarea ? { rows: 3 } : {})}
      />
    </div>
  );
}

export function StepApplicant({ data, updateField }: Props) {
  return (
    <div className="space-y-5">
      <h2 className="text-lg font-semibold text-zinc-100">Applicant & Filing Details</h2>

      <div className="grid grid-cols-2 gap-4">
        <Field label="Applicant Name" field="applicant_name" data={data} updateField={updateField} />
        <Field label="Applicant Nationality" field="applicant_nationality" data={data} updateField={updateField} />
      </div>
      <Field label="Applicant Address" field="applicant_address" data={data} updateField={updateField} type="textarea" />

      <div className="grid grid-cols-2 gap-4">
        <Field label="Inventor Name" field="inventor_name" data={data} updateField={updateField} />
        <Field label="Inventor Nationality" field="inventor_nationality" data={data} updateField={updateField} />
      </div>
      <Field label="Inventor Address" field="inventor_address" data={data} updateField={updateField} type="textarea" />

      <hr className="border-zinc-800" />

      <div className="grid grid-cols-2 gap-4">
        <Field label="Patent Agent Name" field="agent_name" data={data} updateField={updateField} required={false} />
        <Field label="Agent Reg. No." field="agent_registration_no" data={data} updateField={updateField} required={false} />
      </div>
      <Field label="Assignee Name" field="assignee_name" data={data} updateField={updateField} required={false} placeholder="If different from applicant" />

      <hr className="border-zinc-800" />
      <p className="text-xs text-zinc-500">Priority Claim (optional)</p>
      <div className="grid grid-cols-3 gap-4">
        <Field label="Priority Date" field="priority_date" data={data} updateField={updateField} type="date" required={false} />
        <Field label="Country" field="priority_country" data={data} updateField={updateField} required={false} />
        <Field label="Application No." field="priority_application_no" data={data} updateField={updateField} required={false} />
      </div>
    </div>
  );
}
