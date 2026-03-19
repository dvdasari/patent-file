"use client";

import { useCallback, useEffect, useState } from "react";
import { useParams, useSearchParams } from "next/navigation";
import Link from "next/link";
import { api } from "@/lib/api-client";
import { AuthGuard } from "@/components/layout/auth-guard";
import { SubscriptionGuard } from "@/components/layout/subscription-guard";
import { Navbar } from "@/components/layout/navbar";
import { SectionList } from "@/components/editor/section-list";
import { GenerationStream } from "@/components/editor/generation-stream";

interface Section {
  id: string;
  section_type: string;
  content: string;
  ai_generated: boolean;
  edit_count: number;
}

interface Project {
  id: string;
  title: string;
  status: string;
}

function EditorContent() {
  const params = useParams();
  const searchParams = useSearchParams();
  const projectId = params.id as string;
  const shouldGenerate = searchParams.get("generate") === "true";

  const [project, setProject] = useState<Project | null>(null);
  const [sections, setSections] = useState<Section[]>([]);
  const [loading, setLoading] = useState(true);
  const [generating, setGenerating] = useState(false);

  const loadProject = useCallback(async () => {
    try {
      const data = (await api.getProject(projectId)) as {
        project: Project;
        sections: Section[];
      };
      setProject(data.project);
      setSections(data.sections);
      // Start generation if redirected from wizard with ?generate=true, or if status is already generating
      setGenerating(data.project.status === "generating" || shouldGenerate);
    } catch {
      // handle error
    } finally {
      setLoading(false);
    }
  }, [projectId, shouldGenerate]);

  useEffect(() => {
    loadProject();
  }, [loadProject]);

  function handleSectionUpdate(sectionType: string, newContent: string) {
    setSections((prev) =>
      prev.map((s) =>
        s.section_type === sectionType
          ? { ...s, content: newContent, ai_generated: false, edit_count: s.edit_count + 1 }
          : s
      )
    );
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <p className="text-sm text-zinc-400">Loading project...</p>
      </div>
    );
  }

  if (!project) {
    return (
      <div className="flex items-center justify-center py-20">
        <p className="text-sm text-zinc-400">Project not found.</p>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl px-4 py-8">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-lg font-semibold text-zinc-100">{project.title}</h1>
          <p className="text-xs text-zinc-500 mt-1">
            Status: {project.status} &middot; {sections.length} sections
          </p>
        </div>
        {sections.length > 0 && project.status === "review" && (
          <Link
            href={`/projects/${projectId}/export`}
            className="rounded-md bg-zinc-100 px-3 py-1.5 text-sm font-medium text-zinc-900 hover:bg-zinc-200"
          >
            Export
          </Link>
        )}
      </div>

      {generating ? (
        <GenerationStream
          projectId={projectId}
          onComplete={() => {
            setGenerating(false);
            loadProject();
          }}
        />
      ) : sections.length > 0 ? (
        <SectionList
          projectId={projectId}
          sections={sections}
          onSectionUpdate={handleSectionUpdate}
        />
      ) : (
        <div className="rounded-lg border border-dashed border-zinc-700 p-12 text-center">
          <p className="text-sm text-zinc-400">
            No sections generated yet. Complete the interview wizard first.
          </p>
          <Link
            href={`/projects/new?id=${projectId}`}
            className="mt-4 inline-block rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-900 hover:bg-zinc-200"
          >
            Go to Interview
          </Link>
        </div>
      )}
    </div>
  );
}

export default function EditorPage() {
  return (
    <AuthGuard>
      <SubscriptionGuard>
        <Navbar />
        <EditorContent />
      </SubscriptionGuard>
    </AuthGuard>
  );
}
