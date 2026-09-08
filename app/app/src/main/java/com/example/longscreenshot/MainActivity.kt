package com.example.longscreenshot

import android.content.ContentValues
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.provider.MediaStore
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.longscreenshot.ui.theme.LongScreenshotTheme
import java.io.IOException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class MainActivity : ComponentActivity() {
    private external fun add(left: Int, right: Int): Int
    private external fun inspectImages(encodedImages: Array<ByteArray>): String
    private external fun inspectOverlap(
        firstEncoded: ByteArray,
        secondEncoded: ByteArray,
    ): String

    private external fun stitchImages(
        firstEncoded: ByteArray,
        secondEncoded: ByteArray,
    ): ByteArray

    private external fun stitchImageSequence(
        encodedImages: Array<ByteArray>,
    ): ByteArray

    private data class SavedImage(
        val uri: Uri,
        val displayName: String,
    )

    private fun readEncodedImages(uris: List<Uri>): Array<ByteArray> =
    uris.mapIndexed { index, uri ->
        contentResolver.openInputStream(uri)?.use { input ->
            input.readBytes()
        } ?: throw IOException("Could not open image ${index + 1}")
    }.toTypedArray()

    private fun decodePreview(
        png: ByteArray,
        maxWidth: Int = 1080,
        maxHeight: Int = 4096,
    ): Bitmap? {
        val bounds = BitmapFactory.Options().apply {
            inJustDecodeBounds = true
        }

        BitmapFactory.decodeByteArray(
            png,
            0,
            png.size,
            bounds,
        )

        if (bounds.outWidth <= 0 || bounds.outHeight <= 0) {
            return null
        }

        var sampleSize = 1

        while (
            bounds.outWidth / sampleSize > maxWidth ||
            bounds.outHeight / sampleSize > maxHeight
        ) {
            sampleSize *= 2
        }

        val options = BitmapFactory.Options().apply {
            inSampleSize = sampleSize
        }

        return BitmapFactory.decodeByteArray(
            png,
            0,
            png.size,
            options,
        )
    }

    private fun savePngToPictures(png: ByteArray): SavedImage {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
            throw IOException("Saving currently requires Android 10 or newer")
        }

        val displayName = "LongScreenshot_${System.currentTimeMillis()}.png"

        val values = ContentValues().apply {
            put(MediaStore.MediaColumns.DISPLAY_NAME, displayName)
            put(MediaStore.MediaColumns.MIME_TYPE, "image/png")
            put(
                MediaStore.MediaColumns.RELATIVE_PATH,
                "${Environment.DIRECTORY_PICTURES}/LongScreenshot",
            )
            put(MediaStore.MediaColumns.IS_PENDING, 1)
        }

        val collection = MediaStore.Images.Media.getContentUri(
            MediaStore.VOLUME_EXTERNAL_PRIMARY
        )

        val outputUri = contentResolver.insert(collection, values)
            ?: throw IOException("Could not create the output image")

        try {
            contentResolver.openOutputStream(outputUri, "w")?.use { output ->
                output.write(png)
            } ?: throw IOException("Could not open the output image")

            values.clear()
            values.put(MediaStore.MediaColumns.IS_PENDING, 0)

            if (contentResolver.update(outputUri, values, null, null) == 0) {
                throw IOException("Could not publish the output image")
            }

            return SavedImage(
                uri = outputUri,
                displayName = displayName,
            )
        } catch (error: Exception) {
            try {
                contentResolver.delete(outputUri, null, null)
            } catch (_: Exception) {
                // Preserve the original save error.
            }

            throw error
        }
    }

    companion object {
        init {
            System.loadLibrary("stitcher")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        setContent {
            LongScreenshotTheme {
                var selectedUris by remember {
                    mutableStateOf<List<Uri>>(emptyList())
                }
                var readStatus by remember {
                    mutableStateOf("Images not read")
                }
                var stitchedPng by remember {
                    mutableStateOf<ByteArray?>(null)
                }
                var showPreview by remember {
                    mutableStateOf(false)
                }

                var saveStatus by remember {
                    mutableStateOf("Not saved")
                }
                var savedImageUri by remember {
                    mutableStateOf<Uri?>(null)
                }

                val coroutineScope = rememberCoroutineScope()

                val stitchedPreview by produceState<Bitmap?>(
                    initialValue = null,
                    key1 = stitchedPng,
                    key2 = showPreview,
                ) {
                    value = null

                    val png = stitchedPng

                    if (png != null && showPreview) {
                        value = withContext(Dispatchers.IO) {
                            decodePreview(png)
                        }
                    }
                }

                val photoPicker = rememberLauncherForActivityResult(
                    contract = ActivityResultContracts.PickMultipleVisualMedia(
                        maxItems = 10
                    )
                ) { uris ->
                    selectedUris = uris
                    stitchedPng = null
                    showPreview = false
                    savedImageUri = null
                    saveStatus = "Not saved"
                    readStatus = if (uris.size in 2..10) {
                        "Ready to stitch ${uris.size} screenshots"
                    } else {
                        "Select 2 to 10 screenshots"
                    }
                }

                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    Column(
                        modifier = Modifier
                            .padding(innerPadding)
                            .verticalScroll(rememberScrollState())
                            .padding(horizontal = 20.dp, vertical = 16.dp),
                        verticalArrangement = Arrangement.spacedBy(16.dp),
                    ) {
                        Text(
                            text = "Long Screenshot",
                            fontSize = 32.sp,
                            fontWeight = FontWeight.Bold,
                        )

                        Button(
                            modifier = Modifier
                                .fillMaxWidth()
                                .heightIn(min = 64.dp),
                            onClick = {
                                photoPicker.launch(
                                    PickVisualMediaRequest(
                                        ActivityResultContracts.PickVisualMedia.ImageOnly
                                    )
                                )
                            },
                        ) {
                            Text("Add shots", fontSize = 22.sp)
                        }

                        Text(
                            text = if (selectedUris.isEmpty()) {
                                "No screenshots selected"
                            } else {
                                "${selectedUris.size} screenshots selected"
                            },
                            fontSize = 18.sp,
                        )

                        Button(
                            modifier = Modifier
                                .fillMaxWidth()
                                .heightIn(min = 64.dp),
                            enabled = stitchedPng != null || selectedUris.size in 2..10,
                            onClick = {
                                if (stitchedPng != null) {
                                    showPreview = true
                                } else {
                                    val urisToRead = selectedUris

                                    coroutineScope.launch {
                                        stitchedPng = null
                                        showPreview = false
                                        savedImageUri = null
                                        saveStatus = "Not saved"
                                        readStatus = "Stitching ${urisToRead.size} images..."

                                        val (png, status) = withContext(Dispatchers.IO) {
                                            try {
                                                val encodedImages = readEncodedImages(urisToRead)
                                                val outputPng = stitchImageSequence(encodedImages)

                                                val options = BitmapFactory.Options().apply {
                                                    inJustDecodeBounds = true
                                                }

                                                BitmapFactory.decodeByteArray(
                                                    outputPng,
                                                    0,
                                                    outputPng.size,
                                                    options,
                                                )

                                                if (options.outWidth > 0 && options.outHeight > 0) {
                                                    outputPng to (
                                                        "Stitched ${encodedImages.size} images\n" +
                                                            "PNG: ${outputPng.size} bytes\n" +
                                                            "${options.outWidth} x ${options.outHeight}"
                                                    )
                                                } else {
                                                    null to "Rust returned an invalid PNG"
                                                }
                                            } catch (error: IOException) {
                                                null to (error.message ?: "Could not read images")
                                            } catch (error: SecurityException) {
                                                null to "Permission to read an image was denied"
                                            } catch (error: Exception) {
                                                null to (error.message ?: "Rust stitching failed")
                                            }
                                        }

                                        stitchedPng = png
                                        showPreview = png != null
                                        readStatus = status
                                    }
                                }
                            },
                        ) {
                            Text(
                                text = if (stitchedPng == null) "Stitch" else "Preview",
                                fontSize = 22.sp,
                            )
                        }

                        Text(
                            text = readStatus,
                            fontSize = 18.sp,
                        )

                        Button(
                            modifier = Modifier
                                .fillMaxWidth()
                                .heightIn(min = 64.dp),
                            enabled = stitchedPng != null,
                            onClick = {
                                val pngToSave = stitchedPng

                                if (pngToSave != null) {
                                    coroutineScope.launch {
                                        saveStatus = "Saving..."

                                        val (savedImage, status) = withContext(Dispatchers.IO) {
                                            try {
                                                val saved = savePngToPictures(pngToSave)

                                                saved to (
                                                    "Saved to Pictures/LongScreenshot/" +
                                                        saved.displayName
                                                )
                                            } catch (error: IOException) {
                                                null to (error.message ?: "Could not save image")
                                            } catch (error: SecurityException) {
                                                null to "Permission to save the image was denied"
                                            } catch (error: Exception) {
                                                null to (error.message ?: "Could not save image")
                                            }
                                        }

                                        savedImageUri = savedImage?.uri
                                        saveStatus = status
                                    }
                                }
                            },
                        ) {
                            Text("Save", fontSize = 22.sp)
                        }

                        Text(
                            text = saveStatus,
                            fontSize = 18.sp,
                        )

                        if (showPreview && stitchedPng != null && stitchedPreview == null) {
                            Text(
                                text = "Preparing preview...",
                                fontSize = 18.sp,
                            )
                        }

                        stitchedPreview?.let { preview ->
                            Text(
                                text = "Preview",
                                fontSize = 24.sp,
                                fontWeight = FontWeight.Bold,
                            )

                            Image(
                                bitmap = preview.asImageBitmap(),
                                contentDescription = "Stitched screenshot preview",
                                modifier = Modifier
                                    .clickable { showPreview = false }
                                    .fillMaxWidth()
                                    .aspectRatio(
                                        preview.width.toFloat() / preview.height.toFloat()
                                    ),
                                contentScale = ContentScale.FillWidth,
                            )
                        }
                    }
                }
            }
        }
    }
}
