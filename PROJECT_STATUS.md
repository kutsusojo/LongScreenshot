# LongScreenshot project status

Updated: 2026-09-09

## Current milestone

Milestone 7 — Android result UI — in progress.

Milestones 1, 2, 3, 4, 5, and 6 are complete. Milestone 7 has started.

## Completed setup

- Project directory:
  C:\Users\sojo\Projects\LongScreenshot
- The directory was confirmed empty before project setup.
- Windows host architecture: AMD64
- rustup 1.29.1 installed
- rustc 1.98.1 installed
- Cargo 1.98.1 installed
- rustfmt installed
- Clippy installed
- Windows host target installed:
  x86_64-pc-windows-msvc
- Android Rust target installed:
  aarch64-linux-android
- Codex filesystem access works from the new workspace.
- Android Studio installed and verified.
- Android Studio bundled JDK verified:
  OpenJDK 25.0.2
- Android SDK installed at:
  C:\Users\sojo\AppData\Local\Android\Sdk
- Android SDK Platform-Tools and ADB 37.0.1 installed.
- Stable Android NDK r29 installed and verified:
  29.0.14206865
- Android NDK r30 beta 3 is also installed but will not be used for the project.
- Kotlin/Jetpack Compose Empty Activity project created under:
  C:\Users\sojo\Projects\LongScreenshot\app
- The unchanged debug application built successfully with:
  .\gradlew.bat assembleDebug
- Debug APK created at:
  C:\Users\sojo\Projects\LongScreenshot\app\app\build\outputs\apk\debug\app-debug.apk
- Physical Xiaomi phone connected and authorized through ADB.
- Physical phone verified as Android 14, API 34, with arm64-v8a ABI.
- The unchanged application was installed as com.example.longscreenshot and displayed "Hello Android!" on the physical phone.
- Rust and Cargo were verified from the Android Studio terminal.
- cargo-ndk 4.1.2 installed and verified.
- cargo-ndk was explicitly pointed at stable NDK r29 and generated the expected aarch64-linux-android linker and sysroot environment.
- Minimal Rust library crate created at:
  C:\Users\sojo\Projects\LongScreenshot\stitcher
