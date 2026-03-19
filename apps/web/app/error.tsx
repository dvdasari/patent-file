"use client";

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-950">
      <div className="max-w-md space-y-4 text-center">
        <h2 className="text-lg font-semibold text-zinc-100">Something went wrong</h2>
        <p className="text-sm text-zinc-400">{error.message || "An unexpected error occurred."}</p>
        <button
          onClick={reset}
          className="rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-900 hover:bg-zinc-200"
        >
          Try again
        </button>
      </div>
    </div>
  );
}
