import { AspectRatio } from "@mantine/core";

export default function Page() {
  return (
    <div className="mt-4 flex justify-center h-[calc(100vh-120px)]">
      <AspectRatio ratio={1.0 / 1.2} w="min(100vh * 1.2, 100vw)" maw="95vw">
        <canvas
          className="block border-1 border-yellow-400/50 w-full h-full"
          style={{
            boxShadow: "0 0 12px rgba(250, 204, 21, 0.35), 0 0 2px rgba(255, 255, 255, 0.25)",
          }}
        />
      </AspectRatio>
    </div>
  );
}
