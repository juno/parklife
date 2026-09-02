import { useCallback, useEffect, useRef, useState } from "react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open, save } from "@tauri-apps/plugin-dialog";
import "./App.css";

const IMAGE_EXTS = ["png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff"];

type Pt = { x: number; y: number };
type Mode = "rect" | "lasso";

const isImagePath = (p: string) =>
  IMAGE_EXTS.includes(p.split(".").pop()?.toLowerCase() ?? "");

function rectPoly(a: Pt, b: Pt): Pt[] {
  return [
    { x: a.x, y: a.y },
    { x: b.x, y: a.y },
    { x: b.x, y: b.y },
    { x: a.x, y: b.y },
  ];
}

function App() {
  const [history, setHistory] = useState<string[]>([]);
  const [histIndex, setHistIndex] = useState(0);
  const [mode, setMode] = useState<Mode>("rect");
  const [draft, setDraft] = useState<Pt[] | null>(null);
  const [sigma, setSigma] = useState(8);
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState("");
  const [dragOver, setDragOver] = useState(false);
  const [nav, setNav] = useState(0);

  const imgRef = useRef<HTMLImageElement>(null);
  const drawing = useRef(false);
  const anchor = useRef<Pt | null>(null);

  const workingPath = history[histIndex] ?? null;
  const imgSrc = workingPath ? `${convertFileSrc(workingPath)}?v=${nav}` : "";
  const canUndo = histIndex > 0;
  const canRedo = histIndex < history.length - 1;
  const canApply = !!workingPath && !!draft && draft.length >= 3 && !busy;

  const startSession = useCallback(async (path: string) => {
    setBusy(true);
    setStatus("Loading…");
    try {
      const wp = await invoke<string>("start_session", { src: path });
      setHistory([wp]);
      setHistIndex(0);
      setDraft(null);
      setStatus("");
      setNav((n) => n + 1);
    } catch (err) {
      setStatus(`Error: ${err}`);
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    getCurrentWebview()
      .onDragDropEvent((event) => {
        const p = event.payload;
        if (p.type === "enter" || p.type === "over") {
          setDragOver(true);
        } else if (p.type === "leave") {
          setDragOver(false);
        } else if (p.type === "drop") {
          setDragOver(false);
          const file = p.paths.find(isImagePath);
          if (file) startSession(file);
        }
      })
      .then((f) => {
        unlisten = f;
      });
    return () => unlisten?.();
  }, [startSession]);

  async function openViaDialog() {
    const picked = await open({
      multiple: false,
      filters: [{ name: "Images", extensions: IMAGE_EXTS }],
    });
    if (typeof picked === "string") startSession(picked);
  }

  function pointFromEvent(e: React.PointerEvent): Pt {
    const r = imgRef.current!.getBoundingClientRect();
    return {
      x: Math.max(0, Math.min(r.width, e.clientX - r.left)),
      y: Math.max(0, Math.min(r.height, e.clientY - r.top)),
    };
  }

  function onPointerDown(e: React.PointerEvent) {
    if (!workingPath) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    drawing.current = true;
    const p = pointFromEvent(e);
    anchor.current = p;
    setDraft(mode === "rect" ? rectPoly(p, p) : [p]);
  }

  function onPointerMove(e: React.PointerEvent) {
    if (!drawing.current) return;
    const p = pointFromEvent(e);
    if (mode === "rect") {
      setDraft(rectPoly(anchor.current!, p));
    } else {
      setDraft((d) => {
        if (!d) return [p];
        const last = d[d.length - 1];
        return Math.hypot(p.x - last.x, p.y - last.y) > 2 ? [...d, p] : d;
      });
    }
  }

  function onPointerUp(e: React.PointerEvent) {
    if (!drawing.current) return;
    drawing.current = false;
    e.currentTarget.releasePointerCapture(e.pointerId);
    setDraft((d) => {
      if (!d || d.length < 3) return null;
      const xs = d.map((p) => p.x);
      const ys = d.map((p) => p.y);
      const area = (Math.max(...xs) - Math.min(...xs)) * (Math.max(...ys) - Math.min(...ys));
      return area < 16 ? null : d;
    });
  }

  async function applyBlur() {
    const img = imgRef.current;
    if (!workingPath || !draft || draft.length < 3 || !img) return;
    const r = img.getBoundingClientRect();
    const sx = img.naturalWidth / r.width;
    const sy = img.naturalHeight / r.height;
    const points = draft.map((p) => [p.x * sx, p.y * sy]);

    setBusy(true);
    setStatus("Blurring…");
    try {
      const wp = await invoke<string>("apply_blur", {
        working: workingPath,
        points,
        sigma,
      });
      setHistory((h) => [...h.slice(0, histIndex + 1), wp]);
      setHistIndex((i) => i + 1);
      setDraft(null);
      setStatus("");
      setNav((n) => n + 1);
    } catch (err) {
      setStatus(`Error: ${err}`);
    } finally {
      setBusy(false);
    }
  }

  function step(to: number) {
    setHistIndex(to);
    setDraft(null);
    setNav((n) => n + 1);
  }

  async function saveCopy() {
    if (!workingPath) return;
    const dst = await save({ filters: [{ name: "JPEG", extensions: ["jpg", "jpeg"] }] });
    if (!dst) return;
    setBusy(true);
    setStatus("Saving…");
    try {
      await invoke("save_copy", { working: workingPath, dst });
      setStatus(`Saved: ${dst}`);
    } catch (err) {
      setStatus(`Error: ${err}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="app">
      <div className="toolbar">
        <button onClick={openViaDialog} disabled={busy}>
          Open…
        </button>
        <div className="modes">
          <button
            className={mode === "rect" ? "on" : ""}
            onClick={() => setMode("rect")}
          >
            Rectangle
          </button>
          <button
            className={mode === "lasso" ? "on" : ""}
            onClick={() => setMode("lasso")}
          >
            Freehand
          </button>
        </div>
        <button onClick={applyBlur} disabled={!canApply}>
          Apply blur
        </button>
        <button onClick={() => step(histIndex - 1)} disabled={!canUndo || busy}>
          Undo
        </button>
        <button onClick={() => step(histIndex + 1)} disabled={!canRedo || busy}>
          Redo
        </button>
        <button onClick={() => step(0)} disabled={!canUndo || busy}>
          Reset
        </button>
        <label className="strength">
          Blur
          <input
            type="range"
            min={1}
            max={30}
            value={sigma}
            onChange={(e) => setSigma(Number(e.currentTarget.value))}
          />
          <span>{sigma}</span>
        </label>
        <button onClick={saveCopy} disabled={!workingPath || busy}>
          Save copy…
        </button>
      </div>

      <div className={`stage${dragOver ? " drag-over" : ""}`}>
        {imgSrc ? (
          <div className="canvas">
            <img ref={imgRef} src={imgSrc} alt="" draggable={false} />
            <svg
              className="overlay"
              onPointerDown={onPointerDown}
              onPointerMove={onPointerMove}
              onPointerUp={onPointerUp}
            >
              {draft && draft.length >= 2 && (
                <polygon
                  points={draft.map((p) => `${p.x},${p.y}`).join(" ")}
                  className={mode === "lasso" && drawing.current ? "open" : ""}
                />
              )}
            </svg>
          </div>
        ) : (
          <p className="hint">
            Open or drop an image, then drag across it to pick a region to blur.
          </p>
        )}
      </div>

      <p className="status">
        {status || (workingPath ? `Step ${histIndex} / ${history.length - 1}` : "")}
      </p>
    </main>
  );
}

export default App;
