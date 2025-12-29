import "../../layouts/tailwind.css";

export default function GameLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="bg-black text-yellow-400 h-full flex flex-col overflow-hidden">
      <main className="flex-1 overflow-hidden">{children}</main>
    </div>
  );
}
