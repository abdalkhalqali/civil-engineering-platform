import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

enum ViewportInteraction { orbit, pan }

enum ViewportProjection { perspective, orthographic }

class EngineeringViewport extends StatefulWidget {
  const EngineeringViewport({
    required this.interaction,
    required this.projection,
    required this.onElementSelected,
    super.key,
  });

  final ViewportInteraction interaction;
  final ViewportProjection projection;
  final ValueChanged<String> onElementSelected;

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
          if (hit != null) widget.onElementSelected(hit);
        },
        child: CustomPaint(
          painter: _EngineeringScenePainter(
            yaw: _yaw,
            pitch: _pitch,
            zoom: _zoom,
            pan: _pan,
            projection: widget.projection,
            selectedElement: _selected,
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
    final elements = <String, List<_Vec3>>{
      'Column C-01': [_Vec3(-3.2, -2.2, 0), _Vec3(-3.2, -2.2, 4)],
      'Column C-02': [_Vec3(3.2, -2.2, 0), _Vec3(3.2, -2.2, 4)],
      'Column C-03': [_Vec3(-3.2, 2.2, 0), _Vec3(-3.2, 2.2, 4)],
      'Column C-04': [_Vec3(3.2, 2.2, 0), _Vec3(3.2, 2.2, 4)],
    };
    String? closest;
    var distance = 22.0;
    for (final entry in elements.entries) {
      final a = camera.project(entry.value[0]);
      final b = camera.project(entry.value[1]);
      final d = _distanceToSegment(point, a, b);
      if (d < distance) {
        distance = d;
        closest = entry.key;
      }
    }
    return closest;
  }

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
    required this.selectedElement,
  });

  final double yaw;
  final double pitch;
  final double zoom;
  final Offset pan;
  final ViewportProjection projection;
  final String? selectedElement;

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

    _drawGrid(canvas, camera);
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
    for (var i = -10; i <= 10; i++) {
      final paint = i % 5 == 0 ? major : minor;
      _line(
        canvas,
        camera,
        _Vec3(i.toDouble(), -10, 0),
        _Vec3(i.toDouble(), 10, 0),
        paint,
      );
      _line(
        canvas,
        camera,
        _Vec3(-10, i.toDouble(), 0),
        _Vec3(10, i.toDouble(), 0),
        paint,
      );
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
    final posts = <String, List<_Vec3>>{
      'Column C-01': [_Vec3(-3.2, -2.2, 0), _Vec3(-3.2, -2.2, 4)],
      'Column C-02': [_Vec3(3.2, -2.2, 0), _Vec3(3.2, -2.2, 4)],
      'Column C-03': [_Vec3(-3.2, 2.2, 0), _Vec3(-3.2, 2.2, 4)],
      'Column C-04': [_Vec3(3.2, 2.2, 0), _Vec3(3.2, 2.2, 4)],
    };
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

    final corners = [
      const _Vec3(-3.2, -2.2, 4),
      const _Vec3(3.2, -2.2, 4),
      const _Vec3(3.2, 2.2, 4),
      const _Vec3(-3.2, 2.2, 4),
    ];
    final slab = corners.map(camera.project).toList();
    final slabPaint = Paint()
      ..color = const Color(0x403d8b95)
      ..style = PaintingStyle.fill;
    final slabOutline = Paint()
      ..color = const Color(0xff5ebbc0)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2;
    final polygon = Path()..addPolygon(slab, true);
    canvas.drawPath(polygon, slabPaint);
    canvas.drawPath(polygon, slabOutline);

    for (final entry in posts.entries) {
      final paint = entry.key == selectedElement ? selectedPaint : columnPaint;
      _line(canvas, camera, entry.value[0], entry.value[1], paint);
    }
    for (var i = 0; i < corners.length; i++) {
      _line(
        canvas,
        camera,
        corners[i],
        corners[(i + 1) % corners.length],
        beamPaint,
      );
    }
    final lowerCorners = corners.map((p) => p.copyWith(z: 0)).toList();
    for (var i = 0; i < lowerCorners.length; i++) {
      _line(
        canvas,
        camera,
        lowerCorners[i],
        lowerCorners[(i + 1) % lowerCorners.length],
        Paint()
          ..color = const Color(0xff7a9da2)
          ..strokeWidth = 2,
      );
    }
    _label(
      canvas,
      'SLAB · 4.00 m',
      camera.project(const _Vec3(-3.2, 2.2, 4)) + const Offset(-14, -14),
      const Color(0xff98d9d4),
    );
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
}

class _Vec3 {
  const _Vec3(this.x, this.y, this.z);

  final double x;
  final double y;
  final double z;

  _Vec3 copyWith({double? x, double? y, double? z}) =>
      _Vec3(x ?? this.x, y ?? this.y, z ?? this.z);
}
