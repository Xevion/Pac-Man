// https://vike.dev/Head

//# BATI.has("mantine")
import { ColorSchemeScript } from "@mantine/core";

export default function HeadDefault() {
  return (
    <>
      <link rel="icon" href="/favicon.ico" />
      <ColorSchemeScript />
    </>
  );
}
