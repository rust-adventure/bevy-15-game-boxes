#import bevy_pbr::{
    // pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}
// replacement for the above `pbr_input_from_standard_material` with customizations
// TODO: replace this by using custom attributes for vertex colors instead
#import bevy_segment_outline::bevy_pbr::pbr_fragment::pbr_input_from_standard_material;

#import bevy_pbr::view_transformations::depth_ndc_to_view_z

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

// struct MyExtendedMaterial {
//     quantize_steps: u32,
// }

// @group(2) @binding(100)
// var<uniform> my_extended_material: MyExtendedMaterial;

@group(2) @binding(100) var<storage, read> sdfs: array<vec4<f32>>;
@group(2) @binding(101) var decals_color_texture: texture_2d<f32>;
@group(2) @binding(102) var decals_color_sampler: sampler;
@group(2) @binding(103) var grit_color_texture: texture_2d<f32>;
@group(2) @binding(104) var grit_color_sampler: sampler;
@group(2) @binding(105)
var vertex_id_material: texture_storage_2d<rgba8unorm, read_write>;

@fragment
fn fragment(
    #ifdef MULTISAMPLED
    @builtin(sample_index) sample_index: u32,
    #endif
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    #ifndef MULTISAMPLED
    let sample_index = 0u;
    #endif

    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    #ifdef VERTEX_COLORS
    let depth = bevy_pbr::prepass_utils::prepass_depth(in.position, sample_index);
    if depth <= in.position.z {
        // write vertex color to storage texture
        let mipLevel = 0;
        textureStore(
            vertex_id_material,
            vec2u(in.position.xy),
            in.color
            // vec4(in.color.r, 0., 0.,1.)
        );
    }
    #endif

    // we can optionally modify the input before lighting and alpha_discard is applied
    // pbr_input.material.base_color.b = pbr_input.material.base_color.r;
    var distance_to_nearest: f32 = 1000.;
    for (var i = 0; i < i32(arrayLength(&sdfs)); i++) {
            distance_to_nearest = min(
                distance_to_nearest,
                // - max_distance (3 for main game, 1 for example)
                length(in.world_position.xyz - sdfs[i].xyz) - 3.0
            );
    }
    if distance_to_nearest <= 10.0 {
        let can = 3. * distance_to_nearest * textureSample(grit_color_texture, grit_color_sampler, in.world_position.xz / 3.).r;
        let color = mix(
            pbr_input.material.base_color,
            vec4(0.8,0.8,0.8,pbr_input.material.base_color.a),
            saturate(can)
        );
        pbr_input.material.base_color.r = color.r;
        pbr_input.material.base_color.g = color.g;
        pbr_input.material.base_color.b = color.b;

    } else {
        pbr_input.material.base_color.r = 0.8;
        pbr_input.material.base_color.g = 0.8;
        pbr_input.material.base_color.b = 0.8;
    }


    // pbr_input.material.base_color = vec4(distance_to_nearest,distance_to_nearest,distance_to_nearest,1.);

    // pbr_input.material.base_color = vec4(sdfs[0].x,sdfs[0].y,sdfs[0].z,1.);
    // pbr_input.material.base_color = vec4(in.position.x/10.,in.position.y/10.,in.position.z/10.,1.);

    // pbr_input.material.base_color = vec4(in.world_position.x,in.world_position.y,in.world_position.z,1.);
    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // we can optionally modify the lit color before post-processing is applied
    // out.color = vec4<f32>(vec4<u32>(out.color * f32(my_extended_material.quantize_steps))) / f32(my_extended_material.quantize_steps);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    // we can optionally modify the final result here
    // out.color = out.color * 2.0;
#endif

    return out;
}