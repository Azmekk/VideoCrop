using FluentAssertions;
using VideoCrop.Core.IO;

namespace VideoCrop.Tests;

public class OutputNamerTests : IDisposable
{
    private readonly string _dir;

    public OutputNamerTests()
    {
        _dir = Path.Combine(Path.GetTempPath(), "VideoCropNamerTests_" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(_dir);
    }

    public void Dispose()
    {
        try { Directory.Delete(_dir, recursive: true); } catch { }
    }

    [Fact]
    public void First_call_uses_VideoCrop_suffix()
    {
        var result = OutputNamer.GetNextAvailable("C:/in/movie.mp4", _dir, ".mp4");
        Path.GetFileName(result).Should().Be("movie_VideoCrop.mp4");
    }

    [Fact]
    public void Collision_with_base_uses_index_2()
    {
        File.WriteAllText(Path.Combine(_dir, "movie_VideoCrop.mp4"), "");
        var result = OutputNamer.GetNextAvailable("C:/in/movie.mp4", _dir, ".mp4");
        Path.GetFileName(result).Should().Be("movie_VideoCrop2.mp4");
    }

    [Fact]
    public void Collision_with_base_and_index_2_uses_index_3()
    {
        File.WriteAllText(Path.Combine(_dir, "movie_VideoCrop.mp4"), "");
        File.WriteAllText(Path.Combine(_dir, "movie_VideoCrop2.mp4"), "");
        var result = OutputNamer.GetNextAvailable("C:/in/movie.mp4", _dir, ".mp4");
        Path.GetFileName(result).Should().Be("movie_VideoCrop3.mp4");
    }

    [Fact]
    public void Source_already_ending_in_VideoCrop_does_not_double_suffix()
    {
        var result = OutputNamer.GetNextAvailable("C:/in/clip_VideoCrop.mp4", _dir, ".mp4");
        Path.GetFileName(result).Should().Be("clip_VideoCrop.mp4");
    }

    [Fact]
    public void Source_already_ending_in_VideoCrop3_does_not_double_suffix()
    {
        var result = OutputNamer.GetNextAvailable("C:/in/clip_VideoCrop3.mp4", _dir, ".mp4");
        Path.GetFileName(result).Should().Be("clip_VideoCrop.mp4");
    }

    [Fact]
    public void Unicode_filename_handled()
    {
        var result = OutputNamer.GetNextAvailable("C:/in/日本語.mp4", _dir, ".mp4");
        Path.GetFileName(result).Should().Be("日本語_VideoCrop.mp4");
    }

    [Fact]
    public void Extension_with_no_dot_works()
    {
        var result = OutputNamer.GetNextAvailable("C:/in/clip.mp4", _dir, "webm");
        Path.GetFileName(result).Should().Be("clip_VideoCrop.webm");
    }
}
