using FluentAssertions;
using VideoCrop.Core.Encoding;

namespace VideoCrop.Tests;

public class EncodeProgressParserTests
{
    [Fact]
    public void Parses_progress_continue_block()
    {
        var p = new EncodeProgressParser();
        p.OnLine("frame=123").Should().BeNull();
        p.OnLine("fps=58.2").Should().BeNull();
        p.OnLine("out_time_us=4920000").Should().BeNull();
        p.OnLine("speed=1.95x").Should().BeNull();
        var result = p.OnLine("progress=continue");

        result.Should().NotBeNull();
        result!.Frame.Should().Be(123);
        result.Fps.Should().BeApproximately(58.2, 0.01);
        result.OutTime.Should().BeCloseTo(TimeSpan.FromMilliseconds(4920), TimeSpan.FromMilliseconds(1));
        result.Speed.Should().BeApproximately(1.95, 0.001);
        result.IsFinished.Should().BeFalse();
    }

    [Fact]
    public void Progress_end_marks_finished()
    {
        var p = new EncodeProgressParser();
        p.OnLine("frame=900");
        p.OnLine("progress=end")!.IsFinished.Should().BeTrue();
    }

    [Fact]
    public void Ignores_unknown_keys()
    {
        var p = new EncodeProgressParser();
        p.OnLine("bitrate=2000kbps").Should().BeNull();
        p.OnLine("garbage line!!").Should().BeNull();
    }
}
