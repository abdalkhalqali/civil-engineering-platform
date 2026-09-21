import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

import '../ffi_bridge/generated/api.dart';

enum ViewportInteraction { orbit, pan }

enum ViewportProjection { perspective, orthographic }

class EngineeringViewport extends StatefulWidget {
  const EngineeringViewport({
    required this.interaction,
    required this.projection,
    required this.snapshot,
    required this.onElementSelected,
    this.onGroundPointSelected,
    this.hiddenCategories = const <String>{},
    this.showGrid = true,
    this.resetToken = 0,
    super.key,
  });

  final ViewportInteraction interaction;
  final ViewportProjection projection;
  final WorkspaceSnapshot? snapshot;
  final ValueChanged<String> onElementSelected;
  final ValueChanged<Offset>? onGroundPointSelected;
  final Set<String> hiddenCategories;
  final bool showGrid;
  final int resetToken;

  @override
  State<EngineeringViewport> createState() => _EngineeringViewportState();
}

class _EngineeringViewportState extends State<EngineeringViewport> {
  double _yaw = -0.68;
  double _pitch = -0.56;
  double _zoom = 1;
  Offset _pan = Offset.zero;
  Offset _lastFocal = Offset.zero;
  String? _selected;
  int _lastResetToken = 0;

  @override
  void didUpdateWidget(covariant EngineeringViewport oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.resetToken != _lastResetToken) {
      _lastResetToken = widget.resetToken;
      _yaw = -0.68;
      _pitch = -0.56;
      _zoom = 1;
      _pan = Offset.zero;
    }
    if (widget.snapshot != oldWidget.snapshot &&
        _selected != null &&
        !(widget.snapshot?.elements.any(
              (element) => element.name == _selected,
            ) ??
            false)) {
      _selected = null;
    }
  }

  @override
  Widget build(BuildContext context) {
    return Listener(
      onPointerSignal: (signal) {
        if (signal is PointerScrollEvent) {
          setState(() {
            _zoom = (_zoom * (signal.scrollDelta.dy > 0 ? .9 : 1.1)).clamp(
              .45,
              2.8,
            );
          });
        }
      },
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onScaleStart: (details) => _lastFocal = details.focalPoint,
        onScaleUpdate: (details) {
          final delta = details.focalPoint - _lastFocal;
          _lastFocal = details.focalPoint;
          setState(() {
            if (details.scale != 1) {
              _zoom = (_zoom * details.scale).clamp(.45, 2.8);
            }
            if (widget.interaction == ViewportInteraction.orbit) {
              _yaw += delta.dx * .008;
              _pitch = (_pitch + delta.dy * .008).clamp(-1.35, .35);
            } else {
              _pan += delta;
            }
          });
        },
        onTapUp: (details) {
          final hit = _pickElement(details.localPosition, context.size);
          setState(() => _selected = hit);
          if (hit != null) {
            widget.onElementSelected(hit);
          } else {
            final size = context.size;
            if (size != null) {
              final camera = _SceneCamera(
                size: size,
                yaw: _yaw,
                pitch: _pitch,
                zoom: _zoom,
                pan: _pan,
                projection: widget.projection,
              );
              widget.onGroundPointSelected?.call(
                camera.groundPoint(details.localPosition),
              );
            }
          }
        },
        child: CustomPaint(
          painter: _EngineeringScenePainter(
            yaw: _yaw,
            pitch: _pitch,
            zoom: _zoom,
            pan: _pan,
            projection: widget.projection,
            snapshot: widget.snapshot,
            selectedElement: _selected,
            hiddenCategories: widget.hiddenCategories,
            showGrid: widget.showGrid,
          ),
          child: const SizedBox.expand(),
        ),
      ),
    );
  }

  String? _pickElement(Offset point, Size? size) {
    if (size == null || size.isEmpty) return null;
    final camera = _SceneCamera(
      size: size,
      yaw: _yaw,
      pitch: _pitch,
      zoom: _zoom,
      pan: _pan,
      projection: widget.projection,
    );
    String? closest;
    var distance = 22.0;
    for (final element
        in widget.snapshot?.elements ?? const <ElementSnapshot>[]) {
      for (final segment in _elementSegments(element)) {
        final a = camera.project(segment.$1);
        final b = camera.project(segment.$2);
        final d = _distanceToSegment(point, a, b);
        if (d < distance) {
          distance = d;
          closest = element.name;
        }
      }
    }
    return closest;
  }

  List<(_Vec3, _Vec3)> _elementSegments(ElementSnapshot element) {
    switch (element.category) {
      case 'column':
        return [
          (
            _Vec3(element.x, element.y, element.z),
            _Vec3(element.x, element.y, element.topZ),
          ),
        ];
      case 'beam':
        return [(_vec3(element.start), _vec3(element.end))];
      case 'wall':
        return [(_vec3(element.start), _vec3(element.end))];
      case 'slab':
        return [
          for (var i = 0; i < element.boundary.length; i++)
            (
              _vec3(element.boundary[i]),
              _vec3(element.boundary[(i + 1) % element.boundary.length]),
            ),
        ];
      case 'foundation':
        return [
          for (var i = 0; i < element.boundary.length; i++)
            (
              _vec3(element.boundary[i]),
              _vec3(element.boundary[(i + 1) % element.boundary.length]),
            ),
        ];
      default:
        return const [];
    }
  }

  _Vec3 _vec3(PointSnapshot point) => _Vec3(point.x, point.y, point.z);

  double _distanceToSegment(Offset p, Offset a, Offset b) {
    final ab = b - a;
    final lengthSquared = ab.dx * ab.dx + ab.dy * ab.dy;
    if (lengthSquared == 0) return (p - a).distance;
    final t = ((p.dx - a.dx) * ab.dx + (p.dy - a.dy) * ab.dy) / lengthSquared;
    final clamped = t.clamp(0.0, 1.0);
    final projection = Offset(a.dx + ab.dx * clamped, a.dy + ab.dy * clamped);
    return (p - projection).distance;
  }
}

