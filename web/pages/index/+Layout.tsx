import "../../layouts/tailwind.css";

export default function GameLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="bg-black text-yellow-400 min-h-screen flex flex-col">
      <main className="flex-1">{children}</main>
    </div>
  );
}
