import "@mantine/core/styles.css";
import "./tailwind.css";
import "@fontsource/pixelify-sans";
import "@fontsource/nunito/800.css";

import { AppShell, Burger, Group, MantineProvider, Flex, Stack, Drawer } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import theme from "./theme";
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
  const [opened, { toggle, close }] = useDisclosure();

  const mainLinks = links
    .filter((link) => link.href.startsWith("/"))
    .map((link) => <Link href={link.href} label={link.label} />);

  const sourceLinks = links
    .filter((link) => !link.href.startsWith("/"))
    .map((link) => (
      <a
        href={link.href}
        title={link.label}
        target="_blank"
        className="transition-all duration-300 hover:drop-shadow-sm hover:drop-shadow-yellow-400"
      >
        {link.icon}
      </a>
    ));

  return (
    <MantineProvider forceColorScheme="dark" theme={theme}>
      <div className="bg-black text-yellow-400 min-h-screen flex flex-col">
        <AppShell header={{ height: 60 }} padding="md">
          <AppShell.Header>
            <Flex h="100%" px="md" align="center" justify="space-between">
              <Flex h="100%" align="center" gap="md">
                <Burger opened={opened} onClick={toggle} hiddenFrom="sm" size="sm" />
                <Group visibleFrom="sm">{mainLinks}</Group>
              </Flex>
              <Group visibleFrom="sm">{sourceLinks}</Group>
            </Flex>
          </AppShell.Header>
          <AppShell.Main>{children}</AppShell.Main>
        </AppShell>
        <Drawer opened={opened} onClose={close} title="Navigation">
          <Stack>
            {links.map((link) => (
              <Link href={link.href} label={link.label} />
            ))}
          </Stack>
        </Drawer>
      </div>
    </MantineProvider>
  );
}
