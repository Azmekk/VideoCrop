using System;
using System.Collections.Generic;
using Microsoft.UI;
using Microsoft.UI.Input;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Shapes;
using Windows.Foundation;
using Windows.UI;
using VideoCrop.Core.Models;

namespace VideoCrop.App.Controls;

public sealed class CropOverlay : UserControl
{
    public int SourceWidth { get; private set; } = 1920;
    public int SourceHeight { get; private set; } = 1080;
    public ImageSource? BackgroundImage
    {
        get => _bgImage.Source;
        set => _bgImage.Source = value;
    }

    public enum AspectMode { Free, R16x9, R4x3, R1x1, R9x16, Original }
    public AspectMode Aspect { get; set; } = AspectMode.Free;

    public CropSpec Crop
    {
        get => new(_x, _y, _w, _h);
        set
        {
            _x = value.X; _y = value.Y; _w = value.Width; _h = value.Height;
            Layout();
            RaiseChange();
        }
    }

    public event EventHandler<CropSpec>? CropChanged;

    private readonly Grid _root;
    private readonly Image _bgImage;
    private readonly Canvas _canvas;
    private readonly Rectangle _dim;
    private readonly Rectangle _rect;
    private readonly Border[] _handles = new Border[8];

    // Inset on each side (in source-pixel units, since the overlay coord
    // space matches source dimensions) reserved so handles drawn at the
    // crop rect's edge can fully extend past the image without getting
    // clipped by the Viewbox / dialog bounds.
    private const double HandleMargin = 16;

    private readonly InputCursor _arrowCursor;
    private readonly InputCursor _nwSeCursor;
    private readonly InputCursor _neSwCursor;
    private readonly InputCursor _nsCursor;
    private readonly InputCursor _weCursor;
    private readonly InputCursor _moveCursor;

    private int _x, _y, _w, _h;
    private enum Drag { None, Move, NW, N, NE, E, SE, S, SW, W }
    private Drag _drag = Drag.None;
    private Point _dragStart;
    private (int x, int y, int w, int h) _dragOrig;

    public CropOverlay()
    {
        _bgImage = new Image
        {
            Stretch = Stretch.Uniform,
            Margin = new Thickness(HandleMargin),
        };
        _canvas = new Canvas { HorizontalAlignment = HorizontalAlignment.Stretch, VerticalAlignment = VerticalAlignment.Stretch };
        _dim = new Rectangle { Fill = new SolidColorBrush(Color.FromArgb(120, 0, 0, 0)) };
        _rect = new Rectangle
        {
            Stroke = new SolidColorBrush(Color.FromArgb(255, 255, 255, 255)),
            StrokeThickness = 2,
            Fill = new SolidColorBrush(Color.FromArgb(0, 0, 0, 0)),
        };
        _canvas.Children.Add(_dim);
        _canvas.Children.Add(_rect);
        for (var i = 0; i < 8; i++)
        {
            var h = new Border
            {
                Width = 12,
                Height = 12,
                Background = new SolidColorBrush(Color.FromArgb(255, 255, 255, 255)),
                BorderBrush = new SolidColorBrush(Color.FromArgb(255, 60, 60, 60)),
                BorderThickness = new Thickness(1),
                CornerRadius = new CornerRadius(2),
            };
            _handles[i] = h;
            _canvas.Children.Add(h);
        }

        _root = new Grid();
        _root.Children.Add(_bgImage);
        _root.Children.Add(_canvas);
        Content = _root;

        _canvas.Background = new SolidColorBrush(Color.FromArgb(1, 0, 0, 0));
        _canvas.PointerPressed += OnPointerPressed;
        _canvas.PointerMoved += OnPointerMoved;
        _canvas.PointerReleased += OnPointerReleased;
        _canvas.PointerCaptureLost += OnPointerReleased;

        _arrowCursor = InputSystemCursor.Create(InputSystemCursorShape.Arrow);
        _nwSeCursor = InputSystemCursor.Create(InputSystemCursorShape.SizeNorthwestSoutheast);
        _neSwCursor = InputSystemCursor.Create(InputSystemCursorShape.SizeNortheastSouthwest);
        _nsCursor = InputSystemCursor.Create(InputSystemCursorShape.SizeNorthSouth);
        _weCursor = InputSystemCursor.Create(InputSystemCursorShape.SizeWestEast);
        _moveCursor = InputSystemCursor.Create(InputSystemCursorShape.SizeAll);
        ProtectedCursor = _arrowCursor;

        SizeChanged += (_, _) => Layout();
    }

