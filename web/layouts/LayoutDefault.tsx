import "./tailwind.css";
import "@fontsource/pixelify-sans";
import "@fontsource/nunito/800.css";
import "@fontsource/nunito";

import { useState } from "react";
import { usePageContext } from "vike-react/usePageContext";
import { IconBrandGithub, IconDownload, IconDeviceGamepad3, IconTrophy } from "@tabler/icons-react";

const links = [
  {
    label: "Play",
    href: "/",
    icon: <IconDeviceGamepad3 size={28} />,
  },
  {
    label: "Leaderboard",
    href: "/leaderboard",
    icon: <IconTrophy size={28} />,
  },
  {
    label: "Download",
    href: "/download",
    icon: <IconDownload size={28} />,
  },
  {
    label: "GitHub",
    href: "https://github.com/Xevion/Pac-Man",
    icon: <IconBrandGithub size={28} />,
  },
];

export function Link({ href, label }: { href: string; label: string }) {
  const pageContext = usePageContext();
  const { urlPathname } = pageContext;
  const isActive = href === "/" ? urlPathname === href : urlPathname.startsWith(href);
  return (
    <a href={href} className={isActive ? "text-yellow-400" : "text-gray-400"}>
      {label}
    </a>
  );
}

export default function LayoutDefault({ children }: { children: React.ReactNode }) {
  const [opened, setOpened] = useState(false);
  const toggle = () => setOpened((v) => !v);
  const close = () => setOpened(false);

  const mainLinks = links
    .filter((link) => link.href.startsWith("/"))
    .map((link) => <Link href={link.href} key={link.label} label={link.label} />);

  const sourceLinks = links
    .filter((link) => !link.href.startsWith("/"))
    .map((link) => (
      <a
        href={link.href}
        title={link.label}
        key={link.label}
        target="_blank"
        className="transition-[drop-shadow] duration-1000 hover:drop-shadow-sm drop-shadow-yellow-400"
      >
        {link.icon}
      </a>
    ));

  return (
    <div className="bg-black text-yellow-400 min-h-screen flex flex-col">
      <header className="sticky top-0 z-20 h-[60px] border-b border-yellow-400/25 bg-black">
        <div className="h-full px-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <button
              aria-label="Open navigation"
              onClick={toggle}
              className="sm:hidden inline-flex items-center justify-center w-9 h-9 rounded border border-yellow-400/30 text-yellow-400"
            >
              <span className="sr-only">Toggle menu</span>
              <div className="w-5 h-0.5 bg-yellow-400" />
            </button>
            <nav className="hidden sm:flex gap-4 items-center">{mainLinks}</nav>
          </div>
          <div className="hidden sm:flex gap-4 items-center">{sourceLinks}</div>
        </div>
      </header>
      <main className="flex-1">{children}</main>

      {opened && (
        <div className="fixed inset-0 z-30">
          <div className="absolute inset-0 bg-black/60" onClick={close} />
          <div className="absolute left-0 top-0 h-full w-72 max-w-[80vw] bg-black border-r border-yellow-400/25 p-4">
            <div className="mb-4 flex items-center justify-between">
              <h2 className="text-lg font-bold">Navigation</h2>
              <button
                aria-label="Close navigation"
                onClick={close}
                className="inline-flex items-center justify-center w-8 h-8 rounded border border-yellow-400/30 text-yellow-400"
              >
                ✕
              </button>
            </div>
            <div className="flex flex-col gap-3">
              {links.map((link) => (
                <Link href={link.href} key={link.label} label={link.label} />
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
