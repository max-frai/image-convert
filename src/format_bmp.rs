use crate::{
    compute_output_size_sharpen, fetch_magic_wand, magick_rust::{bindings, PixelWand},
    starts_ends_with_caseless::EndsWithCaseless, ColorName, ImageConfig, ImageResource, InterlaceType,
};

#[derive(Debug)]
/// The output config of a BMP image.
pub struct BMPConfig {
    /// The width of the output image. `0` means the original width.
    pub width: u16,
    /// The height of the output image. `0` means the original height.
    pub height: u16,
    /// Only shrink the image, not to enlarge it.
    pub shrink_only: bool,
    /// The higher the sharper. A negative value means auto adjustment.
    pub sharpen: f64,
    /// The color is used for fill up the alpha background.
    pub background_color: Option<ColorName>,
}

impl BMPConfig {
    /// Create a `BMPConfig` instance with default values.
    /// ```rust,ignore
    /// BMPConfig {
    ///     width: 0u16,
    ///     height: 0u16,
    ///     shrink_only: true,
    ///     sharpen: -1f64,
    /// }
    /// ```
    #[inline]
    pub fn new() -> BMPConfig {
        BMPConfig {
            width: 0u16,
            height: 0u16,
            shrink_only: true,
            sharpen: -1f64,
            background_color: None,
        }
    }
}

impl Default for BMPConfig {
    #[inline]
    fn default() -> Self {
        BMPConfig::new()
    }
}

impl ImageConfig for BMPConfig {
    fn get_width(&self) -> u16 {
        self.width
    }

    fn get_height(&self) -> u16 {
        self.height
    }

    fn get_sharpen(&self) -> f64 {
        self.sharpen
    }

    fn is_shrink_only(&self) -> bool {
        self.shrink_only
    }
}

/// Convert an image to a BMP image.
pub fn to_bmp(
    output: &mut ImageResource,
    input: &ImageResource,
    config: &BMPConfig,
) -> Result<(), &'static str> {
    let (mut mw, vector) = fetch_magic_wand(input, config)?;

    if let Some(background_color) = config.background_color {
        let mut pw = PixelWand::new();
        pw.set_color(background_color.as_str())?;
        mw.set_image_background_color(&pw)?;
        mw.set_image_alpha_channel(bindings::AlphaChannelOption_RemoveAlphaChannel)?;
    }

    if !vector {
        let (width, height, sharpen) = compute_output_size_sharpen(&mw, config);

        mw.resize_image(width as usize, height as usize, bindings::FilterType_LanczosFilter);

        mw.sharpen_image(0f64, sharpen)?;
    }

    mw.profile_image("*", None)?;

    mw.set_image_compression_quality(100)?;

    mw.set_interlace_scheme(InterlaceType::LineInterlace.ordinal() as bindings::InterlaceType)?;

    mw.set_image_format("BMP")?;

    match output {
        ImageResource::Path(p) => {
            if !p.ends_with_caseless_ascii(".bmp") {
                return Err("The file extension name is not bmp.");
            }

            mw.write_image(p.as_str())?;
        }
        ImageResource::Data(b) => {
            let mut temp = mw.write_image_blob("BMP")?;
            b.append(&mut temp);
        }
        ImageResource::MagickWand(mw_2) => {
            *mw_2 = mw;
        }
    }

    Ok(())
}