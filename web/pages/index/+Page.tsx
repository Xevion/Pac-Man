import { useEffect } from "react";

export default function Page() {
  useEffect(() => {
    if (!(window as any).Module) {
      const canvas = document.getElementById("canvas");

      (window as any).Module = {
        canvas: canvas,
        locateFile: (path: string) => {
          return path.startsWith("/") ? path : `/${path}`;
        },
        preRun: [],
      };

      const script = document.createElement("script");
      script.src = "/pacman.js";
      script.async = false;
      document.body.appendChild(script);

      return () => {
        script.remove();
      };
    }
  }, []);

  return (
    <div className="mt-4 flex justify-center h-[calc(100vh-120px)]">
      <div
        className="block border-1 border-yellow-400/50 aspect-[5/6] h-[min(calc(100vh-120px),_calc(95vw_*_6/5))] w-auto"
        style={{
          boxShadow:
            "0 0 12px rgba(250, 204, 21, 0.35), 0 0 2px rgba(255, 255, 255, 0.25)",
        }}
      >
        <canvas id="canvas" className="w-full h-full" />
      </div>
    </div>
  );
}
