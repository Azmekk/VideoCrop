using FluentAssertions;
using VideoCrop.Core.Encoding;
using VideoCrop.Core.Models;

namespace VideoCrop.Tests;

public class EncodeCommandBuilderTests
{
    private static EncodeJob BuildBaseJob(
        CutSpec? cut = null,
        CropSpec? crop = null,
        ResizeSpec? resize = null,
        CompressionSpec? comp = null)
    {
        return new EncodeJob(
            InputPath: @"C:\in\movie.mp4",
            OutputPath: @"C:\out\movie_VideoCrop.mp4",
            Cut: cut,
            Crop: crop,
            Resize: resize,
            Compression: comp ?? CompressionSpec.WebMediumDefault(),
            SourceDuration: TimeSpan.FromSeconds(30));
    }

    [Fact]
    public void Base_encode_includes_codec_and_audio()
    {
        var invocation = EncodeCommandBuilder.Build(BuildBaseJob());
        var args = invocation.Arguments;
        args.Should().Contain("-i");
        args.Should().Contain(@"C:\in\movie.mp4");
        args.Should().Contain("-c:v");
        args.Should().Contain("libx264");
        args.Should().Contain("-crf");
        args.Should().Contain("23");
        args.Should().Contain("-c:a");
        args.Should().Contain("aac");
        args.Should().Contain("-b:a");
        args.Should().Contain("128k");
        args.Should().Contain("-pix_fmt");
        args.Should().Contain("yuv420p");
        args.Should().Contain("-progress");
        args.Should().Contain("pipe:1");
    }

    [Fact]
    public void Crop_only_produces_crop_filter()
    {
        var crop = new CropSpec(10, 20, 1600, 900);
        var invocation = EncodeCommandBuilder.Build(BuildBaseJob(crop: crop));
        var filter = EncodeCommandBuilder.BuildVideoFilter(BuildBaseJob(crop: crop));
        filter.Should().Be("crop=1600:900:10:20");
        invocation.Arguments.Should().Contain("-vf");
        invocation.Arguments.Should().Contain("crop=1600:900:10:20");
    }

    [Fact]
    public void Resize_only_locked_uses_lanczos()
    {
        var resize = new ResizeSpec(1280, 720, AspectLocked: true);
        var filter = EncodeCommandBuilder.BuildVideoFilter(BuildBaseJob(resize: resize));
        filter.Should().Be("scale=1280:-2:flags=lanczos");
    }

    [Fact]
    public void Resize_unlocked_uses_explicit_dimensions()
    {
        var resize = new ResizeSpec(1280, 800, AspectLocked: false);
        var filter = EncodeCommandBuilder.BuildVideoFilter(BuildBaseJob(resize: resize));
        filter.Should().Be("scale=1280:800:flags=lanczos");
    }

    [Fact]
    public void Crop_and_resize_are_ordered_crop_then_scale()
    {
        var crop = new CropSpec(0, 0, 1600, 900);
        var resize = new ResizeSpec(1280, 720, AspectLocked: true);
        var filter = EncodeCommandBuilder.BuildVideoFilter(BuildBaseJob(crop: crop, resize: resize));
        filter.Should().Be("crop=1600:900:0:0,scale=1280:-2:flags=lanczos");
    }

    [Fact]
    public void Neither_crop_nor_resize_yields_null_filter()
    {
        var filter = EncodeCommandBuilder.BuildVideoFilter(BuildBaseJob());
        filter.Should().BeNull();
    }

    [Fact]
    public void Resize_rounds_odd_dimensions_to_even()
    {
        var resize = new ResizeSpec(1281, 723, AspectLocked: false);
        var filter = EncodeCommandBuilder.BuildVideoFilter(BuildBaseJob(resize: resize));
        filter.Should().Be("scale=1280:722:flags=lanczos");
    }

    [Fact]
    public void Cut_inaccurate_emits_ss_before_input()
    {
        var cut = new CutSpec(TimeSpan.FromSeconds(2), TimeSpan.FromSeconds(7), Accurate: false);
        var invocation = EncodeCommandBuilder.Build(BuildBaseJob(cut: cut));
        var args = invocation.Arguments.ToList();
        var ssIdx = args.IndexOf("-ss");
        var iIdx = args.IndexOf("-i");
        ssIdx.Should().BeGreaterThan(-1);
        iIdx.Should().BeGreaterThan(ssIdx);
    }

    [Fact]
    public void Cut_accurate_emits_ss_after_input()
    {
        var cut = new CutSpec(TimeSpan.FromSeconds(2), TimeSpan.FromSeconds(7), Accurate: true);
        var invocation = EncodeCommandBuilder.Build(BuildBaseJob(cut: cut));
        var args = invocation.Arguments.ToList();
        var iIdx = args.IndexOf("-i");
        var ssIdx = args.IndexOf("-ss");
        iIdx.Should().BeGreaterThan(-1);
        ssIdx.Should().BeGreaterThan(iIdx);
    }

    [Fact]
    public void Av1_uses_libsvtav1_with_preset_number()
    {
        var spec = new CompressionSpec(VideoCodec.Av1, RateControl.Crf, 28, null, null, SpeedPreset.Medium, AudioCodec.Aac, 128, PixelFormat.Yuv420p);
        var invocation = EncodeCommandBuilder.Build(BuildBaseJob(comp: spec));
        invocation.Arguments.Should().Contain("libsvtav1");
        invocation.Arguments.Should().Contain("-preset");
        invocation.Arguments.Should().Contain("6");
        invocation.Arguments.Should().Contain("-crf");
        invocation.Arguments.Should().Contain("28");
        invocation.Arguments.Should().Contain("-b:v");
        invocation.Arguments.Should().Contain("0");
    }

    [Fact]
    public void Two_pass_emits_pass1_and_pass2_with_stats_prefix()
    {
        var spec = new CompressionSpec(VideoCodec.H264, RateControl.TwoPassTargetSize, 23, TargetBitrateBps: 2_000_000, null, SpeedPreset.Medium, AudioCodec.Aac, 128, PixelFormat.Yuv420p);
        var job = BuildBaseJob(comp: spec);
        var passes = EncodeCommandBuilder.BuildTwoPass(job, @"C:\tmp\videocrop_pass");

        passes.Pass1.Should().Contain("-pass");
        passes.Pass1.Should().Contain("1");
        passes.Pass1.Should().Contain("-an");
        passes.Pass2.Should().Contain("-pass");
        passes.Pass2.Should().Contain("2");
    }

    [Fact]
    public void ComputeTargetBitrate_accounts_for_audio()
    {
        // 8MB target, 10 seconds, 128k audio → leaves ~6.2 Mbit for video
        var bps = EncodeCommandBuilder.ComputeTargetBitrate(8 * 1024 * 1024, TimeSpan.FromSeconds(10), 128);
        bps.Should().BeGreaterThan(5_000_000).And.BeLessThan(7_000_000);
    }
}