    public void SetSource(int width, int height)
    {
        SourceWidth = Math.Max(2, width);
        SourceHeight = Math.Max(2, height);
        if (_w == 0 || _h == 0) { _x = 0; _y = 0; _w = SourceWidth; _h = SourceHeight; }
        Layout();
    }

    private (double scale, double offsetX, double offsetY) GetTransform()
    {
        var canvasW = ActualWidth - 2 * HandleMargin;
        var canvasH = ActualHeight - 2 * HandleMargin;
        if (canvasW <= 0 || canvasH <= 0 || SourceWidth <= 0 || SourceHeight <= 0)
            return (1, 0, 0);
        var sx = canvasW / SourceWidth;
        var sy = canvasH / SourceHeight;
        var s = Math.Min(sx, sy);
        var ox = HandleMargin + (canvasW - SourceWidth * s) / 2;
        var oy = HandleMargin + (canvasH - SourceHeight * s) / 2;
        return (s, ox, oy);
    }

    private void Layout()
    {
        var (s, ox, oy) = GetTransform();
        if (s <= 0) return;

        Canvas.SetLeft(_dim, ox);
        Canvas.SetTop(_dim, oy);
        _dim.Width = SourceWidth * s;
        _dim.Height = SourceHeight * s;

        var rx = ox + _x * s;
        var ry = oy + _y * s;
        var rw = _w * s;
        var rh = _h * s;
        Canvas.SetLeft(_rect, rx);
        Canvas.SetTop(_rect, ry);
        _rect.Width = Math.Max(0, rw);
        _rect.Height = Math.Max(0, rh);

        var positions = new[]
        {
            (rx, ry), (rx + rw / 2, ry), (rx + rw, ry),
            (rx + rw, ry + rh / 2),
            (rx + rw, ry + rh), (rx + rw / 2, ry + rh), (rx, ry + rh),
            (rx, ry + rh / 2),
        };
        for (var i = 0; i < 8; i++)
        {
            var (hx, hy) = positions[i];
            Canvas.SetLeft(_handles[i], hx - _handles[i].Width / 2);
            Canvas.SetTop(_handles[i], hy - _handles[i].Height / 2);
        }
    }

    private void OnPointerPressed(object sender, PointerRoutedEventArgs e)
    {
        var p = e.GetCurrentPoint(_canvas).Position;
        _drag = HitTest(p);
        if (_drag == Drag.None) return;
        _dragStart = p;
        _dragOrig = (_x, _y, _w, _h);
        _canvas.CapturePointer(e.Pointer);
    }

    private void OnPointerMoved(object sender, PointerRoutedEventArgs e)
    {
        var p = e.GetCurrentPoint(_canvas).Position;
        UpdateCursor(_drag != Drag.None ? _drag : HitTest(p));

        if (_drag == Drag.None) return;
        var (s, _, _) = GetTransform();
        if (s <= 0) return;
        var dxPx = (p.X - _dragStart.X) / s;
        var dyPx = (p.Y - _dragStart.Y) / s;
        ApplyDrag((int)Math.Round(dxPx), (int)Math.Round(dyPx));
        Layout();
        RaiseChange();
    }

    private void UpdateCursor(Drag mode)
    {
        ProtectedCursor = mode switch
        {
            Drag.NW or Drag.SE => _nwSeCursor,
            Drag.NE or Drag.SW => _neSwCursor,
            Drag.N or Drag.S => _nsCursor,
            Drag.E or Drag.W => _weCursor,
            Drag.Move => _moveCursor,
            _ => _arrowCursor,
        };
    }

