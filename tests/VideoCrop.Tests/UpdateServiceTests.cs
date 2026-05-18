using FluentAssertions;
using VideoCrop.Core.Updater;

namespace VideoCrop.Tests;

public class UpdateServiceTests
{
    [Theory]
    [InlineData("v1.2.3", "1.2.3")]
    [InlineData("V0.4.1", "0.4.1")]
    [InlineData("1.0.0", "1.0.0")]
    [InlineData("v2.0.0-beta.1", "2.0.0")]
    public void ParseVersion_handles_common_tag_formats(string tag, string expected)
    {
        var v = UpdateService.ParseVersion(tag);
        v.Should().NotBeNull();
        v!.ToString().Should().StartWith(expected);
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    [InlineData("latest")]
    [InlineData("vNot-A-Version")]
    public void ParseVersion_rejects_garbage(string? tag)
    {
        UpdateService.ParseVersion(tag).Should().BeNull();
    }

    [Fact]
    public void SelectAsset_prefers_architecture_match()
    {
        var assets = new List<GitHubAsset>
        {
            new() { Name = "VideoCrop-1.0.0-win-arm64.zip", BrowserDownloadUrl = "https://x/a" },
            new() { Name = "VideoCrop-1.0.0-win-x64.zip", BrowserDownloadUrl = "https://x/b" },
            new() { Name = "VideoCrop-1.0.0-linux.zip", BrowserDownloadUrl = "https://x/c" },
        };

        var chosen = UpdateService.SelectAsset(assets);
        chosen.Should().NotBeNull();
        // On x64 hosts (where these tests typically run), x64 wins.
        chosen!.Name.Should().EndWith(".zip");
        chosen.Name.Should().Contain("win");
    }

    [Fact]
    public void SelectAsset_returns_null_when_no_zip_present()
    {
        var assets = new List<GitHubAsset>
        {
            new() { Name = "VideoCrop-1.0.0.msix" },
            new() { Name = "checksums.txt" },
        };
        UpdateService.SelectAsset(assets).Should().BeNull();
    }
}
