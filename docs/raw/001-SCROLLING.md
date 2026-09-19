# App List Scrolling

2026.09.19

The launcher must support both pointer-driven scrolling and keyboard or remote navigation. The current implementation keeps one shared horizontal offset for both input methods.

## Attempts That Did Not Work

### A plain `HorizontalLayout`

The app cards were initially placed directly in a `HorizontalLayout`. The layout rendered every card but had no viewport, so cards beyond the window edge were inaccessible.

### Binding `Flickable` directly to the selected index

The first `Flickable` implementation used a binding like this:

```slint
content-x: selected-app * 324px;
```

This caused two problems:

- The `Flickable` also handles user input, so its own navigation could compete with the app-selection key bindings.
- `content-x` is a content translation, not a positive scroll distance. Positive values move the contents right, which is the opposite of what is needed to reveal later cards.

### A clipped `Rectangle` with a positioned row

A clipped `Rectangle` with an `x` binding on the app row gave deterministic keyboard scrolling. It removed `Flickable`, however, so mouse and trackpad scrolling stopped working.

### Permanently binding the `Flickable` offset to selection

A permanent binding from the selected index to the scroll offset would also overwrite pointer-driven scrolling whenever the binding reevaluated. Pointer scrolling and keyboard navigation need to update the same mutable state instead.

## Working Approach

`AppWindow` has two properties:

```slint
in-out property <int> selected-app: 0;
in-out property <length> app-scroll-x: 0px;
```

The app list is a `Flickable` connected to the shared offset:

```slint
app-list := Flickable {
    content-x <=> root.app-scroll-x;
    content-width: root.apps.length * 324px - 24px;
    content-height: 240px;
}
```

The two-way binding lets `Flickable` update `app-scroll-x` when the user scrolls with a mouse or trackpad.

Each keyboard navigation handler then updates both the selected index and the offset:

```slint
root.selected-app = Math.min(root.selected-app + 1, root.apps.length - 1);
root.app-scroll-x = -Math.max(
    0px,
    root.selected-app * 324px - (app-list.width - 300px),
);
```

Each card occupies `300px` and the gap is `24px`, making its horizontal stride `324px`. The expression leaves the row at its existing origin while the selected card fits in the viewport. Once it would cross the right edge, it shifts the content left just enough to make the complete selected card visible.

The negative sign is required because `Flickable.content-x` translates its content: `0px` is the leftmost position, while negative values move the content left and expose later cards.