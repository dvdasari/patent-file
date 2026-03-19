"use client";

export default function ProjectsError({
  error,
  reset,
}: {
  error: Error;
  reset: () => void;
}) {
  return (
    <div className="mx-auto max-w-3xl px-4 py-20 text-center">
      <h2 className="text-lg font-semibold text-zinc-100">Failed to load projects</h2>
      <p className="mt-2 text-sm text-zinc-400">{error.message}</p>
      <button
        onClick={reset}
        className="mt-4 rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-900 hover:bg-zinc-200"
      >
        Retry
      </button>
    </div>
  );
}