- The generated Rust host unit test passes.
- Rust add function changed to use i32 values and verified with add(20, 22) == 42.
- cargo fmt, cargo test, and cargo clippy -- -D warnings all pass.
- stitcher is configured with crate-type = ["cdylib"].
- Rust library cross-compiled with stable NDK r29 for arm64-v8a at Android API 24.
- libstitcher.so was produced and its ELF header identifies a 64-bit AArch64 shared object.
- jni 0.22.4 dependency added.
- Rust exports Java_com_example_longscreenshot_MainActivity_add with the JNI system ABI.
- Formatting, host tests, Clippy, and the Android cross-build pass with the JNI entry point.
- llvm-nm confirms the JNI symbol is globally exported from libstitcher.so.
- libstitcher.so copied to app/app/src/main/jniLibs/arm64-v8a/.
- The Android debug APK rebuilt successfully and contains lib/arm64-v8a/libstitcher.so.
- MainActivity loads libstitcher.so with System.loadLibrary("stitcher").
- MainActivity declares the external add(Int, Int) function.
- The application still launches and displays "Hello Android!" on the physical phone, verifying that the native library loads successfully.
- The generated greeting was replaced with one Compose button and a result display.
- Pressing "Call Rust" on the physical phone calls add(20, 22) through JNI and changes the displayed result from 0 to 42.
- MainActivity launches Android Photo Picker for up to 10 images.
- Selecting multiple screenshots updates the displayed selection count on the physical phone.
- The Milestone 1 Rust button still returns 42 after the Photo Picker change.
- The selected Uri list is displayed in the order returned by Photo Picker with positions starting at 1.
- The Compose screen is vertically scrollable so all selected entries remain accessible.
- Three selected screenshots were verified on the physical phone with three distinct displayed Uri entries.
- image 0.25.10 added with default features disabled and only PNG/JPEG support enabled.
- Host tests, Clippy, and the ARM64 Android cross-build pass with the image dependency.
- Added error.rs with StitcherError::CouldNotDecodeImage and retained decoder source errors.
- Added image_input.rs with ImageInfo and decode_image_info(&[u8]) -> Result<ImageInfo, StitcherError>.
- Host tests verify valid PNG dimensions and invalid encoded data.
- rustfmt, all three host tests, Clippy, and the ARM64 Android cross-build pass.
- Added decode_image_infos(&[&[u8]]) -> Result<Vec<ImageInfo>, StitcherError>.
- Multiple-image decoding preserves input order and Vec::len provides the image count.
- Tests verify two images with distinct dimensions and one-based reporting of an invalid second image.
- Manual review plus rustfmt, all five host tests, Clippy, and the ARM64 Android cross-build pass.
- Added the JNI entry point Java_com_example_longscreenshot_MainActivity_inspectImages.
- The JNI entry point accepts Kotlin Array<ByteArray> as JObjectArray<JByteArray>.
- Encoded Java byte arrays are copied into Rust-owned Vec<u8> buffers at the JNI boundary, then borrowed as &[u8] for decoding.
- Rust formats the decoded image count and each image's width and height as a Java String.
- Decoder failures are returned as understandable custom error strings; JNI infrastructure errors and panics are safely handled at the FFI boundary.
- All six host tests, Clippy with warnings denied, and the ARM64 Android cross-build pass.
- llvm-nm confirms Java_com_example_longscreenshot_MainActivity_inspectImages is globally exported from libstitcher.so.
- The updated libstitcher.so was copied into app/app/src/main/jniLibs/arm64-v8a/ and its SHA-256 hash matches the Cargo build output.
- The Android debug APK rebuilt successfully and contains the updated arm64-v8a libstitcher.so.
- The Android symbol-strip warning is non-fatal; Gradle packages the Rust debug library without stripping it.
- MainActivity now declares inspectImages(encodedImages: Array<ByteArray>): String, and the Android debug build succeeds with that JNI declaration.
- MainActivity reads selected Photo Picker Uris on Dispatchers.IO, closes each InputStream with use, and creates an ordered Array<ByteArray>.
- Three selected images were read successfully on the physical phone: 319,594 encoded bytes total.
- MainActivity passes the ordered Array<ByteArray> to Rust through inspectImages and displays Rust's returned report.
- Physical-phone verification succeeded with three decoded images: 900 x 1200, 1051 x 1051, and 1551 x 2048.
- This verifies the complete Milestone 3 path: Photo Picker Uri -> Kotlin ByteArray -> JNI -> Rust-owned Vec<u8> -> borrowed &[u8] -> Rust image decoder -> JString -> Compose UI.
- Milestone 4A added GrayscaleImage with owned row-major Vec<u8> pixels and a borrowed &[u8] accessor.
- decode_grayscale_image converts an encoded PNG/JPEG to one-byte luminance pixels while preserving width and height.
- A synthetic 2 x 3 grayscale PNG verifies row-major pixel order [10, 20, 30, 40, 50, 60].
- rustfmt, all seven host tests, Clippy with warnings denied, and the ARM64 Android cross-build pass for Milestone 4A.
- Milestone 4B added overlap.rs with mean_absolute_error for two borrowed grayscale pixel slices.
- The metric uses u8::abs_diff per pixel, accumulates into u64, and returns the mean as f64.
- Empty or differently sized slices return None rather than producing an invalid score.
- rustfmt, all ten host tests, Clippy with warnings denied, and the ARM64 Android cross-build pass for Milestone 4B.
- Milestone 4C added overlap_error for one supplied overlap height.
- overlap_error compares the bottom grayscale strip of image A with the equally tall top strip of image B without copying either strip.
- It rejects zero width, unequal widths, zero overlap, and overlap heights that do not fit both images.
- Synthetic tests verify an exact two-row overlap, a known MAE of 2.0, and invalid dimensions.
- rustfmt, all thirteen host tests, Clippy with warnings denied, and the ARM64 Android cross-build pass for Milestone 4C.
- Milestone 4D added OverlapMatch with an overlap height and MAE score.
- find_best_overlap evaluates an inclusive candidate-height range and retains the valid candidate with the lowest error.
- Synthetic tests find the unique three-row overlap and reject zero, reversed, or entirely out-of-range searches.
- rustfmt, all fifteen host tests, Clippy with warnings denied, and the ARM64 Android cross-build pass for Milestone 4D.
- Milestone 4E added ImagesHaveDifferentWidths and CouldNotFindReliableOverlap errors.
- find_reliable_overlap returns Result, rejects unequal widths, and accepts a best match only when its full MAE is within a caller-supplied finite nonnegative limit.
- Synthetic tests verify acceptance below the limit, rejection above it, and understandable unequal-width reporting.
- rustfmt, all eighteen host tests, Clippy with warnings denied, and the ARM64 Android cross-build pass for Milestone 4E.
- Milestone 4F added sampled_overlap_error and uses every eighth row and column while ranking overlap candidates.
- The selected candidate is re-evaluated with the existing full-resolution overlap_error before its score is returned or checked against the reliability limit.
- Tests verify sampled scoring, rejection of a zero sampling step, and a final full-resolution MAE of 41.5 when unsampled pixels differ.
- rustfmt, all twenty-one host tests, Clippy with warnings denied, and the ARM64 Android cross-build pass for Milestone 4F.
- Milestone 4G added detect_encoded_overlap for two borrowed encoded PNG/JPEG byte slices.
- Each encoded image is decoded exactly once into an owned GrayscaleImage, then both decoded images are borrowed by the reliable overlap search.
- Synthetic encoded PNGs verify a three-row exact overlap, and invalid bytes in the second input report Could not decode image 2.
- A manual review found no Rust correctness, formatting, or Clippy issues across lib.rs, error.rs, image_input.rs, and overlap.rs.
- rustfmt, all twenty-three host tests, Clippy with warnings denied, and the ARM64 Android cross-build pass for Milestone 4G.
- Milestone 4H added default_overlap_range using 20 percent of the shorter image through one row less than its full height.
- detect_default_encoded_overlap decodes both inputs once and applies the default candidate range with an initial maximum full-resolution MAE of 5.0.
- Tests verify range derivation, successful default detection, and rejection of excessive error.
- rustfmt, all twenty-six host tests, Clippy with warnings denied, and the ARM64 Android cross-build pass for Milestone 4H.
- Milestone 4I added Java_com_example_longscreenshot_MainActivity_inspectOverlap for two Java byte arrays.
- The JNI boundary copies both arrays into Rust-owned Vec<u8> buffers, calls detect_default_encoded_overlap, and returns either height/full MAE or an understandable error as JString.
- llvm-nm confirms the inspectOverlap JNI symbol is globally exported from the ARM64 libstitcher.so.
- The overlap report formatting test is correctly grouped inside the cfg(test) module.
- rustfmt, all twenty-seven host tests, Clippy with warnings denied, and the ARM64 Android cross-build pass for Milestone 4I.
- Milestone 4J copied the updated ARM64 libstitcher.so into the Android jniLibs directory; its SHA-256 matches the Cargo output.
- MainActivity declares inspectOverlap(firstEncoded: ByteArray, secondEncoded: ByteArray): String.
- The Android debug APK rebuild succeeds and contains lib/arm64-v8a/libstitcher.so.
- Milestone 4K connected the two-image Compose action to the Rust overlap inspector and verified it on the physical phone.
- The first real pair of 1220 x 2712 consecutive webpage screenshots produced a best candidate height of 1040 with full-image MAE 24.637, above the reliability limit of 5.0.
- The diagnostic path now reports the best candidate, MAE, reliability limit, and reliability decision without weakening the production threshold.
- Visual inspection shows fixed status and navigation bars, which violate the current whole-strip comparison model and are the leading explanation for the high MAE.
- Milestone 4L added vertical-margin-aware sampled and full-resolution overlap scoring while preserving the original zero-margin functions.
- The default policy ignores 5 percent at each vertical end of the compared strips, excluding fixed system bars without changing the MAE reliability limit of 5.0.
- A synthetic test verifies that mismatching fixed bands produce MAE 47.2 without a margin and MAE 0.0 when the bands are excluded.
- Retesting the same physical-phone screenshots retained the best overlap height of 1040 and reduced MAE from 24.637 to 0.031, producing Reliable: yes.
- Final Milestone 4 verification passes: rustfmt check, all 28 host tests, Clippy with warnings denied, and the ARM64 Android cross-build with NDK r29.
- Milestone 5A added stitch.rs with stitch_rgba_images for two borrowed RGBA images and a supplied overlap height.
- The stitcher validates widths and overlap height, calculates dimensions with checked arithmetic, allocates the final RGBA buffer once, preserves all of image A, and appends only the non-overlapping tail of image B.
- Added understandable InvalidOverlapHeight and CouldNotCreateOutputImage errors.
- Synthetic tests verify output dimensions and row order, unequal-width reporting, and invalid-overlap reporting.
- rustfmt, all 31 host tests, Clippy with warnings denied, and the ARM64 Android cross-build pass for Milestone 5A.
- Milestone 5B added encode_png for a borrowed RGBA image using PngEncoder and a Vec<u8> output buffer without cloning the full image.
- Added CouldNotEncodeImage with the underlying image::ImageError retained as its error source.
- A round-trip synthetic test verifies the PNG signature, decoded dimensions, and stitched pixel order.
- rustfmt, all 32 host tests, Clippy with warnings denied, and the ARM64 Android cross-build pass for Milestone 5B.
- Milestone 5C added decode_rgba_image for borrowed encoded PNG/JPEG bytes.
- stitch_encoded_images now composes RGBA decoding, supplied-height stitching, and PNG encoding in one pure-Rust pipeline.
- Synthetic tests verify the encoded-input PNG result and correct reporting of an invalid second image.
- rustfmt, all 34 host tests, Clippy with warnings denied, and the ARM64 Android cross-build pass for Milestone 5C.
- Milestone 5D added stitch_default_encoded_images, composing reliable default overlap detection with encoded RGBA stitching and PNG output.
- The automatic pipeline uses detect_default_encoded_overlap, so candidates above the MAE reliability limit are rejected before stitching.
- Synthetic tests verify successful automatic PNG stitching and rejection of unrelated images.
- rustfmt, all 36 host tests, Clippy with warnings denied, and the ARM64 Android cross-build pass for Milestone 5D.
- Milestone 5E added Java_com_example_longscreenshot_MainActivity_stitchImages, accepting two Java byte arrays and returning stitched PNG bytes as a Java byte array.
- StitcherError values are thrown into Java, JNI failures use ThrowRuntimeExAndDefault, and panics remain contained at the FFI boundary.
- rustfmt, all 36 host tests, Clippy with warnings denied, the ARM64 Android cross-build, and the exported stitchImages JNI symbol verification pass.
- Milestone 5F copied the updated ARM64 libstitcher.so into Android jniLibs; the Cargo and Android copies have matching SHA-256 hashes.
- MainActivity declares stitchImages(ByteArray, ByteArray): ByteArray.
- The debug APK was rebuilt, and llvm-nm confirms the copied library exports Java_com_example_longscreenshot_MainActivity_stitchImages.
- Milestone 5G added a physical-phone stitching action that reads exactly two Photo Picker inputs on Dispatchers.IO and calls Rust through JNI.
- Android validates the returned PNG in bounds-only mode to avoid allocating a second full bitmap merely to inspect dimensions.
- The physical-phone result was a valid 4,943,974-byte PNG with dimensions 1220 x 4378.
- This verifies the complete Milestone 5 path: two encoded screenshots -> JNI -> reliable Rust overlap detection -> RGBA cropping and stitching -> Rust PNG encoding -> JNI ByteArray -> Android PNG validation.
- Milestone 6A added stitch_rgba_image_sequence for 2-10 borrowed decoded RGBA images and N-1 supplied overlap heights.
- The sequence stitcher validates image count, overlap count, matching widths, and each overlap against its consecutive pair of original images.
- It calculates the final height before copying pixels and allocates the final RGBA output buffer once.
- New understandable errors cover too few screenshots, too many screenshots, and an invalid number of overlaps.
- Formatting, all 41 host tests, Clippy with warnings denied, and the ARM64 Android cross-build with NDK r29 pass for Milestone 6A.
- Milestone 6B extracted default overlap detection for borrowed decoded grayscale images and added detect_default_overlaps for an image sequence.
- Pairwise detection uses windows(2), so overlaps are measured between consecutive original images and middle images are borrowed without being decoded again.
- The sequence returns N-1 OverlapMatch values and stops with an understandable error if any consecutive pair is unreliable.
- Formatting, all 43 host tests, Clippy with warnings denied, and the ARM64 Android cross-build with NDK r29 pass for Milestone 6B.
- Milestone 6C added stitch_default_encoded_image_sequence for 2-10 borrowed encoded screenshots and retained the two-image wrapper for compatibility.
- Each encoded input is decoded once to RGBA; a smaller grayscale representation is derived for overlap detection and released before the final stitched allocation.
- Consecutive overlap heights feed stitch_rgba_image_sequence, the original RGBA inputs are released after stitching, and one PNG is encoded.
- Tests cover three-image automatic stitching, one-based decode error positions, and count validation before decoding.
- Formatting, all 47 host tests, Clippy with warnings denied, and the ARM64 Android cross-build with NDK r29 pass for Milestone 6C.
- Milestone 6D added Java_com_example_longscreenshot_MainActivity_stitchImageSequence, accepting an ordered Java Array<ByteArray> and returning stitched PNG bytes.
- The JNI boundary copies each Java byte array into Rust-owned storage, borrows those buffers for the 2-10 image pipeline, throws Rust errors into Java, and contains JNI failures and panics at the FFI boundary.
- Formatting, all 47 host tests, and Clippy with warnings denied pass; llvm-nm confirms the new JNI function is globally exported from the ARM64 libstitcher.so.
- Milestone 6E copied the updated ARM64 libstitcher.so into Android jniLibs; its SHA-256 hash matches the Cargo build output.
- MainActivity declares stitchImageSequence(Array<ByteArray>): ByteArray with a name and parameter type matching the exported JNI symbol.
- The Android debug APK rebuild succeeds with the new native declaration and packaged library.
- Milestone 6F connected the Android stitch action to every selected screenshot when the selection contains 2-10 images.
- The Android UI passes the ordered Array<ByteArray> through JNI to the Rust sequence pipeline and validates the returned PNG dimensions without decoding the full result bitmap.
- Physical-phone verification succeeded with three screenshots, producing a valid 7,326,178-byte PNG with dimensions 1220 x 6159.
- This verifies the complete Milestone 6 path: three ordered encoded screenshots -> JNI array -> consecutive Rust overlap detection -> one final stitched allocation -> PNG encoding -> JNI ByteArray -> Android validation.
- Milestone 7A retains a successful stitched PNG as in-memory Compose state and clears the previous output when inputs change or a new stitch begins.
- JNI, image reading, and PNG validation remain on Dispatchers.IO, while Compose state updates occur after returning to the main thread.
- The app deliberately uses remember rather than rememberSaveable so multi-megabyte PNG data is not placed in Android saved-instance state.
- The Android debug build and physical-phone checks pass: a successful result is retained with the expected byte count, and changing the selection clears it.
- Milestone 7B decodes a bounded preview bitmap on Dispatchers.IO while retaining the original PNG bytes for later save and share operations.
- The preview renders successfully in the vertically scrollable Compose screen and clears when the selected inputs change.
- Preview inspection exposed repeated fixed bottom navigation bars at internal seams: overlap scoring ignores those bands, but the current pixel-copy seam still preserves them.
- Milestone 7C shares the default vertical-margin calculation between overlap detection and stitch placement.
- Each internal sequence seam is moved upward by the trusted vertical margin: the earlier screenshot ends before its fixed bottom band and the following screenshot begins earlier within the matching overlap.
- Moving both crop boundaries by the same amount preserves the final stitched dimensions and avoids skipping scrollable content.
- A synthetic regression test verifies that mismatching fixed bands are absent from the stitched rows.
- Formatting, all 49 host tests, Clippy with warnings denied, and the ARM64 Android cross-build with NDK r29 pass for the Milestone 7C Rust change.
- The updated ARM64 library was packaged and the internal-seam fix was verified on the physical phone.
- Internal fixed bottom navigation bars are removed while the stitched content remains continuous.
- Milestone 7D added MediaStore saving for Android 10 and newer, publishing completed PNG files to Pictures/LongScreenshot.
- The save action writes on Dispatchers.IO, uses IS_PENDING while writing, and reports the saved display name.
- A verified source/configuration backup was created at backups/LongScreenshot-source-before-ui-20260908-2200.zip before simplifying the UI.
- Milestone 7E reduces the visible workflow to three large full-width buttons: Add shots, Stitch/Preview, and Save.
- The middle button changes from Stitch to Preview after a result exists; successful stitching opens the preview, tapping the preview closes it, and Preview reopens it without rerunning Rust.
- Selecting new screenshots clears the old result and restores the middle button to Stitch.
- The Milestone 7E Android debug build and physical-phone verification succeed: the simplified UI looks correct and the stitch, preview toggle, and save workflow work as intended.
- Git for Windows 2.55.0 is installed, and the workspace is initialized on branch main with origin https://github.com/kutsusojo/LongScreenshot.git.
- Root ignore rules exclude machine-specific Android configuration, generated build outputs and native libraries, Rust target output, IDE state, and local backup archives.
- The reviewed initial source snapshot was committed as 3261d07 and pushed successfully to GitHub.

## Not yet completed

- share/export UI

## Next checkpoint

Milestone 7F - Add Android sharing for the saved stitched PNG without changing
the three-button primary workflow.
