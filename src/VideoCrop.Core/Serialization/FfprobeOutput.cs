using System.Text.Json.Serialization;

namespace VideoCrop.Core.Serialization;

internal sealed class FfprobeOutput
{
    [JsonPropertyName("streams")]
    public List<FfprobeStream> Streams { get; set; } = new();

    [JsonPropertyName("format")]
    public FfprobeFormat Format { get; set; } = new();
}

internal sealed class FfprobeStream
{
    [JsonPropertyName("index")] public int Index { get; set; }
    [JsonPropertyName("codec_name")] public string? CodecName { get; set; }
    [JsonPropertyName("codec_type")] public string? CodecType { get; set; }
    [JsonPropertyName("width")] public int? Width { get; set; }
    [JsonPropertyName("height")] public int? Height { get; set; }
    [JsonPropertyName("sample_aspect_ratio")] public string? SampleAspectRatio { get; set; }
    [JsonPropertyName("display_aspect_ratio")] public string? DisplayAspectRatio { get; set; }
    [JsonPropertyName("r_frame_rate")] public string? RFrameRate { get; set; }
    [JsonPropertyName("avg_frame_rate")] public string? AvgFrameRate { get; set; }
    [JsonPropertyName("bit_rate")] public string? BitRate { get; set; }
    [JsonPropertyName("channels")] public int? Channels { get; set; }
    [JsonPropertyName("sample_rate")] public string? SampleRate { get; set; }
    [JsonPropertyName("tags")] public Dictionary<string, string>? Tags { get; set; }
    [JsonPropertyName("duration")] public string? Duration { get; set; }
}

internal sealed class FfprobeFormat
{
    [JsonPropertyName("filename")] public string? Filename { get; set; }
    [JsonPropertyName("duration")] public string? Duration { get; set; }
    [JsonPropertyName("bit_rate")] public string? BitRate { get; set; }
    [JsonPropertyName("format_name")] public string? FormatName { get; set; }
    [JsonPropertyName("format_long_name")] public string? FormatLongName { get; set; }
    [JsonPropertyName("tags")] public Dictionary<string, string>? Tags { get; set; }
}
