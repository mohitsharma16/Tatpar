import { useState, useCallback, useRef, useEffect } from "react";

// ============================================================
// Tatpar — useResizable Hook
// Provides drag-to-resize behavior for the terminal pane
// ============================================================

interface UseResizableOptions {
  /** Initial default height in pixels (default: 200) */
  initialHeight?: number;
  /** Minimum terminal height in pixels (default: 32) */
  minHeight?: number;
  /** Minimum editor height to keep in view (default: 120) */
  minEditorHeight?: number;
  /** LocalStorage key for persistence */
  storageKey?: string;
  /** Default height to restore on double-click */
  defaultHeight?: number;
}

export function useResizable({
  initialHeight = 200,
  minHeight = 32,
  minEditorHeight = 120,
  storageKey = "tatpar:terminal-height",
  defaultHeight = 200,
}: UseResizableOptions = {}) {
  const [height, setHeight] = useState<number>(() => {
    try {
      const saved = localStorage.getItem(storageKey);
      if (saved) {
        const val = parseInt(saved, 10);
        if (!isNaN(val) && val >= minHeight) return val;
      }
    } catch {}
    return initialHeight;
  });

  const [isResizing, setIsResizing] = useState(false);
  const startDragRef = useRef<{ startY: number; startHeight: number } | null>(null);

  const startResizing = useCallback(
    (e: React.PointerEvent) => {
      e.preventDefault();
      startDragRef.current = {
        startY: e.clientY,
        startHeight: height,
      };
      setIsResizing(true);
    },
    [height]
  );

  const toggleOrReset = useCallback(() => {
    setHeight((prev) => {
      const next = prev <= 40 ? defaultHeight : 32;
      try {
        localStorage.setItem(storageKey, String(next));
      } catch {}
      return next;
    });
  }, [defaultHeight, storageKey]);

  useEffect(() => {
    if (!isResizing) return;

    const handlePointerMove = (e: PointerEvent) => {
      if (!startDragRef.current) return;
      const { startY, startHeight } = startDragRef.current;
      const delta = startY - e.clientY;
      const maxHeight = Math.max(window.innerHeight - minEditorHeight, minHeight);
      const newHeight = Math.min(Math.max(startHeight + delta, minHeight), maxHeight);
      setHeight(newHeight);
    };

    const handlePointerUp = (e: PointerEvent) => {
      if (startDragRef.current) {
        const { startY, startHeight } = startDragRef.current;
        const delta = startY - e.clientY;
        const maxHeight = Math.max(window.innerHeight - minEditorHeight, minHeight);
        const finalHeight = Math.min(Math.max(startHeight + delta, minHeight), maxHeight);
        setHeight(finalHeight);
        try {
          localStorage.setItem(storageKey, String(finalHeight));
        } catch {}
      }
      setIsResizing(false);
      startDragRef.current = null;
    };

    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", handlePointerUp);

    return () => {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
    };
  }, [isResizing, minHeight, minEditorHeight, storageKey]);

  return {
    height,
    isResizing,
    startResizing,
    toggleOrReset,
  };
}
