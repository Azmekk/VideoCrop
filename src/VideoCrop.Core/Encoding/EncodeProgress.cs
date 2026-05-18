using System.Globalization;
using System.Text.RegularExpressions;

namespace VideoCrop.Core.Encoding;

public sealed record EncodeProgress(
    long Frame,
    double Fps,
    TimeSpan OutTime,
    double Speed,
    bool IsFinished);

public sealed class EncodeProgressParser
{
    private static readonly Regex KeyValueRegex = new(@"^(?<key>[a-zA-Z_]+)=(?<value>.*)$", RegexOptions.Compiled);

    private long _frame;
    private double _fps;
    private TimeSpan _outTime;
    private double _speed;

    public EncodeProgress? OnLine(string line)
    {
        var m = KeyValueRegex.Match(line);
        if (!m.Success) return null;
        var key = m.Groups["key"].Value;
        var value = m.Groups["value"].Value.Trim();
        switch (key)
        {
            case "frame":
                if (long.TryParse(value, NumberStyles.Integer, CultureInfo.InvariantCulture, out var f)) _frame = f;
                break;
            case "fps":
                if (double.TryParse(value, NumberStyles.Float, CultureInfo.InvariantCulture, out var fps)) _fps = fps;
                break;
            case "out_time_us":
            case "out_time_ms":
                if (long.TryParse(value, NumberStyles.Integer, CultureInfo.InvariantCulture, out var usec))
                {
                    _outTime = TimeSpan.FromMicroseconds(usec);
                }
                break;
            case "out_time":
                if (TimeSpan.TryParse(value, CultureInfo.InvariantCulture, out var parsedTime))
                    _outTime = parsedTime;
                break;
            case "speed":
                var stripped = value.EndsWith("x", StringComparison.OrdinalIgnoreCase) ? value[..^1] : value;
                if (double.TryParse(stripped, NumberStyles.Float, CultureInfo.InvariantCulture, out var sp)) _speed = sp;
                break;
            case "progress":
                return new EncodeProgress(_frame, _fps, _outTime, _speed, IsFinished: value == "end");
        }
        return null;
    }
}
