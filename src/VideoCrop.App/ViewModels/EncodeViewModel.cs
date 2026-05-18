using System;
using System.Collections.Generic;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.Extensions.Logging;
using VideoCrop.Core.Encoding;
using VideoCrop.Core.IO;
using VideoCrop.Core.Models;
using VideoCrop.Core.Processes;

namespace VideoCrop.App.ViewModels;

public sealed class EncodeViewModel(IFfmpegRunner ffmpeg, ILogger<EncodeViewModel> logger) : ObservableObject
{
    private readonly SynchronizationContext? _uiContext = SynchronizationContext.Current;

    private CancellationTokenSource? _cts;
    private bool _isRunning;
    private double _progress;
    private string _statusMessage = "";
    private string _errorTail = "";
    private TimeSpan _outTime;
    private TimeSpan _totalDuration;
    private double _speed;
    private long _frame;
    private double _fps;

    public bool IsRunning { get => _isRunning; private set => SetProperty(ref _isRunning, value); }
    public double Progress { get => _progress; private set => SetProperty(ref _progress, value); }
    public string StatusMessage { get => _statusMessage; private set => SetProperty(ref _statusMessage, value); }
    public string ErrorTail { get => _errorTail; private set => SetProperty(ref _errorTail, value); }
    public TimeSpan OutTime { get => _outTime; private set => SetProperty(ref _outTime, value); }
    public double Speed { get => _speed; private set => SetProperty(ref _speed, value); }
    public long Frame { get => _frame; private set => SetProperty(ref _frame, value); }
    public double Fps { get => _fps; private set => SetProperty(ref _fps, value); }
    public TimeSpan? Eta
    {
        get
        {
            if (_speed <= 0.01 || _totalDuration <= TimeSpan.Zero) return null;
            var remaining = _totalDuration - _outTime;
            if (remaining <= TimeSpan.Zero) return TimeSpan.Zero;
            return TimeSpan.FromSeconds(remaining.TotalSeconds / _speed);
        }
    }

    public async Task<bool> RunAsync(EncodeJob job)
    {
        if (IsRunning) return false;
        _cts = new CancellationTokenSource();
        IsRunning = true;
        Progress = 0;
        StatusMessage = "Encoding…";
        ErrorTail = "";
        OutTime = TimeSpan.Zero;
        Speed = 0;
        Frame = 0;
        Fps = 0;
        _totalDuration = job.Cut?.Duration ?? job.SourceDuration;

        var stderr = new StringBuilder();
        try
        {
            if (job.Compression.RateControl == RateControl.TwoPassTargetSize)
            {
                return await RunTwoPassAsync(job, stderr).ConfigureAwait(true);
            }

            var invocation = EncodeCommandBuilder.Build(job);
            logger.LogInformation("Running ffmpeg: {Args}", string.Join(' ', invocation.Arguments));
            var exit = await ffmpeg.RunAsync(
                invocation.Arguments,
                onProgress: p => Post(() => UpdateProgress(p)),
                onStdErrLine: line =>
                {
                    if (stderr.Length < 32_000) stderr.AppendLine(line);
                    // ToString on the producer thread — same thread that
                    // appends — so the StringBuilder isn't read concurrently
                    // by the UI dispatcher Post (which would race the append).
                    var snapshot = TruncateTail(stderr.ToString());
                    Post(() => ErrorTail = snapshot);
                },
                cancellationToken: _cts.Token).ConfigureAwait(true);

            if (_cts.IsCancellationRequested)
            {
                StatusMessage = "Cancelled.";
                return false;
            }
            if (exit != 0)
            {
                StatusMessage = $"ffmpeg failed (exit {exit}).";
                ErrorTail = TruncateTail(stderr.ToString());
                return false;
            }
            Progress = 1.0;
            StatusMessage = "Done.";
            return true;
        }
        catch (OperationCanceledException)
        {
            StatusMessage = "Cancelled.";
            return false;
        }
        catch (Exception ex)
        {
            logger.LogError(ex, "Encode failed");
            StatusMessage = $"Failed: {ex.Message}";
            return false;
        }
        finally
        {
            IsRunning = false;
            _cts?.Dispose();
            _cts = null;
        }
    }

    public void Cancel()
    {
        try { _cts?.Cancel(); } catch { }
    }

    private async Task<bool> RunTwoPassAsync(EncodeJob job, StringBuilder stderr)
    {
        if (job.Compression.TargetSizeBytes is not { } sizeBytes)
        {
            StatusMessage = "Target size not set.";
            return false;
        }
        var duration = job.Cut?.Duration ?? job.SourceDuration;
        var bps = EncodeCommandBuilder.ComputeTargetBitrate(sizeBytes, duration, job.Compression.AudioBitrateKbps);
        var revisedComp = job.Compression with { TargetBitrateBps = bps };
        var revisedJob = job with { Compression = revisedComp };

        using var temp = new TempFileManager();
        var prefix = temp.Allocate(".log");
        // ffmpeg appends suffix to the prefix — strip extension.
        var rawPrefix = prefix[..^4];
        temp.Register(rawPrefix + "-0.log");
        temp.Register(rawPrefix + "-0.log.mbtree");

        var passes = EncodeCommandBuilder.BuildTwoPass(revisedJob, rawPrefix);

        StatusMessage = "Pass 1/2 (analysis)…";
        var exit1 = await ffmpeg.RunAsync(
            passes.Pass1,
            onProgress: p => Post(() => UpdateProgress(p)),
            onStdErrLine: line =>
            {
                if (stderr.Length < 32_000) stderr.AppendLine(line);
                var snapshot = TruncateTail(stderr.ToString());
                Post(() => ErrorTail = snapshot);
            },
            cancellationToken: _cts!.Token).ConfigureAwait(true);

        if (_cts.IsCancellationRequested) { StatusMessage = "Cancelled."; return false; }
        if (exit1 != 0)
        {
            StatusMessage = $"Pass 1 failed (exit {exit1}).";
            ErrorTail = TruncateTail(stderr.ToString());
            return false;
        }

        StatusMessage = "Pass 2/2 (encode)…";
        OutTime = TimeSpan.Zero;
        var exit2 = await ffmpeg.RunAsync(
            passes.Pass2,
            onProgress: p => Post(() => UpdateProgress(p)),
            onStdErrLine: line =>
            {
                if (stderr.Length < 32_000) stderr.AppendLine(line);
                var snapshot = TruncateTail(stderr.ToString());
                Post(() => ErrorTail = snapshot);
            },
            cancellationToken: _cts.Token).ConfigureAwait(true);

        if (_cts.IsCancellationRequested) { StatusMessage = "Cancelled."; return false; }
        if (exit2 != 0)
        {
            StatusMessage = $"Pass 2 failed (exit {exit2}).";
            ErrorTail = TruncateTail(stderr.ToString());
            return false;
        }

        Progress = 1.0;
        StatusMessage = "Done.";
        return true;
    }

    private void UpdateProgress(EncodeProgress p)
    {
        Frame = p.Frame;
        Fps = p.Fps;
        OutTime = p.OutTime;
        Speed = p.Speed;
        if (_totalDuration > TimeSpan.Zero)
        {
            Progress = Math.Clamp(p.OutTime.TotalSeconds / _totalDuration.TotalSeconds, 0.0, 1.0);
        }
        OnPropertyChanged(nameof(Eta));
    }

    private void Post(Action action)
    {
        if (_uiContext is null) action();
        else _uiContext.Post(_ => action(), null);
    }

    private static string TruncateTail(string s)
    {
        const int max = 4000;
        if (s.Length <= max) return s;
        return s[^max..];
    }
}
