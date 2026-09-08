use image_input::{ImageInfo, decode_image_infos};
use jni::EnvUnowned;
use jni::errors::ThrowRuntimeExAndDefault;
use jni::objects::{JByteArray, JObject, JObjectArray, JString};
use jni::sys::jint;
use overlap::{DEFAULT_MAX_ERROR, OverlapMatch, find_default_encoded_overlap};
use stitch::{stitch_default_encoded_image_sequence, stitch_default_encoded_images};

pub mod error;
pub mod image_input;
pub mod overlap;
pub mod stitch;

pub fn add(left: i32, right: i32) -> i32 {
    left + right
}

fn format_image_report(infos: &[ImageInfo]) -> String {
    let mut lines = Vec::with_capacity(infos.len() + 1);

    lines.push(format!("Image count: {}", infos.len()));

    for (index, info) in infos.iter().enumerate() {
        lines.push(format!(
            "Image {}: {} x {}",
            index + 1,
            info.width,
            info.height
        ));
    }

    lines.join("\n")
}

fn format_overlap_report(overlap: OverlapMatch) -> String {
    let reliable = if overlap.error <= DEFAULT_MAX_ERROR {
        "yes"
    } else {
        "no"
    };

    format!(
        "Best overlap height: {}\nMean error: {:.3}\nReliability limit: {:.3}\nReliable: {}",
        overlap.height, overlap.error, DEFAULT_MAX_ERROR, reliable
    )
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_example_longscreenshot_MainActivity_add<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    left: jint,
    right: jint,
) -> jint {
    add(left, right)
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_example_longscreenshot_MainActivity_inspectImages<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _this: JObject<'local>,
    encoded_images: JObjectArray<'local, JByteArray<'local>>,
) -> JString<'local> {
    unowned_env
        .with_env(|env| {
            let image_count = encoded_images.len(env)?;
            let mut owned_images = Vec::with_capacity(image_count);

            for index in 0..image_count {
                let encoded_image = encoded_images.get_element(env, index)?;
                owned_images.push(env.convert_byte_array(&encoded_image)?);
            }

            let borrowed_images: Vec<&[u8]> =
                owned_images.iter().map(|image| image.as_slice()).collect();

            let report = match decode_image_infos(&borrowed_images) {
                Ok(infos) => format_image_report(&infos),
                Err(error) => error.to_string(),
            };

            env.new_string(report)
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_example_longscreenshot_MainActivity_inspectOverlap<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _this: JObject<'local>,
    first_encoded: JByteArray<'local>,
    second_encoded: JByteArray<'local>,
) -> JString<'local> {
    unowned_env
        .with_env(|env| {
            let first_bytes = env.convert_byte_array(&first_encoded)?;
            let second_bytes = env.convert_byte_array(&second_encoded)?;

            let report = match find_default_encoded_overlap(&first_bytes, &second_bytes) {
                Ok(overlap) => format_overlap_report(overlap),
                Err(error) => error.to_string(),
            };

            env.new_string(report)
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_example_longscreenshot_MainActivity_stitchImages<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _this: JObject<'local>,
    first_encoded: JByteArray<'local>,
    second_encoded: JByteArray<'local>,
) -> JByteArray<'local> {
    unowned_env
        .with_env(|env| -> jni::errors::Result<JByteArray<'local>> {
            let first_bytes = env.convert_byte_array(&first_encoded)?;
            let second_bytes = env.convert_byte_array(&second_encoded)?;

            let png = match stitch_default_encoded_images(&first_bytes, &second_bytes) {
                Ok(png) => png,
                Err(error) => {
                    env.throw(error.to_string())?;
                    return Ok(JByteArray::default());
                }
            };

            env.byte_array_from_slice(&png)
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_example_longscreenshot_MainActivity_stitchImageSequence<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _this: JObject<'local>,
    encoded_images: JObjectArray<'local, JByteArray<'local>>,
) -> JByteArray<'local> {
    unowned_env
        .with_env(|env| -> jni::errors::Result<JByteArray<'local>> {
            let image_count = encoded_images.len(env)?;
            let mut owned_images = Vec::with_capacity(image_count);

            for index in 0..image_count {
                let encoded_image = encoded_images.get_element(env, index)?;
                owned_images.push(env.convert_byte_array(&encoded_image)?);
            }

            let borrowed_images: Vec<&[u8]> =
                owned_images.iter().map(|image| image.as_slice()).collect();

            let png = match stitch_default_encoded_image_sequence(&borrowed_images) {
                Ok(png) => png,
                Err(error) => {
                    env.throw(error.to_string())?;
                    return Ok(JByteArray::default());
                }
            };

            env.byte_array_from_slice(&png)
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_20_and_22() {
        let result = add(20, 22);
        assert_eq!(result, 42);
    }

    #[test]
    fn formats_image_report() {
        let infos = [
            ImageInfo {
                width: 1080,
                height: 2400,
            },
            ImageInfo {
                width: 1080,
                height: 2200,
            },
        ];

        assert_eq!(
            format_image_report(&infos),
            "Image count: 2\nImage 1: 1080 x 2400\nImage 2: 1080 x 2200"
        );
    }

    #[test]
    fn formats_overlap_report() {
        let overlap = OverlapMatch {
            height: 1234,
            error: 0.125,
        };

        assert_eq!(
            format_overlap_report(overlap),
            "Best overlap height: 1234\nMean error: 0.125\nReliability limit: 5.000\nReliable: yes"
        );
    }
}
