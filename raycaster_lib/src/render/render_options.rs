use nalgebra::Vector2;

const DEFAULT_RAY_STEP_QUALITY: f32 = 0.5;
const DEFAULT_RAY_STEP_FAST: f32 = 0.5;
const DEFAULT_EARLY_RAY_TERMINATION: bool = true;
const DEFAULT_EMPTY_SPACE_SKIPPING: bool = true;

/// Renderer settings
///
/// Controls the resolution and optimisations renderer uses
#[derive(Clone, Copy)]
pub struct RenderOptions {
    /// Resolution of rendered image
    pub resolution: Vector2<u16>,
    /// Use ERT (Early Ray Termination)
    pub early_ray_termination: bool,
    /// Use Empty Space Skipping
    pub empty_space_skipping: bool,
    /// Length of sampling step in high quality render mode
    pub ray_step_quality: f32,
    /// Length of sampling step in fast render mode
    pub ray_step_fast: f32,
}

impl RenderOptions {
    /// Constructor
    pub fn new(
        resolution: Vector2<u16>,
        early_ray_termination: bool,
        empty_space_skipping: bool,
        ray_step_quality: f32,
        ray_step_fast: f32,
    ) -> Self {
        Self {
            resolution,
            early_ray_termination,
            empty_space_skipping,
            ray_step_quality,
            ray_step_fast,
        }
    }

    /// Use Builder pattern to construct `RenderOptions`
    pub fn builder() -> RenderOptionsBuilder {
        RenderOptionsBuilder::default()
    }
}

/// Builder for `RenderOptions`
#[derive(Default, Clone, Copy)]
pub struct RenderOptionsBuilder {
    /// Resolution of rendered image
    resolution: Option<Vector2<u16>>,
    /// Use ERT (Early Ray Termination)
    early_ray_termination: Option<bool>,
    /// Use Empty Space Skipping
    empty_space_skipping: Option<bool>,
    /// Length of sampling step in high quality render mode
    ray_step_quality: Option<f32>,
    /// Length of sampling step in fast render mode
    ray_step_fast: Option<f32>,
}

impl RenderOptionsBuilder {
    /// New builder
    ///
    /// Default values are `0.5` for sample steps and true for all optimisations
    pub fn new() -> Self {
        Default::default()
    }

    /// Set resolution of rendered image
    ///
    /// Required to successfully build `RenderOptions`
    pub fn resolution(&mut self, resolution: Vector2<u16>) -> &mut Self {
        self.resolution = Some(resolution);
        self
    }

    /// Set ERT optimisation on or off
    pub fn early_ray_termination(&mut self, on: bool) -> &mut Self {
        self.early_ray_termination = Some(on);
        self
    }

    /// Set empty space skipping optimisation on or off
    pub fn empty_space_skipping(&mut self, on: bool) -> &mut Self {
        self.empty_space_skipping = Some(on);
        self
    }

    /// Set sample step length for high quality rendering
    pub fn ray_step_quality(&mut self, step: f32) -> &mut Self {
        self.ray_step_quality = Some(step);
        self
    }

    /// Set sample step length for low quality (fast) rendering
    pub fn ray_step_fast(&mut self, step: f32) -> &mut Self {
        self.ray_step_fast = Some(step);
        self
    }

    /// Build the options
    ///
    /// Fails only if resolution is not specified
    pub fn build(&self) -> Option<RenderOptions> {
        let resolution = match self.resolution {
            Some(r) => r,
            None => return None,
        };

        let EARLY_RAY_TERMINATION = self
            .early_ray_termination
            .unwrap_or(DEFAULT_EARLY_RAY_TERMINATION);
        let empty_space_skipping = self
            .empty_space_skipping
            .unwrap_or(DEFAULT_EMPTY_SPACE_SKIPPING);
        let ray_step_quality = self.ray_step_quality.unwrap_or(DEFAULT_RAY_STEP_QUALITY);
        let ray_step_fast = self.ray_step_fast.unwrap_or(DEFAULT_RAY_STEP_FAST);

        Some(RenderOptions::new(
            resolution,
            EARLY_RAY_TERMINATION,
            empty_space_skipping,
            ray_step_quality,
            ray_step_fast,
        ))
    }

    /// Build the options
    ///
    /// Crashes if resolution is not specified
    /// For safe variant, see `build` method
    pub fn build_unchecked(&self) -> RenderOptions {
        let resolution = self
            .resolution
            .expect("Building render options failed. Resolution not specified.");

        let early_ray_termination = self
            .early_ray_termination
            .unwrap_or(DEFAULT_EARLY_RAY_TERMINATION);
        let empty_space_skipping = self
            .empty_space_skipping
            .unwrap_or(DEFAULT_EMPTY_SPACE_SKIPPING);
        let ray_step_quality = self.ray_step_quality.unwrap_or(DEFAULT_RAY_STEP_QUALITY);
        let ray_step_fast = self.ray_step_fast.unwrap_or(DEFAULT_RAY_STEP_FAST);

        RenderOptions::new(
            resolution,
            early_ray_termination,
            empty_space_skipping,
            ray_step_quality,
            ray_step_fast,
        )
    }
}
