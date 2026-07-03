//! build.zig — libopus 1.5.2 static library build (M2)
//!
//! Compiled with Zig 0.14.x. Produces `zig-out/lib/libopus.a` (Linux/macOS)
//! or `zig-out/lib/opus.lib` (Windows MSVC) that Rust `build.rs` links.
//!
//! Invoked automatically by `crates/opus-core/build.rs`.
//!
//! IMPORTANT: On Windows MSVC targets, libopus's `celt_fatal()` calls
//! `fprintf`, which lives in the MSVC C runtime. We must link libc so
//! the symbol resolves. `link_libc = true` handles this on all platforms
//! (Zig picks the right libc: msvcrt on Windows-msvc, glibc/musl on Linux,
//! system libc on macOS).

const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const libopus_root = "../vendor/libopus";

    const lib = b.addStaticLibrary(.{
        .name = "opus",
        .target = target,
        .optimize = optimize,
    });

    // CRITICAL: link libc so fprintf/stderr (used by celt_fatal) resolve.
    // On windows-msvc this links against msvcrt; on Linux glibc/musl; on
    // macOS the system libSystem.
    lib.linkLibC();

    // ─── libopus compile flags ────────────────────────────────────────────
    const opus_flags = &[_][]const u8{
        "-DOPUS_BUILD",
        "-DPACKAGE_VERSION=\"1.5.2\"",
        "-DUSE_ALLOCA",
        "-DHAVE_LRINTF",
        "-DHAVE_LRINT",
        "-DFLOATING_POINT",
        "-DVAR_ARRAYS",
        "-DENABLE_HARDENING",
        "-fno-math-errno",
        "-std=c99",
        "-Wno-unused-function",
        "-Wno-unused-variable",
        "-Wno-parentheses",
        "-Wno-sign-compare",
    };

    // ─── Include directories ──────────────────────────────────────────────
    lib.addIncludePath(b.path(libopus_root ++ "/include"));
    lib.addIncludePath(b.path(libopus_root ++ "/celt"));
    lib.addIncludePath(b.path(libopus_root ++ "/silk"));
    lib.addIncludePath(b.path(libopus_root ++ "/silk/float"));
    lib.addIncludePath(b.path(libopus_root ++ "/src"));
    lib.addIncludePath(b.path(libopus_root ++ "/dnn"));

    // ─── OPUS core sources ────────────────────────────────────────────────
    const opus_sources = [_][]const u8{
        "src/opus.c",
        "src/opus_decoder.c",
        "src/opus_encoder.c",
        "src/extensions.c",
        "src/opus_multistream.c",
        "src/opus_multistream_encoder.c",
        "src/opus_multistream_decoder.c",
        "src/repacketizer.c",
        "src/opus_projection_encoder.c",
        "src/opus_projection_decoder.c",
        "src/mapping_matrix.c",
        "src/analysis.c",
        "src/mlp.c",
        "src/mlp_data.c",
    };
    for (opus_sources) |src| {
        lib.addCSourceFile(.{
            .file = b.path(b.fmt("{s}/{s}", .{ libopus_root, src })),
            .flags = opus_flags,
        });
    }

    // ─── CELT sources ─────────────────────────────────────────────────────
    const celt_sources = [_][]const u8{
        "celt/bands.c",
        "celt/celt.c",
        "celt/celt_encoder.c",
        "celt/celt_decoder.c",
        "celt/cwrs.c",
        "celt/entcode.c",
        "celt/entdec.c",
        "celt/entenc.c",
        "celt/kiss_fft.c",
        "celt/laplace.c",
        "celt/mathops.c",
        "celt/mdct.c",
        "celt/modes.c",
        "celt/pitch.c",
        "celt/celt_lpc.c",
        "celt/quant_bands.c",
        "celt/rate.c",
        "celt/vq.c",
    };
    for (celt_sources) |src| {
        lib.addCSourceFile(.{
            .file = b.path(b.fmt("{s}/{s}", .{ libopus_root, src })),
            .flags = opus_flags,
        });
    }

    // ─── SILK core sources ────────────────────────────────────────────────
    const silk_sources = [_][]const u8{
        "silk/CNG.c",
        "silk/code_signs.c",
        "silk/init_decoder.c",
        "silk/decode_core.c",
        "silk/decode_frame.c",
        "silk/decode_parameters.c",
        "silk/decode_indices.c",
        "silk/decode_pulses.c",
        "silk/decoder_set_fs.c",
        "silk/dec_API.c",
        "silk/enc_API.c",
        "silk/encode_indices.c",
        "silk/encode_pulses.c",
        "silk/gain_quant.c",
        "silk/interpolate.c",
        "silk/LP_variable_cutoff.c",
        "silk/NLSF_decode.c",
        "silk/NSQ.c",
        "silk/NSQ_del_dec.c",
        "silk/PLC.c",
        "silk/shell_coder.c",
        "silk/tables_gain.c",
        "silk/tables_LTP.c",
        "silk/tables_NLSF_CB_NB_MB.c",
        "silk/tables_NLSF_CB_WB.c",
        "silk/tables_other.c",
        "silk/tables_pitch_lag.c",
        "silk/tables_pulses_per_block.c",
        "silk/VAD.c",
        "silk/control_audio_bandwidth.c",
        "silk/quant_LTP_gains.c",
        "silk/VQ_WMat_EC.c",
        "silk/HP_variable_cutoff.c",
        "silk/NLSF_encode.c",
        "silk/NLSF_VQ.c",
        "silk/NLSF_unpack.c",
        "silk/NLSF_del_dec_quant.c",
        "silk/process_NLSFs.c",
        "silk/stereo_LR_to_MS.c",
        "silk/stereo_MS_to_LR.c",
        "silk/check_control_input.c",
        "silk/control_SNR.c",
        "silk/init_encoder.c",
        "silk/control_codec.c",
        "silk/A2NLSF.c",
        "silk/ana_filt_bank_1.c",
        "silk/biquad_alt.c",
        "silk/bwexpander_32.c",
        "silk/bwexpander.c",
        "silk/debug.c",
        "silk/decode_pitch.c",
        "silk/inner_prod_aligned.c",
        "silk/lin2log.c",
        "silk/log2lin.c",
        "silk/LPC_analysis_filter.c",
        "silk/LPC_inv_pred_gain.c",
        "silk/table_LSF_cos.c",
        "silk/NLSF2A.c",
        "silk/NLSF_stabilize.c",
        "silk/NLSF_VQ_weights_laroia.c",
        "silk/pitch_est_tables.c",
        "silk/resampler.c",
        "silk/resampler_down2_3.c",
        "silk/resampler_down2.c",
        "silk/resampler_private_AR2.c",
        "silk/resampler_private_down_FIR.c",
        "silk/resampler_private_IIR_FIR.c",
        "silk/resampler_private_up2_HQ.c",
        "silk/resampler_rom.c",
        "silk/sigm_Q15.c",
        "silk/sort.c",
        "silk/sum_sqr_shift.c",
        "silk/stereo_decode_pred.c",
        "silk/stereo_encode_pred.c",
        "silk/stereo_find_predictor.c",
        "silk/stereo_quant_pred.c",
        "silk/LPC_fit.c",
    };
    for (silk_sources) |src| {
        lib.addCSourceFile(.{
            .file = b.path(b.fmt("{s}/{s}", .{ libopus_root, src })),
            .flags = opus_flags,
        });
    }

    // ─── SILK float sources ───────────────────────────────────────────────
    const silk_float_sources = [_][]const u8{
        "silk/float/apply_sine_window_FLP.c",
        "silk/float/corrMatrix_FLP.c",
        "silk/float/encode_frame_FLP.c",
        "silk/float/find_LPC_FLP.c",
        "silk/float/find_LTP_FLP.c",
        "silk/float/find_pitch_lags_FLP.c",
        "silk/float/find_pred_coefs_FLP.c",
        "silk/float/LPC_analysis_filter_FLP.c",
        "silk/float/LTP_analysis_filter_FLP.c",
        "silk/float/LTP_scale_ctrl_FLP.c",
        "silk/float/noise_shape_analysis_FLP.c",
        "silk/float/process_gains_FLP.c",
        "silk/float/regularize_correlations_FLP.c",
        "silk/float/residual_energy_FLP.c",
        "silk/float/warped_autocorrelation_FLP.c",
        "silk/float/wrappers_FLP.c",
        "silk/float/autocorrelation_FLP.c",
        "silk/float/burg_modified_FLP.c",
        "silk/float/bwexpander_FLP.c",
        "silk/float/energy_FLP.c",
        "silk/float/inner_product_FLP.c",
        "silk/float/k2a_FLP.c",
        "silk/float/LPC_inv_pred_gain_FLP.c",
        "silk/float/pitch_analysis_core_FLP.c",
        "silk/float/scale_copy_vector_FLP.c",
        "silk/float/scale_vector_FLP.c",
        "silk/float/schur_FLP.c",
        "silk/float/sort_FLP.c",
    };
    for (silk_float_sources) |src| {
        lib.addCSourceFile(.{
            .file = b.path(b.fmt("{s}/{s}", .{ libopus_root, src })),
            .flags = opus_flags,
        });
    }

    b.installArtifact(lib);

    const check_step = b.step("check", "Build libopus.a and verify");
    check_step.dependOn(&lib.step);
}
