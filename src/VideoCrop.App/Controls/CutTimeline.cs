using System;
using Microsoft.UI;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Shapes;
using Windows.Foundation;
using Windows.UI;

namespace VideoCrop.App.Controls;

public sealed class CutTimeline : UserControl
{
    public static readonly DependencyProperty StartProperty = DependencyProperty.Register(
        nameof(Start), typeof(TimeSpan), typeof(CutTimeline),
        new PropertyMetadata(TimeSpan.Zero, OnAnyChanged));

    public static readonly DependencyProperty EndProperty = DependencyProperty.Register(
        nameof(End), typeof(TimeSpan), typeof(CutTimeline),
        new PropertyMetadata(TimeSpan.Zero, OnAnyChanged));

    public static readonly DependencyProperty DurationProperty = DependencyProperty.Register(
        nameof(Duration), typeof(TimeSpan), typeof(CutTimeline),
        new PropertyMetadata(TimeSpan.Zero, OnAnyChanged));

    public static readonly DependencyProperty PositionProperty = DependencyProperty.Register(
        nameof(Position), typeof(TimeSpan), typeof(CutTimeline),
        new PropertyMetadata(TimeSpan.Zero, OnAnyChanged));

    public TimeSpan Start { get => (TimeSpan)GetValue(StartProperty); set => SetValue(StartProperty, value); }
    public TimeSpan End { get => (TimeSpan)GetValue(EndProperty); set => SetValue(EndProperty, value); }
    public TimeSpan Duration { get => (TimeSpan)GetValue(DurationProperty); set => SetValue(DurationProperty, value); }
    public TimeSpan Position { get => (TimeSpan)GetValue(PositionProperty); set => SetValue(PositionProperty, value); }

    public event EventHandler<TimeSpan>? StartChanged;
    public event EventHandler<TimeSpan>? EndChanged;
    public event EventHandler<TimeSpan>? PositionRequested;

    private readonly Canvas _canvas;
    private readonly Rectangle _trackBg;
    private readonly Rectangle _activeRegion;
    private readonly Rectangle _playhead;
    private readonly Border _startHandle;
    private readonly Border _endHandle;

    private enum DragTarget { None, Start, End }
    private DragTarget _drag = DragTarget.None;

    public CutTimeline()
    {
        Height = 40;

        _canvas = new Canvas { HorizontalAlignment = HorizontalAlignment.Stretch, VerticalAlignment = VerticalAlignment.Stretch };
        _trackBg = new Rectangle { Fill = new SolidColorBrush(Color.FromArgb(64, 0, 0, 0)), Height = 8, RadiusX = 3, RadiusY = 3 };
        _activeRegion = new Rectangle { Fill = (Brush)Application.Current.Resources["AccentFillColorDefaultBrush"], Height = 8, RadiusX = 3, RadiusY = 3 };
        _playhead = new Rectangle { Fill = new SolidColorBrush(Color.FromArgb(255, 255, 255, 255)), Width = 2 };

        _startHandle = MakeHandle();
        _endHandle = MakeHandle();

        _canvas.Children.Add(_trackBg);
        _canvas.Children.Add(_activeRegion);
        _canvas.Children.Add(_playhead);
        _canvas.Children.Add(_startHandle);
        _canvas.Children.Add(_endHandle);

        _canvas.PointerPressed += OnPointerPressed;
        _canvas.PointerMoved += OnPointerMoved;
        _canvas.PointerReleased += OnPointerReleased;
        _canvas.PointerCaptureLost += OnPointerReleased;

        Content = _canvas;
        SizeChanged += (_, _) => Layout();
    }

    private static Border MakeHandle() => new()
    {
        Width = 12,
        Height = 28,
        CornerRadius = new CornerRadius(3),
        Background = new SolidColorBrush(Color.FromArgb(255, 255, 255, 255)),
        BorderBrush = new SolidColorBrush(Color.FromArgb(255, 60, 60, 60)),
        BorderThickness = new Thickness(1),
    };

    private static void OnAnyChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        if (d is CutTimeline t) t.Layout();
    }

    private void Layout()
    {
        var width = ActualWidth;
        var height = ActualHeight;
        if (width <= 0 || height <= 0) return;

        var trackY = (height - 8) / 2;
        _trackBg.Width = width;
        Canvas.SetLeft(_trackBg, 0);
        Canvas.SetTop(_trackBg, trackY);

        var totalSec = Math.Max(Duration.TotalSeconds, 0.001);
        var startX = Math.Clamp(Start.TotalSeconds / totalSec, 0, 1) * width;
        var endX = Math.Clamp(End.TotalSeconds / totalSec, 0, 1) * width;
        if (endX < startX) endX = startX;

        _activeRegion.Width = Math.Max(0, endX - startX);
        Canvas.SetLeft(_activeRegion, startX);
        Canvas.SetTop(_activeRegion, trackY);

        var posX = Math.Clamp(Position.TotalSeconds / totalSec, 0, 1) * width;
        _playhead.Height = height;
        Canvas.SetLeft(_playhead, posX);
        Canvas.SetTop(_playhead, 0);
        var inBounds = Position >= Start && Position <= End;
        _playhead.Opacity = inBounds ? 1.0 : 0.0;

        Canvas.SetLeft(_startHandle, startX - _startHandle.Width / 2);
        Canvas.SetTop(_startHandle, (height - _startHandle.Height) / 2);
        Canvas.SetLeft(_endHandle, endX - _endHandle.Width / 2);
        Canvas.SetTop(_endHandle, (height - _endHandle.Height) / 2);
    }

    private void OnPointerPressed(object sender, PointerRoutedEventArgs e)
    {
        var pos = e.GetCurrentPoint(_canvas).Position;
        _drag = ClosestHandle(pos);
        if (_drag == DragTarget.None)
        {
            var t = TimeFromX(pos.X);
            var clamped = TimeSpan.FromTicks(Math.Clamp(t.Ticks, Start.Ticks, End.Ticks));
            PositionRequested?.Invoke(this, clamped);
            return;
        }
        _canvas.CapturePointer(e.Pointer);
    }

    private void OnPointerMoved(object sender, PointerRoutedEventArgs e)
    {
        if (_drag == DragTarget.None) return;
        var pos = e.GetCurrentPoint(_canvas).Position;
        var t = TimeFromX(pos.X);
        if (_drag == DragTarget.Start) StartChanged?.Invoke(this, t);
        else EndChanged?.Invoke(this, t);
    }

    private void OnPointerReleased(object sender, PointerRoutedEventArgs e)
    {
        if (_drag != DragTarget.None)
        {
            _canvas.ReleasePointerCapture(e.Pointer);
            _drag = DragTarget.None;
        }
    }

    private DragTarget ClosestHandle(Point pos)
    {
        const double tolerance = 14;
        var sx = Canvas.GetLeft(_startHandle) + _startHandle.Width / 2;
        var ex = Canvas.GetLeft(_endHandle) + _endHandle.Width / 2;
        var ds = Math.Abs(pos.X - sx);
        var de = Math.Abs(pos.X - ex);
        var nearest = ds <= de ? DragTarget.Start : DragTarget.End;
        var nearestDist = Math.Min(ds, de);
        return nearestDist <= tolerance ? nearest : DragTarget.None;
    }

    private TimeSpan TimeFromX(double x)
    {
        if (ActualWidth <= 0) return TimeSpan.Zero;
        var fraction = Math.Clamp(x / ActualWidth, 0, 1);
        return TimeSpan.FromSeconds(fraction * Duration.TotalSeconds);
    }
}