    private void OnPointerReleased(object sender, PointerRoutedEventArgs e)
    {
        if (_drag != Drag.None)
        {
            _canvas.ReleasePointerCapture(e.Pointer);
            _drag = Drag.None;
        }
    }

    private void ApplyDrag(int dx, int dy)
    {
        var (ox, oy, ow, oh) = _dragOrig;
        int nx = ox, ny = oy, nw = ow, nh = oh;
        switch (_drag)
        {
            case Drag.Move:
                nx = ox + dx; ny = oy + dy; break;
            case Drag.NW:
                nx = ox + dx; ny = oy + dy; nw = ow - dx; nh = oh - dy; break;
            case Drag.N:
                ny = oy + dy; nh = oh - dy; break;
            case Drag.NE:
                ny = oy + dy; nw = ow + dx; nh = oh - dy; break;
            case Drag.E:
                nw = ow + dx; break;
            case Drag.SE:
                nw = ow + dx; nh = oh + dy; break;
            case Drag.S:
                nh = oh + dy; break;
            case Drag.SW:
                nx = ox + dx; nw = ow - dx; nh = oh + dy; break;
            case Drag.W:
                nx = ox + dx; nw = ow - dx; break;
        }

        if (nw < 16) { nw = 16; if (_drag is Drag.NW or Drag.W or Drag.SW) nx = ox + ow - 16; }
        if (nh < 16) { nh = 16; if (_drag is Drag.NW or Drag.N or Drag.NE) ny = oy + oh - 16; }

        var aspect = AspectValue();
        if (_drag != Drag.Move && aspect is { } targetAspect && targetAspect > 0)
        {
            // Adjust nw/nh to preserve aspect, anchored on opposite edge.
            var cw = (double)nw;
            var ch = (double)nh;
            // Prefer adjusting the lesser axis.
            var aspectFromBox = cw / Math.Max(1, ch);
            if (aspectFromBox > targetAspect) cw = ch * targetAspect;
            else ch = cw / targetAspect;
            nw = (int)Math.Round(cw);
            nh = (int)Math.Round(ch);
            if (_drag is Drag.NW or Drag.W or Drag.SW) nx = ox + ow - nw;
            if (_drag is Drag.NW or Drag.N or Drag.NE) ny = oy + oh - nh;
        }

        nx = Math.Clamp(nx, 0, SourceWidth - 2);
        ny = Math.Clamp(ny, 0, SourceHeight - 2);
        if (nx + nw > SourceWidth) nw = SourceWidth - nx;
        if (ny + nh > SourceHeight) nh = SourceHeight - ny;

        _x = nx; _y = ny; _w = nw; _h = nh;
    }

    private double? AspectValue() => Aspect switch
    {
        AspectMode.R16x9 => 16.0 / 9.0,
        AspectMode.R4x3 => 4.0 / 3.0,
        AspectMode.R1x1 => 1.0,
        AspectMode.R9x16 => 9.0 / 16.0,
        AspectMode.Original => SourceHeight > 0 ? (double)SourceWidth / SourceHeight : null,
        _ => null,
    };

    private Drag HitTest(Point p)
    {
        for (var i = 0; i < 8; i++)
        {
            var h = _handles[i];
            var hx = Canvas.GetLeft(h);
            var hy = Canvas.GetTop(h);
            if (p.X >= hx && p.X <= hx + h.Width && p.Y >= hy && p.Y <= hy + h.Height)
            {
                return i switch
                {
                    0 => Drag.NW, 1 => Drag.N, 2 => Drag.NE,
                    3 => Drag.E, 4 => Drag.SE, 5 => Drag.S,
                    6 => Drag.SW, 7 => Drag.W,
                    _ => Drag.None,
                };
            }
        }

        var rx = Canvas.GetLeft(_rect);
        var ry = Canvas.GetTop(_rect);
        if (p.X >= rx && p.X <= rx + _rect.Width && p.Y >= ry && p.Y <= ry + _rect.Height)
            return Drag.Move;
        return Drag.None;
    }

    private void RaiseChange() => CropChanged?.Invoke(this, new CropSpec(_x, _y, _w, _h));
}
