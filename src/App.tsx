import { useRef, useState } from "react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import "./App.css";

const IMAGE_EXTS = ["png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff"];

type Rect = { x: number; y: number; w: number; h: number };

function normalize(x0: number, y0: number, x1: number, y1: number): Rect {
  return {
    x: Math.min(x0, x1),
    y: Math.min(y0, y1),
    w: Math.abs(x1 - x0),
    h: Math.abs(y1 - y0),
  };
}

function App() {
  const [srcPath, setSrcPath] = useState<string | null>(null);
  const [imgSrc, setImgSrc] = useState("");
  const [sel, setSel] = useState<Rect | null>(null);
  const [sigma, setSigma] = useState(8);
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState("");

  const imgRef = useRef<HTMLImageElement>(null);
  const dragStart = useRef<{ x: number; y: number } | null>(null);

  async function openImage() {
    const picked = await open({
      multiple: false,
      filters: [{ name: "Images", extensions: IMAGE_EXTS }],
    });
    if (typeof picked !== "string") return;
    setSrcPath(picked);
    setImgSrc(convertFileSrc(picked));
    setSel(null);
    setStatus("");
  }

  function pointFromEvent(e: React.PointerEvent) {
    const rect = imgRef.current!.getBoundingClientRect();
    return {
      x: Math.max(0, Math.min(rect.width, e.clientX - rect.left)),
      y: Math.max(0, Math.min(rect.height, e.clientY - rect.top)),
    };
  }

  function onPointerDown(e: React.PointerEvent) {
    if (!imgSrc) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    dragStart.current = pointFromEvent(e);
    setSel(null);
  }

  function onPointerMove(e: React.PointerEvent) {
    if (!dragStart.current) return;
    const p = pointFromEvent(e);
    setSel(normalize(dragStart.current.x, dragStart.current.y, p.x, p.y));
  }

  function onPointerUp(e: React.PointerEvent) {
    if (!dragStart.current) return;
    const p = pointFromEvent(e);
    setSel(normalize(dragStart.current.x, dragStart.current.y, p.x, p.y));
    dragStart.current = null;
    e.currentTarget.releasePointerCapture(e.pointerId);
  }

  async function saveResult() {
    const img = imgRef.current;
    if (!srcPath || !sel || !img) return;

    const dst = await save({
      filters: [{ name: "Images", extensions: IMAGE_EXTS }],
    });
    if (!dst) return;

    // displayed pixels -> source image pixels
    const rect = img.getBoundingClientRect();
    const sx = img.naturalWidth / rect.width;
    const sy = img.naturalHeight / rect.height;

    setBusy(true);
    setStatus("Blurring…");
    try {
      await invoke("blur_and_save", {
        src: srcPath,
        dst,
        x: Math.round(sel.x * sx),
        y: Math.round(sel.y * sy),
        width: Math.round(sel.w * sx),
        height: Math.round(sel.h * sy),
        sigma,
      });
      setSrcPath(dst);
      setImgSrc(`${convertFileSrc(dst)}?t=${Date.now()}`);
      setSel(null);
      setStatus(`Saved: ${dst}`);
    } catch (err) {
      setStatus(`Error: ${err}`);
    } finally {
      setBusy(false);
    }
  }

  const canSave = !!srcPath && !!sel && sel.w > 2 && sel.h > 2 && !busy;

  return (
    <main className="app">
      <div className="toolbar">
        <button onClick={openImage} disabled={busy}>
          Open image…
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
        <button onClick={saveResult} disabled={!canSave}>
          Save blurred copy…
        </button>
      </div>

      <div className="stage">
        {imgSrc ? (
          <div className="canvas">
            <img ref={imgRef} src={imgSrc} alt="" draggable={false} />
            <div
              className="capture"
              onPointerDown={onPointerDown}
              onPointerMove={onPointerMove}
              onPointerUp={onPointerUp}
            />
            {sel && (
              <div
                className="selection"
                style={{
                  left: sel.x,
                  top: sel.y,
                  width: sel.w,
                  height: sel.h,
                  backdropFilter: `blur(${sigma}px)`,
                  WebkitBackdropFilter: `blur(${sigma}px)`,
                }}
              />
            )}
          </div>
        ) : (
          <p className="hint">Open an image, then drag across it to pick a region to blur.</p>
        )}
      </div>

      <p className="status">{status}</p>
    </main>
  );
}

export default App;
