using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using VideoCrop.Core.Models;

namespace VideoCrop.App.ViewModels;

public sealed class CutViewModel : ObservableObject
{
    public static readonly TimeSpan MinDuration = TimeSpan.FromMilliseconds(100);

    private TimeSpan _start;
    private TimeSpan _end;
    private TimeSpan _sourceDuration;
    private bool _accurate;

    public PlayerViewModel? Player { get; set; }

    public event EventHandler? BoundsChanged;

    public TimeSpan Start
    {
        get => _start;
        set
        {
            if (value < TimeSpan.Zero) value = TimeSpan.Zero;
            if (value > _end - MinDuration) value = _end - MinDuration;
            if (SetProperty(ref _start, value))
            {
                NotifyDerived();
                ApplyToPlayer();
            }
        }
    }

    public TimeSpan End
    {
        get => _end;
        set
        {
            if (value > _sourceDuration) value = _sourceDuration;
            if (value < _start + MinDuration) value = _start + MinDuration;
            if (SetProperty(ref _end, value))
            {
                NotifyDerived();
                ApplyToPlayer();
            }
        }
    }

    public TimeSpan SourceDuration
    {
        get => _sourceDuration;
        set
        {
            if (SetProperty(ref _sourceDuration, value))
            {
                if (_end > value || _end == TimeSpan.Zero) _end = value;
                NotifyDerived();
                ApplyToPlayer();
            }
        }
    }

    public bool Accurate
    {
        get => _accurate;
        set => SetProperty(ref _accurate, value);
    }

    public TimeSpan Duration => _end - _start;
    public bool IsActive => _start > TimeSpan.Zero || _end < _sourceDuration - TimeSpan.FromMilliseconds(1);

    public CutSpec? AsSpecOrNull() => IsActive ? new CutSpec(_start, _end, _accurate) : null;

    public void Reset()
    {
        _start = TimeSpan.Zero;
        _end = _sourceDuration;
        NotifyDerived();
        OnPropertyChanged(nameof(Start));
        OnPropertyChanged(nameof(End));
        ApplyToPlayer();
    }

    public async Task SetStartFromPlayheadAsync()
    {
        if (Player is null) return;
        Start = TimeSpan.FromSeconds(Player.PositionSeconds);
        await Task.CompletedTask;
    }

    public async Task SetEndFromPlayheadAsync()
    {
        if (Player is null) return;
        End = TimeSpan.FromSeconds(Player.PositionSeconds);
        await Task.CompletedTask;
    }

    private void NotifyDerived()
    {
        OnPropertyChanged(nameof(Duration));
        OnPropertyChanged(nameof(IsActive));
        BoundsChanged?.Invoke(this, EventArgs.Empty);
    }

    private void ApplyToPlayer()
    {
        if (Player is null) return;
        _ = Player.ApplyCutBoundsAsync(IsActive ? _start : (TimeSpan?)null, IsActive ? _end : (TimeSpan?)null);
    }
}