class _EngineeringScenePainter extends CustomPainter {
  _EngineeringScenePainter({
    required this.yaw,
    required this.pitch,
    required this.zoom,
    required this.pan,
    required this.projection,
    required this.snapshot,
    required this.selectedElement,
    required this.hiddenCategories,
    required this.showGrid,
  });

  final double yaw;
  final double pitch;
  final double zoom;
  final Offset pan;
  final ViewportProjection projection;
  final WorkspaceSnapshot? snapshot;
  final String? selectedElement;
  final Set<String> hiddenCategories;
  final bool showGrid;

  @override
  void paint(Canvas canvas, Size size) {
    final background = Paint()
      ..shader = const LinearGradient(
        begin: Alignment.topCenter,
        end: Alignment.bottomCenter,
        colors: [Color(0xff0c2332), Color(0xff07131f)],
      ).createShader(Offset.zero & size);
    canvas.drawRect(Offset.zero & size, background);

    final camera = _SceneCamera(
      size: size,
      yaw: yaw,
      pitch: pitch,
      zoom: zoom,
      pan: pan,
      projection: projection,
    );

    if (showGrid) _drawGrid(canvas, camera);
    _drawBuilding(canvas, camera);
    _drawAxes(canvas, camera);
  }

  void _drawGrid(Canvas canvas, _SceneCamera camera) {
    final minor = Paint()
      ..color = const Color(0xff1c3b4a)
      ..strokeWidth = 0.7;
    final major = Paint()
      ..color = const Color(0xff2a5260)
      ..strokeWidth = 1.05;

    // Collect grid line positions from the snapshot
    final grids = snapshot?.grids ?? const <GridSnapshot>[];
    final xOffsets = <double>[];
    final yOffsets = <double>[];
    for (final grid in grids) {
      if (grid.direction == 'along_y') {
        xOffsets.add(grid.offsetM);
      } else {
        yOffsets.add(grid.offsetM);
      }
    }

    // Draw named grid lines
    for (final gx in xOffsets) {
      _line(
        canvas,
        camera,
        _Vec3(gx, -10, 0),
        _Vec3(gx, 10, 0),
        major,
      );
      // Draw grid label
      final labelPos = camera.project(_Vec3(gx, -10, 0));
      _label(canvas, grids
          .where((g) => g.direction == 'along_y' && (g.offsetM - gx).abs() < 0.01)
          .map((g) => g.name)
          .firstOrNull ?? '', labelPos + const Offset(4, -12), const Color(0xff54e0d7));
    }
    for (final gy in yOffsets) {
      _line(
        canvas,
        camera,
        _Vec3(-10, gy, 0),
        _Vec3(10, gy, 0),
        major,
      );
      // Draw grid label
      final labelPos = camera.project(_Vec3(-10, gy, 0));
      _label(canvas, grids
          .where((g) => g.direction == 'along_x' && (g.offsetM - gy).abs() < 0.01)
          .map((g) => g.name)
          .firstOrNull ?? '', labelPos + const Offset(-14, -12), const Color(0xff54e0d7));
    }

    // Draw default grid lines for areas without named grids
    for (var i = -10; i <= 10; i++) {
      final hasXLine = xOffsets.any((x) => (x - i.toDouble()).abs() < 0.01);
      final hasYLine = yOffsets.any((y) => (y - i.toDouble()).abs() < 0.01);
      if (!hasXLine) {
        _line(
          canvas,
          camera,
          _Vec3(i.toDouble(), -10, 0),
          _Vec3(i.toDouble(), 10, 0),
          minor,
        );
      }
      if (!hasYLine) {
        _line(
          canvas,
          camera,
          _Vec3(-10, i.toDouble(), 0),
          _Vec3(10, i.toDouble(), 0),
          minor,
        );
      }
    }
  }

