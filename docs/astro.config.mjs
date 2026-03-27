import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

export default defineConfig({
  site: "https://kdheepak.com",
  base: "/taskwarrior-tui",
  trailingSlash: "always",
  outDir: "../site",
  integrations: [
    starlight({
      title: "taskwarrior-tui",
      description: "A terminal user interface for Taskwarrior with keyboard-first workflows.",
      favicon: "/favicon.svg",
      disable404Route: true,
      editLink: {
        baseUrl: "https://github.com/kdheepak/taskwarrior-tui/edit/main/docs/",
      },
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/kdheepak/taskwarrior-tui",
        },
      ],
      sidebar: [
        {
          label: "Getting Started",
          items: ["installation", "quick_start", "keybindings", "troubleshooting", "faqs"],
        },
        {
          label: "Configuration",
          items: ["configuration/keys", "configuration/colors", "configuration/advanced"],
        },
        {
          label: "Developer Guide",
          items: ["developer/guide"],
        },
      ],
    }),
  ],
});
