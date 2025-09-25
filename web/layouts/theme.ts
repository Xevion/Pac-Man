import type { MantineThemeOverride } from "@mantine/core";
import { createTheme } from "@mantine/core";

const theme: MantineThemeOverride = createTheme({
  /** Put your mantine theme override here */
  primaryColor: "yellow",

  fontFamily: "'Nunito', sans-serif",
  headings: {
    fontFamily: "'Nunito', sans-serif",
    fontWeight: "800",
  },
  components: {
    AppShell: {
      styles: {
        header: {
          backgroundColor: "#000",
          borderBottom: "1px solid rgba(250, 204, 21, 0.25)",
        },
        navbar: {
          backgroundColor: "#000",
          borderRight: "1px solid rgba(250, 204, 21, 0.25)",
        },
      },
    },
  },
});

export default theme;