  void _drawAxes(Canvas canvas, _SceneCamera camera) {
    final origin = camera.project(const _Vec3(0, 0, 0));
    final xEnd = camera.project(const _Vec3(2.3, 0, 0));
    final yEnd = camera.project(const _Vec3(0, 2.3, 0));
    final zEnd = camera.project(const _Vec3(0, 0, 2.3));
    _line(
      canvas,
      camera,
      const _Vec3(0, 0, 0),
      const _Vec3(2.3, 0, 0),
      Paint()
        ..color = const Color(0xffef736a)
        ..strokeWidth = 2,
    );
    _line(
      canvas,
      camera,
      const _Vec3(0, 0, 0),
      const _Vec3(0, 2.3, 0),
      Paint()
        ..color = const Color(0xff64d69b)
        ..strokeWidth = 2,
    );
    _line(
      canvas,
      camera,
      const _Vec3(0, 0, 0),
      const _Vec3(0, 0, 2.3),
      Paint()
        ..color = const Color(0xff6caaff)
        ..strokeWidth = 2,
    );
    _label(canvas, 'X', xEnd + const Offset(7, 0), const Color(0xffef736a));
    _label(canvas, 'Y', yEnd + const Offset(7, 0), const Color(0xff64d69b));
    _label(canvas, 'Z', zEnd + const Offset(7, -2), const Color(0xff6caaff));
    canvas.drawCircle(origin, 3, Paint()..color = const Color(0xfff3c969));
  }

  void _drawBuilding(Canvas canvas, _SceneCamera camera) {
    final elements = snapshot?.elements ?? const <ElementSnapshot>[];
    final beamPaint = Paint()
      ..color = const Color(0xffe7a85f)
      ..strokeWidth = 5
      ..strokeCap = StrokeCap.round;
    final columnPaint = Paint()
      ..color = const Color(0xffc8d8d7)
      ..strokeWidth = 6
      ..strokeCap = StrokeCap.round;
    final selectedPaint = Paint()
      ..color = const Color(0xff62eee2)
      ..strokeWidth = 9
      ..strokeCap = StrokeCap.round;
    final wallFillPaint = Paint()
      ..color = const Color(0x50d4a574)
      ..style = PaintingStyle.fill;
    final wallStrokePaint = Paint()
      ..color = const Color(0xffd4a574)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;
    final wallSelectedPaint = Paint()
      ..color = const Color(0xff62eee2)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.5;
    final foundationFillPaint = Paint()
      ..color = const Color(0x40a08060)
      ..style = PaintingStyle.fill;
    final foundationStrokePaint = Paint()
      ..color = const Color(0xffa08060)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2;
    final foundationSelectedPaint = Paint()
      ..color = const Color(0xff62eee2)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.0;

    // Draw level elevation lines
    final levels = snapshot?.levels ?? const <LevelSnapshot>[];
    for (final level in levels) {
      if (level.elevationM.abs() > 0.01) {
        final levelPaint = Paint()
          ..color = const Color(0x3054e0d7)
          ..strokeWidth = 0.5
          ..style = PaintingStyle.stroke;
        _line(
          canvas,
          camera,
          _Vec3(-8, -8, level.elevationM),
          _Vec3(8, 8, level.elevationM),
          levelPaint,
        );
        _label(
          canvas,
          '${level.name} (${level.elevationM.toStringAsFixed(2)} m)',
          camera.project(_Vec3(-8, -8, level.elevationM)) + const Offset(4, -14),
          const Color(0x8054e0d7),
        );
      }
    }

    for (final element in elements) {
      if (hiddenCategories.contains(element.category)) continue;
      final selected = element.name == selectedElement;
      switch (element.category) {
        case 'column':
          _drawColumn3D(canvas, camera, element, selected ? selectedPaint : columnPaint);
        case 'beam':
          _drawBeam3D(canvas, camera, element, selected ? selectedPaint : beamPaint);
        case 'slab':
          final points = element.boundary.map(_vec3).toList();
          if (points.length >= 3) {
            final polygon = Path()
              ..addPolygon(points.map(camera.project).toList(), true);
            canvas.drawPath(
              polygon,
              Paint()
                ..color = selected
                    ? const Color(0x704ee4dc)
                    : const Color(0x403d8b95)
                ..style = PaintingStyle.fill,
            );
            canvas.drawPath(
              polygon,
              Paint()
                ..color = selected
                    ? const Color(0xff62eee2)
                    : const Color(0xff5ebbc0)
                ..style = PaintingStyle.stroke
                ..strokeWidth = selected ? 2.2 : 1.2,
            );
            _label(
              canvas,
              '${element.name} · ${points.first.z.toStringAsFixed(2)} m',
              camera.project(points.first) + const Offset(-14, -14),
              const Color(0xff98d9d4),
            );
          }
        case 'wall':
          _drawWall3D(canvas, camera, element, selected);
        case 'foundation':
          _drawFoundation3D(canvas, camera, element, selected);
      }
    }
  }

