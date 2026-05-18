using FluentAssertions;
using VideoCrop.Core.Processes;

namespace VideoCrop.Tests;

public class FfprobeParserTests
{
    private const string SampleH264Json = """
    {
      "streams": [
        {
          "index": 0,
          "codec_name": "h264",
          "codec_type": "video",
          "width": 1920,
          "height": 1080,
          "sample_aspect_ratio": "1:1",
          "display_aspect_ratio": "16:9",
          "r_frame_rate": "30000/1001",
          "avg_frame_rate": "30000/1001",
          "duration": "12.345000"
        },
        {
          "index": 1,
          "codec_name": "aac",
          "codec_type": "audio",
          "channels": 2,
          "sample_rate": "48000",
          "bit_rate": "192000",
          "tags": { "language": "eng" }
        }
      ],
      "format": {
        "filename": "test.mp4",
        "duration": "12.345000",
        "bit_rate": "5000000",
        "format_name": "mov,mp4,m4a,3gp,3g2,mj2"
      }
    }
    """;

    [Fact]
    public void Parses_basic_h264_video()
    {
        var info = FfprobeRunner.ParseVideoInfo(SampleH264Json, "C:/test.mp4");

        info.Width.Should().Be(1920);
        info.Height.Should().Be(1080);
        info.VideoCodec.Should().Be("h264");
        info.SampleAspectRatioNum.Should().Be(1);
        info.SampleAspectRatioDen.Should().Be(1);
        info.Fps.Should().BeApproximately(29.97, 0.01);
        info.Duration.Should().BeCloseTo(TimeSpan.FromMilliseconds(12345), TimeSpan.FromMilliseconds(1));
        info.BitrateBitsPerSecond.Should().Be(5_000_000);
        info.AudioStreams.Should().HaveCount(1);
        info.AudioStreams[0].Codec.Should().Be("aac");
        info.AudioStreams[0].Language.Should().Be("eng");
        info.AudioStreams[0].SampleRate.Should().Be(48000);
        info.DisplayAspectRatio.Should().BeApproximately(16.0 / 9.0, 0.01);
    }

    [Fact]
    public void Anamorphic_sar_affects_display_aspect_ratio()
    {
        var json = SampleH264Json.Replace("\"sample_aspect_ratio\": \"1:1\"", "\"sample_aspect_ratio\": \"4:3\"");

        var info = FfprobeRunner.ParseVideoInfo(json, "C:/anamorphic.mp4");

        // 1920 * 4/3 / 1080 ≈ 2.37 (cinemascope-ish)
        info.DisplayAspectRatio.Should().BeApproximately(1920.0 * 4 / 3 / 1080, 0.01);
    }

    [Fact]
    public void Missing_video_stream_throws()
    {
        const string audioOnly = """
        {
          "streams": [
            { "index": 0, "codec_name": "aac", "codec_type": "audio", "channels": 2, "sample_rate": "44100" }
          ],
          "format": { "duration": "5.0", "format_name": "mp3" }
        }
        """;

        Action act = () => FfprobeRunner.ParseVideoInfo(audioOnly, "C:/x.mp3");
        act.Should().Throw<InvalidDataException>();
    }

    [Fact]
    public void Empty_audio_streams_handled()
    {
        const string videoOnly = """
        {
          "streams": [
            { "index": 0, "codec_name": "vp9", "codec_type": "video", "width": 1280, "height": 720, "r_frame_rate": "24/1" }
          ],
          "format": { "duration": "3.0", "format_name": "matroska,webm" }
        }
        """;

        var info = FfprobeRunner.ParseVideoInfo(videoOnly, "C:/silent.webm");
        info.AudioStreams.Should().BeEmpty();
        info.VideoCodec.Should().Be("vp9");
        info.Fps.Should().BeApproximately(24, 0.01);
    }
}
