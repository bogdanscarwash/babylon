# A100-Optimized Settings

## Core Configuration
- Base Resolution: Up to 2048x2048 base generation
- Batch Size: 4-6 images simultaneously
- Batch Count: 4-6 variations per prompt
- VAE: Keep in VRAM, no tiling needed
- Sequential CPU Offload: Disabled

## Performance Settings
- Sampling Steps: 
  * Initial Pass: 50 steps
  * Hires Fix: 25-30 steps
  * Total: 75-80 steps (A100 can handle higher step counts efficiently)
- Sampling Method: DPM++ 3M SDE (full precision)
- CFG Scale: 7-8 for photorealistic outputs
- Clip Skip: 1 (SDXL optimal)

## Hires Fix Configuration
- Enable: Yes
- Upscaler: Latent
- Upscale Factor: 2x
- Target Resolutions:
  * Landscape: 3072x2048 or 4096x2304
  * Portrait: 2048x3072
  * Square: 2560x2560
- Denoise Strength: 0.45
- Upscale Mode: Crop and Resize

## Refiner Settings
- Model: sd_xl_refiner_1.0.safetensors
- Switch At: 0.8
- Keep in VRAM: Yes
- Batch Processing: Parallel with base model

## Resolution Guidelines by Scene Type
1. Urban Landscapes: 3072x2048
2. Interior Scenes: 2560x2048
3. Combat Scenes: 3072x1728 (cinematic ratio)
4. Detailed Architecture: 2816x2816

## Memory Utilization
- Keep all models in VRAM
- Enable parallel processing
- No VAE tiling needed
- Full precision operations
- Cross-attention optimization: Enabled

## Batch Processing
For your combat/urban scene prompts:
- Generate 4-6 variations simultaneously
- Run 2-3 batches per prompt
- Total of 8-18 images per prompt for selection

## Quality Optimizations
- Enable full precision (no half precision needed)
- Maximum attention resolution
- No memory optimizations required
- Maximum cross-attention resolution
- Enable high-resolution VAE processing

# Performance Notes
1. The A100 can handle maximum resolution outputs without tiling
2. Run multiple batches in parallel for faster iteration
3. Use maximum step counts for highest quality
4. Enable all quality features without memory constraints
5. Take advantage of tensor cores for faster processing
6. Enable maximum attention resolution
7. Use full VAE precision

# Recommended Workflow
1. Initial batch at 2048x2048 (4-6 variations)
2. Upscale best results to 4096x2304
3. Run refined passes on selected outputs
4. Generate variations with different seeds
5. Parallel process multiple prompts

The A100 removes virtually all technical limitations, so focus on:
- Maximum quality settings
- Parallel batch processing
- Higher resolution outputs
- Multiple variations per prompt
- Full precision operations