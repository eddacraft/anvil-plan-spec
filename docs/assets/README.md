# Documentation Assets

Image and binary assets referenced from the README and other docs.

## Assets

- `social-preview.svg` / `social-preview.png` — GitHub social/OG preview card
  (1280×640). Built on the eddacraft/anvil brand system (see
  [`eddacraft/brand-and-design`](https://github.com/eddacraft/brand-and-design)):
  Void background, the official anvil brandmark, Departure Mono wordmark, a
  terminal-window container with mono facts, anvil-ember `$` and edda-green
  `[ OK ]` — no gradients, no shadows, sharp corners.
  - Re-render from source: `rsvg-convert -w 1280 -h 640 social-preview.svg -o social-preview.png`
  - **Set it:** repo **Settings → General → Social preview → Edit → upload the PNG**
    (web-UI only; committing the file here does not set the preview).

Pending captures:

- `init-wizard.png` — screenshot of the Ratatui-based `aps init` onboarding wizard
