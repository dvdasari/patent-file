"use client";

import { useCallback, useEffect, useRef } from "react";

export function useAutoSave(
  value: string,
  onSave: (value: string) => Promise<void>,
  delay = 2000
) {
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);
  const lastSavedRef = useRef(value);

  const save = useCallback(async () => {
    if (value !== lastSavedRef.current) {
      await onSave(value);
      lastSavedRef.current = value;
    }
  }, [value, onSave]);

  // Debounced auto-save
  useEffect(() => {
    if (value === lastSavedRef.current) return;

    if (timeoutRef.current) clearTimeout(timeoutRef.current);
    timeoutRef.current = setTimeout(save, delay);

    return () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    };
  }, [value, save, delay]);

  // Save on blur
  const onBlur = useCallback(() => {
    save();
  }, [save]);

  return { onBlur };
}
