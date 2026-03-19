"use client";

import { useSearchParams } from "next/navigation";
import { AuthGuard } from "@/components/layout/auth-guard";
import { SubscriptionGuard } from "@/components/layout/subscription-guard";
import { Navbar } from "@/components/layout/navbar";
import { InterviewWizard } from "@/components/interview/interview-wizard";

function NewProjectContent() {
  const searchParams = useSearchParams();
  const projectId = searchParams.get("id");

  if (!projectId) {
    return (
      <div className="flex items-center justify-center py-20">
        <p className="text-sm text-zinc-400">No project ID provided. Go back to projects.</p>
      </div>
    );
  }

  return <InterviewWizard projectId={projectId} />;
}

export default function NewProjectPage() {
  return (
    <AuthGuard>
      <SubscriptionGuard>
        <Navbar />
        <NewProjectContent />
      </SubscriptionGuard>
    </AuthGuard>
  );
}
