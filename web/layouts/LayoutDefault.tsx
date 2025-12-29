import "./tailwind.css";
import "@fontsource/pixelify-sans";
import "@fontsource/outfit/400.css";
import "@fontsource/outfit/500.css";
import "@fontsource/russo-one";
import "overlayscrollbars/overlayscrollbars.css";

import { useState, useEffect } from "react";
import { usePageContext } from "vike-react/usePageContext";
import { IconBrandGithub, IconDownload, IconDeviceGamepad3, IconTrophy } from "@tabler/icons-react";
import { OverlayScrollbarsComponent } from "overlayscrollbars-react";

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

export function Link({ href, label, icon }: { href: string; label: string; icon?: React.ReactNode }) {
  const pageContext = usePageContext();
  const { urlPathname } = pageContext;
  const isActive = href === "/" ? urlPathname === href : urlPathname.startsWith(href);
  return (
    <a 
      href={href} 
      className={`
        flex items-center gap-1.5 
        tracking-wide
        transition-colors duration-200
        ${isActive 
          ? "text-white" 
          : "text-gray-500 hover:text-gray-300"
        }
      `}
    >
      {icon}
      <span>{label}</span>
    </a>
  );
}

export default function LayoutDefault({ children }: { children: React.ReactNode }) {
  const [opened, setOpened] = useState(false);
  const toggle = () => setOpened((v) => !v);
  const close = () => setOpened(false);
  
  const pageContext = usePageContext();
  const { urlPathname } = pageContext;
  const isIndexPage = urlPathname === "/";
  const isLeaderboardPage = urlPathname.startsWith("/leaderboard");
  const isDownloadPage = urlPathname.startsWith("/download");

  const sourceLinks = links
    .filter((link) => !link.href.startsWith("/"))
    .map((link) => (
      <a
        href={link.href}
        title={link.label}
        key={link.label}
        target="_blank"
        className="text-gray-500 hover:text-gray-300 transition-colors duration-200"
      >
        {link.icon}
      </a>
    ));

  return (
    <div className="bg-black text-yellow-400 h-screen flex flex-col overflow-hidden">
      <header className="shrink-0 h-[60px] border-b border-yellow-400/25 bg-black z-20">
        <div className="h-full px-4 flex items-center justify-center">
          <button
            aria-label="Open navigation"
            onClick={toggle}
            className="sm:hidden absolute left-4 inline-flex items-center justify-center w-9 h-9 rounded border border-yellow-400/30 text-yellow-400"
          >
            <span className="sr-only">Toggle menu</span>
            <div className="w-5 h-0.5 bg-yellow-400" />
          </button>
          
          <div className="flex items-center gap-8">
            <Link href="/leaderboard" label="Leaderboard" icon={<IconTrophy size={18} />} />
            
            <a 
              href="/"
              onClick={(e) => {
                if (isIndexPage) {
                  e.preventDefault();
                }
              }}
            >
              <h1 
                className={`text-3xl tracking-[0.3em] title-hover ${
                  isIndexPage 
                    ? 'text-yellow-400' 
                    : 'glimmer-text'
                }`}
                style={{ fontFamily: 'Russo One' }}
              >
                PAC-MAN
              </h1>
            </a>
            
            <Link href="/download" label="Download" icon={<IconDownload size={18} />} />
          </div>
          
          <div className="absolute right-4 hidden sm:flex gap-4 items-center">{sourceLinks}</div>
        </div>
      </header>
      
      <OverlayScrollbarsComponent 
        defer
        options={{
          scrollbars: {
            theme: 'os-theme-light',
            autoHide: 'scroll',
            autoHideDelay: 1300,
          },
        }}
        className="flex-1"
      >
        <main>{children}</main>
      </OverlayScrollbarsComponent>

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
                <Link href={link.href} key={link.label} label={link.label} icon={link.icon} />
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
