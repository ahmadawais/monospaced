![Monospaced](https://raw.githubusercontent.com/ahmadawais/monospaced/main/.github/monospaced.png)

# Monospaced

Unicode monospace text converter for quick paste, convert, and copy.

The site takes regular text and swaps supported letters and numbers with Unicode monospace characters so the result keeps a fixed-width, typewriter-style feel across many apps and websites.

Live domain: [monospaced.awais.dev](https://monospaced.awais.dev)

## What It Does

- Converts plain text into Unicode monospace text
- Auto-copies converted output when possible
- Keeps the UI lightweight and mobile-friendly
- Ships as a single static page with no build step

## CLI

There is also a Rust CLI in [`cli/`](./cli) that ships on npm as `monospaced`.
It prints the converted text and also copies it to the clipboard when the OS clipboard is available.

Run it directly with `npx`:

```bash
npx monospaced "npx create-next-app 14"
```

Direct args after install:

```bash
monospaced "npx create-next-app 14"
```

Piped input:

```bash
echo "npx create-next-app 14" | monospaced
```

Interactive mode:

```bash
monospaced
```

Help:

```bash
monospaced --help
```

## Attribution

Made by (human) [Ahmad Awais](https://x.com/MrAhmadAwais) and (agent) [Command Code](https://commandcode.ai).
