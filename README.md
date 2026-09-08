# LongScreenshot

[English](#english) | [日本語](#日本語)

## English

LongScreenshot is a small Android application that combines 2–10 vertically
overlapping screenshots into one long PNG image.

This is primarily a **Rust learning project**. Similar applications already
exist, but this project is being built incrementally to learn practical Rust,
Android integration, image processing, error handling, and JNI while keeping
the application runnable after every checkpoint.

### Current status

Milestones 1–6 are complete. Milestone 7, the Android result UI, is in
progress.

Working on a physical Android phone:

- select 2–10 screenshots with Android Photo Picker;
- preserve the selected top-to-bottom order;
- pass encoded image bytes from Kotlin to Rust through JNI;
- decode PNG and JPEG images in Rust;
- detect overlaps between consecutive screenshots;
- remove duplicated overlap regions and stitch all images;
- return a PNG from Rust to Android;
- display a bounded preview;
- close and reopen the preview without stitching again; and
- save the result to `Pictures/LongScreenshot` on Android 10 or newer.

Android sharing/export is the next unfinished checkpoint. See
[PROJECT_STATUS.md](PROJECT_STATUS.md) for the detailed development history.

### User workflow

The application has three primary buttons:

1. **Add shots** — select screenshots in top-to-bottom order.
2. **Stitch** — run overlap detection and stitching in Rust. After a result is
   available, this button becomes **Preview**.
3. **Save** — publish the stitched PNG to `Pictures/LongScreenshot`.

After stitching, tap the preview image to close it. Press **Preview** to reopen
the existing result without rerunning Rust. Selecting new screenshots clears
the previous in-memory result.

### How it works

```text
Android Photo Picker
        |
        v
Kotlin reads ordered encoded images as ByteArray values
        |
        v
JNI copies the Java byte arrays into Rust-owned Vec<u8> buffers
        |
        v
Rust decodes images and detects consecutive overlaps
        |
        v
Rust allocates the final RGBA image once, crops duplicates, and encodes PNG
        |
        v
JNI returns the PNG ByteArray to Kotlin
        |
        v
Compose preview and MediaStore save
```

Overlap candidates are ranked with sampled grayscale mean absolute error and
then checked at full resolution. The default policy ignores vertical edge
bands that may contain fixed Android status/navigation bars. Internal stitch
seams are moved into the trusted overlap region so fixed bottom bars from
earlier screenshots are replaced by matching content from the following
screenshot.

### How much overlap is enough?

The current algorithm searches overlap heights starting at **20% of the
shorter screenshot's height**. An actual overlap smaller than that is outside
the search range and will not be detected by the default policy.

For normal use, aim for approximately **30–50% overlap** between each pair of
consecutive screenshots. In practice, scroll by about 50–70% of the visible
screen height, leaving roughly one-third to one-half of the previous content
visible in the next screenshot.

For screenshots that are 2712 pixels tall:

- algorithmic minimum: about 542 pixels (20%);
- recommended overlap: about 814–1356 pixels (30–50%); and
- physically tested example: 1040 pixels, about 38%.

The shared area should contain distinctive, stable content such as text,
lines, or image details. A large overlap made mostly of blank space, repeating
patterns, animations, or changing content can still be unreliable. More
overlap usually improves the available evidence, but it also requires more
screenshots and does not compensate for unstable content.

### Project structure

```text
LongScreenshot/
├── app/                         Android Studio project
│   └── app/                     Android application module
├── stitcher/                    Rust cdylib crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs               Rust API and JNI entry points
│       ├── error.rs             Understandable error types
│       ├── image_input.rs       PNG/JPEG decoding and image representations
│       ├── overlap.rs           Overlap scoring and detection
│       └── stitch.rs            Cropping, stitching, and PNG encoding
├── AGENTS.md                    Incremental learning/development rules
└── PROJECT_STATUS.md            Detailed checkpoint history
```

The Rust crate uses `crate-type = ["cdylib"]`, producing
`libstitcher.so`. Android packages this shared library under the
`arm64-v8a` ABI. The corresponding Rust compilation target is
`aarch64-linux-android`.

### Requirements

The currently verified development environment is:

- Windows AMD64;
- Android Studio with Android SDK and Platform-Tools;
- Android NDK r29 (`29.0.14206865`);
- Rust stable with `rustfmt` and Clippy;
- Rust target `aarch64-linux-android`;
- `cargo-ndk` 4.1.2; and
- an ARM64 Android device or emulator.

The Android application has `minSdk = 24`. Saving through the current
MediaStore implementation requires Android 10/API 29 or newer.

### Build and test on Windows

The generated native library is intentionally not committed. Build it before
building the Android application.

From the repository root in PowerShell:

```powershell
$env:ANDROID_NDK_HOME = "$env:LOCALAPPDATA\Android\Sdk\ndk\29.0.14206865"

Set-Location .\stitcher
cargo fmt -- --check
cargo test
cargo clippy -- -D warnings
cargo ndk --target arm64-v8a --platform 24 build

New-Item `
  -ItemType Directory `
  -Force `
  -Path ..\app\app\src\main\jniLibs\arm64-v8a | Out-Null

Copy-Item `
  -LiteralPath .\target\aarch64-linux-android\debug\libstitcher.so `
  -Destination ..\app\app\src\main\jniLibs\arm64-v8a\libstitcher.so

Set-Location ..\app
.\gradlew.bat assembleDebug
```

The debug APK is written to:

```text
app/app/build/outputs/apk/debug/app-debug.apk
```

The Rust suite currently contains 49 passing host tests. The Rust crate has
also been verified by Clippy with warnings denied and cross-compiled for
Android ARM64 with NDK r29.

### Version 1 assumptions

- screenshots have identical widths and scale;
- screenshots are selected in correct top-to-bottom order;
- scrolling is vertical with approximately zero horizontal displacement; and
- consecutive screenshots contain enough stable overlap, preferably 30–50%
  (see [How much overlap is enough?](#how-much-overlap-is-enough)).

### Non-goals for version 1

- automatic screen capture or scrolling;
- machine learning or OpenCV;
- automatic screenshot ordering; and
- robust handling of difficult fixed headers, animations, or changing page
  content.

No release APK or app-store release is provided yet.

---

## 日本語

LongScreenshot は、縦方向に重なり合う 2～10 枚のスクリーンショットを、
1 枚の長い PNG 画像に結合する小さな Android アプリです。

このプロジェクトの主目的は **Rust の学習**です。同様の機能を持つアプリは
すでに存在しますが、このプロジェクトでは、各チェックポイントでアプリを
実行可能な状態に保ちながら、Rust、Android 連携、画像処理、エラー処理、JNI を
段階的に学ぶことを重視しています。

### 現在の状態

Milestone 1～6 は完了し、Android の結果 UI を作る Milestone 7 を進行中です。

実機 Android 端末で確認済みの機能:

- Android Photo Picker で 2～10 枚の画像を選択する;
- 選択された上から下への順序を保持する;
- Kotlin から JNI を通して Rust に画像データを渡す;
- Rust で PNG/JPEG をデコードする;
- 隣接する元画像どうしの重なりを検出する;
- 重複部分を除去して全画像を結合する;
- Rust で生成した PNG を Android に返す;
- メモリ使用量を抑えたプレビューを表示する;
- 再結合せずにプレビューを閉じたり開いたりする; および
- Android 10 以降で `Pictures/LongScreenshot` に結果を保存する。

次の未完了チェックポイントは Android の共有・エクスポート機能です。詳細な
開発履歴は [PROJECT_STATUS.md](PROJECT_STATUS.md) を参照してください。

### 操作方法

アプリの主要なボタンは 3 つです。

1. **Add shots** — スクリーンショットを上から下の順番で選択します。
2. **Stitch** — Rust で重なり検出と結合を実行します。結果ができると、この
   ボタンは **Preview** に変わります。
3. **Save** — 結合した PNG を `Pictures/LongScreenshot` に保存します。

結合後、プレビュー画像を 1 回タップすると閉じます。**Preview** を押すと Rust
で再処理せずに同じ結果を開きます。新しい画像を選ぶと、以前のメモリ上の結果は
消去されます。

### 処理の流れ

```text
Android Photo Picker
        |
        v
Kotlin が選択順に画像を ByteArray として読み込む
        |
        v
JNI 境界で Java の配列を Rust 所有の Vec<u8> にコピーする
        |
        v
Rust が画像をデコードし、隣接画像の重なりを検出する
        |
        v
Rust が最終 RGBA 画像を一度だけ確保し、重複を除去して PNG にする
        |
        v
JNI が PNG の ByteArray を Kotlin に返す
        |
        v
Compose でプレビューし、MediaStore で保存する
```

重なり候補は、グレースケール画像を間引いて平均絶対誤差を計算することで順位を
付け、選ばれた候補をフル解像度で再確認します。既定の判定では、Android の固定
ステータスバーやナビゲーションバーが入りやすい上下端を比較から除外します。
さらに内部の継ぎ目を信頼できる重なり領域へ移動し、前の画像に含まれる固定
ボトムバーを、次の画像の一致する内容で置き換えます。

### どのくらいの重なりが必要か

現在のアルゴリズムは、短い方のスクリーンショットの高さに対して **20% 以上**の
重なりを検索します。実際の重なりが 20% 未満の場合、既定の検索範囲外になるため
検出できません。

通常は、隣接する各画像に **30～50% 程度の重なり**を残すことを推奨します。
画面の高さの約 50～70% だけスクロールし、前の画面内容の約 3 分の 1～半分が
次のスクリーンショットにも残るように撮影してください。

高さ 2712 ピクセルのスクリーンショットの場合:

- アルゴリズム上の最小値: 約 542 ピクセル（20%）;
- 推奨する重なり: 約 814～1356 ピクセル（30～50%）; および
- 実機で確認した例: 1040 ピクセル（約 38%）。

重なる領域には、文字、線、画像の細部など、変化せず区別しやすい内容が必要です。
空白、繰り返し模様、アニメーション、撮影ごとに変化する内容が大部分を占める場合、
重なりが広くても正しく検出できないことがあります。重なりを増やすと比較材料は
増えますが、必要な撮影枚数も増え、不安定な内容そのものは解決できません。

### プロジェクト構成

```text
LongScreenshot/
├── app/                         Android Studio プロジェクト
│   └── app/                     Android アプリモジュール
├── stitcher/                    Rust の cdylib クレート
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs               Rust API と JNI エントリポイント
│       ├── error.rs             分かりやすいエラー型
│       ├── image_input.rs       PNG/JPEG のデコードと画像表現
│       ├── overlap.rs           重なりの評価と検出
│       └── stitch.rs            切り取り、結合、PNG エンコード
├── AGENTS.md                    段階的な学習・開発ルール
└── PROJECT_STATUS.md            チェックポイントの詳細履歴
```

Rust クレートは `crate-type = ["cdylib"]` を使用し、`libstitcher.so` を
生成します。Android 側の ABI 名は `arm64-v8a`、対応する Rust のターゲット名は
`aarch64-linux-android` です。

### 必要な環境

現在確認済みの開発環境:

- Windows AMD64;
- Android Studio、Android SDK、Platform-Tools;
- Android NDK r29 (`29.0.14206865`);
- Rust stable、`rustfmt`、Clippy;
- Rust ターゲット `aarch64-linux-android`;
- `cargo-ndk` 4.1.2; および
- ARM64 の Android 実機またはエミュレーター。

Android アプリの `minSdk` は 24 です。現在の MediaStore 保存処理は Android 10
（API 29）以降に対応しています。

### Windows でのビルドとテスト

生成物であるネイティブライブラリは Git に含めていません。Android アプリを
ビルドする前に Rust ライブラリをビルドしてください。

リポジトリのルートから PowerShell で、英語セクションの
[Build and test on Windows](#build-and-test-on-windows) にあるコマンドを実行します。

デバッグ APK の出力先:

```text
app/app/build/outputs/apk/debug/app-debug.apk
```

現在 Rust のホストテストは 49 件すべて成功しています。Clippy は警告をエラー
として扱う設定で成功し、NDK r29 を使った Android ARM64 向けクロスコンパイルも
確認済みです。

### バージョン 1 の前提

- 画像の幅と表示倍率が同じ;
- 画像を正しい上から下の順番で選択する;
- 縦スクロールで、水平方向のずれがほぼない; および
- 隣接画像に、できれば 30～50% の安定した重なりがある（
  [どのくらいの重なりが必要か](#どのくらいの重なりが必要か) を参照）。

### バージョン 1 で実装しないもの

- 画面の自動キャプチャと自動スクロール;
- 機械学習と OpenCV;
- スクリーンショットの自動並べ替え; および
- 複雑な固定ヘッダー、アニメーション、変化するページ内容への完全な対応。

現在、リリース APK やアプリストア向けリリースは提供していません。
