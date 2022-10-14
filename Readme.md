`GuiCss` is a CSS parser designed for Rust Desktop GUI.

OS-specific and Dark-theme-specific CSS code is supported via dedicated
mediaQueries:
- `prefers-color-scheme:light/dark`
- `os-version: macos/linux/windows`

Rules and mediaQueries are invalidated when the file is modified by the
user, or when the system wide theme changes (Dark mode).

```css
@media (prefers-color-scheme: light) {
  hbox {
    --mycolor: black;
  }
}

@media (prefers-color-scheme: dark) {
  hbox {
    --mycolor: white;
  }
}

hbox {
  color: var(--mycolor);
  background-color: red !important;
}

scrollarea::scrollbar {
  width: 12px;
}

@media (os-version: macos) {
  hbox {
    --toolbar-padding: 12px;
  }
}
```