  void _drawColumn3D(
    Canvas canvas,
    _SceneCamera camera,
    ElementSnapshot element,
    Paint paint,
  ) {
    final w = element.width / 2;
    final d = element.depth / 2;
    final x = element.x;
    final y = element.y;
    final z1 = element.z;
    final z2 = element.topZ;
    // Draw column as a 3D box
    final corners = [
      _Vec3(x - w, y - d, z1),
      _Vec3(x + w, y - d, z1),
      _Vec3(x + w, y + d, z1),
      _Vec3(x - w, y + d, z1),
      _Vec3(x - w, y - d, z2),
      _Vec3(x + w, y - d, z2),
      _Vec3(x + w, y + d, z2),
      _Vec3(x - w, y + d, z2),
    ];
    final projected = corners.map(camera.project).toList();
    // Bottom face
    final bottomPath = Path()
      ..addPolygon([projected[0], projected[1], projected[2], projected[3]], true);
    canvas.drawPath(
      bottomPath,
      Paint()
        ..color = paint.color.withValues(alpha: 0.15)
        ..style = PaintingStyle.fill,
    );
    // Top face
    final topPath = Path()
      ..addPolygon([projected[4], projected[5], projected[6], projected[7]], true);
    canvas.drawPath(
      topPath,
      Paint()
        ..color = paint.color.withValues(alpha: 0.3)
        ..style = PaintingStyle.fill,
    );
    // Edges
    for (final edge in [
      [0, 1], [1, 2], [2, 3], [3, 0],
      [4, 5], [5, 6], [6, 7], [7, 4],
      [0, 4], [1, 5], [2, 6], [3, 7],
    ]) {
      canvas.drawLine(projected[edge[0]], projected[edge[1]], paint);
    }
  }

