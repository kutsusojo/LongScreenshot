# LongScreenshot project instructions

## Project goal

Build a small Android application that creates a long screenshot by stitching together multiple vertically overlapping screenshots.

The project is primarily for learning Rust.

The user already knows programming, especially Python and scientific programming, but is learning Rust.

## Teaching mode

Work incrementally and keep the project runnable after every checkpoint.

By default:

1. Inspect existing files before proposing changes.
2. State the goal and exact files involved.
3. Give the smallest coherent code change for the user to type.
4. Explain important Rust syntax and design choices.
5. Give exact build and test commands.
6. Describe the expected output.
7. Wait for the user's test result before continuing.

Do not edit files unless the user explicitly asks Codex to apply the changes.

Do not generate the entire application in one step.

When introducing ownership, borrowing, slices, Result, traits, modules, FFI, or unsafe code, briefly explain why the concept is needed.

Prefer readable, idiomatic Rust over clever abstractions.

Do not use unwrap() for recoverable application errors.

## Architecture

Android-specific code:

- Kotlin
- Jetpack Compose
- Android Photo Picker
- result preview
- save and share

Rust code:

- image decoding
- image representation and processing
- overlap detection
- cropping and stitching
- PNG output
- understandable error handling

Bridge:

- JNI
- cargo-ndk
- arm64-v8a first
- Rust target: aarch64-linux-android

## Intended structure

```text
LongScreenshot/
├── app/
│   └── Android application
└── stitcher/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── error.rs
        ├── image_input.rs
        ├── overlap.rs
        └── stitch.rs
```

Do not create later Rust modules before they are needed.

## Milestones

### Milestone 1 — Rust runs on the physical Android phone

Create the smallest Compose application with one button.

Pressing the button must call Rust through JNI:

add(20, 22) -> 42

Display 42 in the Android UI.

This must verify:

Android UI -> Kotlin -> JNI -> Rust -> JNI -> Kotlin -> UI

For this milestone, explain:

- what Cargo crate-type cdylib means
- why Android loads a .so file
- what JNI does
- extern "system"
- exported JNI symbols
- arm64-v8a versus aarch64-linux-android
- where libstitcher.so is packaged

Build and fix compilation errors before stopping. Stop after the user successfully tests this milestone on the physical phone.

### Milestone 2 — Screenshot selection

Use Android Photo Picker to select multiple screenshots and display their order. Do not stitch images yet.

### Milestone 3 — Rust reads images

Make selected encoded images safely accessible to Rust. Rust decodes them and reports image count, width, and height.

Introduce Result and a custom understandable error type.

### Milestone 4 — Pairwise overlap detection

Implement overlap detection entirely in Rust without ML or OpenCV.

Initially assume:

- identical widths
- identical scale
- correct top-to-bottom order
- vertical scrolling
- approximately zero horizontal displacement
- substantial overlap

Compare the bottom region of image A with the top region of image B using a numerical pixel-error metric. Grayscale conversion, downsampling, and ignored edge margins are acceptable.

Return overlap height and an error or confidence score. Add synthetic Rust unit tests.

### Milestone 5 — Stitch two screenshots

Keep all of image A, remove the duplicate top portion of image B, append the remainder, and produce a PNG.

### Milestone 6 — Stitch N screenshots

Support 2–10 screenshots. Prefer detecting overlaps between consecutive original images and allocating the final output once when practical.

### Milestone 7 — Android result UI

Display the result and allow saving, exporting, and sharing it.

## Explicit non-goals for version 1

Do not implement:

- automatic screen capture
- automatic scrolling
- machine learning
- OpenCV
- automatic image ordering
- difficult fixed-header or animation handling

Those may be considered only after the basic stitcher works reliably.

## Error messages

Eventually translate Rust errors into understandable Android messages, including:

- Could not decode image
- Images have different widths
- Could not find a reliable overlap
- Not enough screenshots selected

## Performance principles

Correctness and clarity are more important than optimization initially.

However:

- avoid repeated full-resolution comparisons
- borrow image data when practical
- avoid unnecessary copies of large pixel buffers
- explain meaningful memory decisions

## Project state

At the beginning of every task, read PROJECT_STATUS.md.

Work only on the current milestone unless the user explicitly changes it.

After the user confirms a checkpoint works, update PROJECT_STATUS.md before proceeding.
