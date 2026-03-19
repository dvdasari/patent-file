"use client";

import { SectionCard } from "./section-card";

interface Section {
  id: string;
  section_type: string;
  content: string;
  ai_generated: boolean;
  edit_count: number;
}

interface SectionListProps {
  projectId: string;
  sections: Section[];
  onSectionUpdate: (sectionType: string, newContent: string) => void;
}

const SECTION_ORDER = [
  "title",
  "field_of_invention",
  "background",
  "summary",
  "detailed_description",
  "claims",
  "abstract",
  "drawings_description",
];

export function SectionList({
  projectId,
  sections,
  onSectionUpdate,
}: SectionListProps) {
  const ordered = SECTION_ORDER.map((type) =>
    sections.find((s) => s.section_type === type)
  ).filter(Boolean) as Section[];

  return (
    <div className="space-y-4">
      {ordered.map((section) => (
        <SectionCard
          key={section.id}
          projectId={projectId}
          sectionType={section.section_type}
          content={section.content}
          aiGenerated={section.ai_generated}
          editCount={section.edit_count}
          onContentUpdate={(newContent) =>
            onSectionUpdate(section.section_type, newContent)
          }
        />
      ))}
    </div>
  );
}