  void _drawBeam3D(
    Canvas canvas,
    _SceneCamera camera,
    ElementSnapshot element,
    Paint paint,
  ) {
    final hw = element.width / 2;
    final hd = element.depth / 2;
    final start = element.start;
    final end = element.end;
    // Draw beam as a 3D box between start and end
    final dx = end.x - start.x;
    final dy = end.y - start.y;
    final len = _sqrt3(dx * dx + dy * dy);
    if (len < 0.001) {
      _line(canvas, camera, _vec3(start), _vec3(end), paint);
      return;
    }
    final nx = -dy / len * hw;
    final ny = dx / len * hw;
    final corners = [
      _Vec3(start.x + nx, start.y + ny, start.z - hd),
      _Vec3(start.x - nx, start.y - ny, start.z - hd),
      _Vec3(end.x - nx, end.y - ny, end.z - hd),
      _Vec3(end.x + nx, end.y + ny, end.z - hd),
      _Vec3(start.x + nx, start.y + ny, start.z + hd),
      _Vec3(start.x - nx, start.y - ny, start.z + hd),
      _Vec3(end.x - nx, end.y - ny, end.z + hd),
      _Vec3(end.x + nx, end.y + ny, end.z + hd),
    ];
    final projected = corners.map(camera.project).toList();
    // Top face
    final topPath = Path()
      ..addPolygon([projected[4], projected[5], projected[6], projected[7]], true);
    canvas.drawPath(
      topPath,
      Paint()
        ..color = paint.color.withValues(alpha: 0.25)
        ..style = PaintingStyle.fill,
    );
    // Edges
    for (final edge in [
      [0, 1], [1, 2], [2, 3], [3, 0],
      [4, 5], [5, 6], [6, 7], [7, 4],
      [0, 4], [1, 5], [2, 6], [3, 7],
    ]) {
      canvas.drawLine(projected[edge[0]], projected[edge[1]], paint);
    }
  }

  void _drawWall3D(
    Canvas canvas,
    _SceneCamera camera,
    ElementSnapshot element,
    bool selected,
  ) {
    final start = element.start;
    final end = element.end;
    final t = element.thickness / 2;
    final dx = end.x - start.x;
    final dy = end.y - start.y;
    final len = _sqrt3(dx * dx + dy * dy);
    if (len < 0.001) return;
    final nx = -dy / len * t;
    final ny = dx / len * t;
    // Bottom rectangle
    final bottomCorners = [
      _Vec3(start.x + nx, start.y + ny, start.z),
      _Vec3(start.x - nx, start.y - ny, start.z),
      _Vec3(end.x - nx, end.y - ny, start.z),
      _Vec3(end.x + nx, end.y + ny, start.z),
    ];
    // Top rectangle
    final topCorners = [
      _Vec3(start.x + nx, start.y + ny, element.topZ),
      _Vec3(start.x - nx, start.y - ny, element.topZ),
      _Vec3(end.x - nx, end.y - ny, element.topZ),
      _Vec3(end.x + nx, end.y + ny, element.topZ),
    ];
    final bProj = bottomCorners.map(camera.project).toList();
    final tProj = topCorners.map(camera.project).toList();
    // Front face
    final frontPath = Path()
      ..addPolygon([bProj[0], bProj[3], tProj[3], tProj[0]], true);
    canvas.drawPath(
      frontPath,
      Paint()
        ..color = selected ? const Color(0x704ee4dc) : const Color(0x50d4a574)
        ..style = PaintingStyle.fill,
    );
    canvas.drawPath(
      frontPath,
      selected
          ? Paint()
              ..color = const Color(0xff62eee2)
              ..style = PaintingStyle.stroke
              ..strokeWidth = 2.0
          : Paint()
              ..color = const Color(0xffd4a574)
              ..style = PaintingStyle.stroke
              ..strokeWidth = 1.0,
    );
    // Top edge
    canvas.drawLine(tProj[0], tProj[3],
        selected ? Paint()..color = const Color(0xff62eee2)..strokeWidth = 2.0 : Paint()..color = const Color(0xffd4a574)..strokeWidth = 1.0);
    canvas.drawLine(tProj[1], tProj[2],
        selected ? Paint()..color = const Color(0xff62eee2)..strokeWidth = 2.0 : Paint()..color = const Color(0xffd4a574)..strokeWidth = 1.0);
    canvas.drawLine(tProj[0], tProj[1],
        selected ? Paint()..color = const Color(0xff62eee2)..strokeWidth = 2.0 : Paint()..color = const Color(0xffd4a574)..strokeWidth = 1.0);
    canvas.drawLine(tProj[2], tProj[3],
        selected ? Paint()..color = const Color(0xff62eee2)..strokeWidth = 2.0 : Paint()..color = const Color(0xffd4a574)..strokeWidth = 1.0);
  }

  void _drawFoundation3D(
    Canvas canvas,
    _SceneCamera camera,
    ElementSnapshot element,
    bool selected,
  ) {
    final points = element.boundary.map(_vec3).toList();
    if (points.length < 3) return;
    // Bottom face
    final bottomPath = Path()
      ..addPolygon(points.map(camera.project).toList(), true);
    canvas.drawPath(
      bottomPath,
      Paint()
        ..color = selected ? const Color(0x704ee4dc) : const Color(0x40a08060)
        ..style = PaintingStyle.fill,
    );
    canvas.drawPath(
      bottomPath,
      Paint()
        ..color = selected ? const Color(0xff62eee2) : const Color(0xffa08060)
        ..style = PaintingStyle.stroke
        ..strokeWidth = selected ? 2.0 : 1.2,
    );
    // Draw thickness as side edges if visible
    final topPoints = points
        .map((p) => _Vec3(p.x, p.y, p.z - element.thickness))
        .toList();
    final topProj = topPoints.map(camera.project).toList();
    final topPath = Path()
      ..addPolygon(topProj, true);
    canvas.drawPath(
      topPath,
      Paint()
        ..color = selected ? const Color(0x704ee4dc) : const Color(0x30a08060)
        ..style = PaintingStyle.fill,
    );
  }

  double _sqrt3(double value) {
    if (value <= 0) return 0;
    double guess = value / 2;
    for (int i = 0; i < 15; i++) {
      guess = (guess + value / guess) / 2;
    }
    return guess;
  }

  void _line(
    Canvas canvas,
    _SceneCamera camera,
    _Vec3 a,
    _Vec3 b,
    Paint paint,
  ) {
    canvas.drawLine(camera.project(a), camera.project(b), paint);
  }

  _Vec3 _vec3(PointSnapshot point) => _Vec3(point.x, point.y, point.z);

  void _label(Canvas canvas, String text, Offset position, Color color) {
    final painter = TextPainter(
      text: TextSpan(
        text: text,
        style: TextStyle(
          color: color,
          fontSize: 10,
          fontWeight: FontWeight.w700,
          letterSpacing: .4,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    painter.paint(canvas, position);
  }

  @override
  bool shouldRepaint(covariant _EngineeringScenePainter oldDelegate) {
    return oldDelegate.yaw != yaw ||
        oldDelegate.pitch != pitch ||
        oldDelegate.zoom != zoom ||
        oldDelegate.pan != pan ||
        oldDelegate.projection != projection ||
        oldDelegate.snapshot != snapshot ||
        oldDelegate.selectedElement != selectedElement;
  }
}

class _SceneCamera {
  _SceneCamera({
    required this.size,
    required this.yaw,
    required this.pitch,
    required this.zoom,
    required this.pan,
    required this.projection,
  });

  final Size size;
  final double yaw;
  final double pitch;
  final double zoom;
  final Offset pan;
  final ViewportProjection projection;

  Offset project(_Vec3 point) {
    final cosYaw = math.cos(yaw);
    final sinYaw = math.sin(yaw);
    final horizontal = point.x * cosYaw - point.y * sinYaw;
    final depth = point.x * sinYaw + point.y * cosYaw;
    final cosPitch = math.cos(pitch);
    final sinPitch = math.sin(pitch);
    final vertical = point.z * cosPitch - depth * sinPitch;
    final cameraDepth = depth * cosPitch + point.z * sinPitch;
    final baseScale = math.min(size.width, size.height) * .062 * zoom;
    final scale = projection == ViewportProjection.perspective
        ? baseScale * (12 / (12 + cameraDepth * .32))
        : baseScale;
    return Offset(
      size.width / 2 + pan.dx + horizontal * scale,
      size.height / 2 + pan.dy - vertical * scale,
    );
  }

  Offset groundPoint(Offset screenPoint) {
    final baseScale = math.min(size.width, size.height) * .062 * zoom;
    final horizontal = (screenPoint.dx - size.width / 2 - pan.dx) / baseScale;
    final vertical = -(screenPoint.dy - size.height / 2 - pan.dy) / baseScale;
    final sinPitch = math.sin(pitch);
    final depth = sinPitch.abs() < .001 ? 0.0 : vertical / -sinPitch;
    final cosYaw = math.cos(yaw);
    final sinYaw = math.sin(yaw);
    return Offset(
      horizontal * cosYaw + depth * sinYaw,
      -horizontal * sinYaw + depth * cosYaw,
    );
  }
}

class _Vec3 {
  const _Vec3(this.x, this.y, this.z);

  final double x;
  final double y;
  final double z;

  _Vec3 copyWith({double? x, double? y, double? z}) =>
      _Vec3(x ?? this.x, y ?? this.y, z ?? this.z);
}
